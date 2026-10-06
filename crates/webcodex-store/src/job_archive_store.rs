//! Private, physically bounded archive locators. No output blobs or shared-WAL
//! fallback. A pinned reader may refuse a commit, never retain page versions.
use anyhow::{ensure, Context};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, TransactionBehavior};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use webcodex_core::job_archive::{
    ARCHIVE_LOCATOR_DB_MAX_BYTES, ARCHIVE_MAX_TERMINAL, ARCHIVE_METADATA_MAX_BYTES,
    ARCHIVE_RETENTION_SECS, ARCHIVE_SERVER_RESERVED_BYTES,
};
use webcodex_core::runner_job_receipt::ArchivedJobReceipt;

const DIRECTORY: &str = "job-archive-locator";
const FILE: &str = "archives.sqlite3";
const PAGE_SIZE: u64 = 4096;
const SCHEMA: &str = "CREATE TABLE wc_job_archives (
    job_id TEXT PRIMARY KEY CHECK(length(CAST(job_id AS BLOB)) BETWEEN 1 AND 128),
    client_id TEXT NOT NULL CHECK(length(CAST(client_id AS BLOB)) BETWEEN 1 AND 128),
    payload TEXT NOT NULL CHECK(length(CAST(payload AS BLOB)) <= 16384),
    committed_at INTEGER NOT NULL CHECK(committed_at > 0),
    expires_at INTEGER NOT NULL CHECK(expires_at = committed_at + 604800)
) WITHOUT ROWID";

pub(crate) struct JobArchiveStore {
    directory: PathBuf,
    // One connection and one transaction owner; no backups or connection pool.
    conn: Mutex<Option<ArchiveConnection>>,
}

pub(crate) struct ArchiveConnection {
    conn: Connection,
    directory_identity: (u64, u64),
    file_identity: (u64, u64),
}

impl std::ops::Deref for ArchiveConnection {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.conn
    }
}
impl std::ops::DerefMut for ArchiveConnection {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

impl JobArchiveStore {
    pub(crate) fn new(state_path: &Path) -> Self {
        Self {
            directory: state_path
                .parent()
                .expect("state database parent")
                .join(DIRECTORY),
            conn: Mutex::new(None),
        }
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.directory.join(FILE)
    }

    pub(crate) fn connection(&self) -> anyhow::Result<MutexGuard<'_, Option<ArchiveConnection>>> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| anyhow::anyhow!("archive lock poisoned"))?;
        if guard.is_none() {
            *guard = Some(self.open()?);
        }
        self.check_files()?;
        self.check_identity(guard.as_ref().unwrap())?;
        Ok(guard)
    }

    fn open(&self) -> anyhow::Result<ArchiveConnection> {
        check_ancestors(self.directory.parent().context("archive parent")?)?;
        match fs::symlink_metadata(&self.directory) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::DirBuilderExt;
                    let mut builder = fs::DirBuilder::new();
                    builder.mode(0o700);
                    if let Err(error) = builder.create(&self.directory) {
                        if error.kind() != std::io::ErrorKind::AlreadyExists {
                            return Err(error.into());
                        }
                    }
                }
                #[cfg(not(unix))]
                anyhow::bail!("private archive storage requires Unix ownership checks");
                File::open(self.directory.parent().unwrap())?.sync_all()?;
            }
            Err(error) => return Err(error.into()),
        }
        self.check_files()?;
        let path = self.path();
        if !path.try_exists()? {
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                match OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)
                {
                    Ok(file) => {
                        file.sync_all()?;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error.into()),
                }
            }
            File::open(&self.directory)?.sync_all()?;
        }
        self.check_files()?;
        let directory_identity = file_identity(&fs::symlink_metadata(&self.directory)?);
        let file_identity = file_identity(&fs::symlink_metadata(&path)?);
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        conn.busy_timeout(Duration::from_millis(250))?;
        // Existing incompatible headers must be refused before setting policy.
        let initialized = fs::metadata(&path)?.len() > 0;
        if initialized {
            ensure!(
                pragma_int(&conn, "page_size")? == PAGE_SIZE as i64,
                "archive page size incompatible"
            );
            ensure!(
                pragma_int(&conn, "auto_vacuum")? == 0,
                "archive auto vacuum incompatible"
            );
            ensure!(
                pragma_text(&conn, "journal_mode")? == "delete",
                "archive journal mode incompatible"
            );
        }
        conn.execute_batch(&format!(
            "PRAGMA page_size=4096; PRAGMA max_page_count={}; PRAGMA journal_mode=DELETE;
             PRAGMA synchronous=FULL; PRAGMA temp_store=MEMORY; PRAGMA cache_spill=OFF;
             PRAGMA auto_vacuum=NONE; PRAGMA busy_timeout=250;",
            ARCHIVE_LOCATOR_DB_MAX_BYTES / PAGE_SIZE
        ))?;
        for (name, expected) in [
            ("page_size", PAGE_SIZE as i64),
            (
                "max_page_count",
                (ARCHIVE_LOCATOR_DB_MAX_BYTES / PAGE_SIZE) as i64,
            ),
            ("synchronous", 2),
            ("temp_store", 2),
            ("cache_spill", 0),
            ("auto_vacuum", 0),
            ("busy_timeout", 250),
        ] {
            ensure!(
                pragma_int(&conn, name)? == expected,
                "archive policy {name} mismatch"
            );
        }
        ensure!(
            pragma_text(&conn, "journal_mode")? == "delete",
            "archive journal policy mismatch"
        );
        let version = pragma_int(&conn, "user_version")?;
        let schema: Vec<(String, String)> = conn
            .prepare("SELECT name, sql FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        if version == 0 && schema.is_empty() {
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(SCHEMA)?;
            tx.execute_batch("PRAGMA user_version=1")?;
            tx.commit()?;
        } else {
            ensure!(
                version == 1 && schema == vec![("wc_job_archives".into(), SCHEMA.into())],
                "archive schema incompatible"
            );
        }
        ensure!(
            conn.query_row("SELECT count(*) FROM wc_job_archives", [], |r| r
                .get::<_, i64>(0))?
                <= ARCHIVE_MAX_TERMINAL as i64,
            "archive row policy incompatible"
        );
        self.check_files()?;
        let connection = ArchiveConnection {
            conn,
            directory_identity,
            file_identity,
        };
        self.check_identity(&connection)?;
        Ok(connection)
    }

    pub(crate) fn upsert(&self, value: &ArchivedJobReceipt, now: i64) -> anyhow::Result<()> {
        value.validate(now).map_err(anyhow::Error::msg)?;
        let payload = serde_json::to_string(value)?;
        ensure!(
            payload.len() <= ARCHIVE_METADATA_MAX_BYTES,
            "archive receipt oversized"
        );
        let mut guard = self.connection()?;
        let tx = guard
            .as_mut()
            .unwrap()
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM wc_job_archives WHERE expires_at <= ?1", [now])?;
        // Replay never changes identity or renews the original deadline.
        let exists = tx
            .query_row(
                "SELECT 1 FROM wc_job_archives WHERE job_id=?1",
                [&value.archive.job_id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !exists {
            // Evict only an eligible terminal row, before insert so cardinality
            // never exceeds 256 even inside the transaction. Rollback restores
            // it if SQLITE_FULL or a pinned reader rejects the new commit.
            tx.execute("DELETE FROM wc_job_archives WHERE job_id IN (SELECT job_id FROM wc_job_archives ORDER BY committed_at,job_id LIMIT 1) AND (SELECT count(*) FROM wc_job_archives)>=?1", [ARCHIVE_MAX_TERMINAL as i64])?;
            tx.execute("INSERT INTO wc_job_archives(job_id,client_id,payload,committed_at,expires_at) VALUES (?1,?2,?3,?4,?5)",
                params![value.archive.job_id, value.archive.client_id, payload, value.archive.committed_at, value.archive.committed_at.checked_add(ARCHIVE_RETENTION_SECS).context("archive deadline overflow")?])?;
        }
        tx.commit()?;
        self.check_files()?;
        self.check_identity(guard.as_ref().unwrap())?;
        Ok(())
    }

    pub(crate) fn load(
        &self,
        job_id: &str,
        now: i64,
    ) -> anyhow::Result<Option<ArchivedJobReceipt>> {
        ensure!(
            !job_id.is_empty() && job_id.len() <= 128,
            "archive Job id invalid"
        );
        let guard = self.connection()?;
        let row: Option<(String, String, i64, i64)> = guard.as_ref().unwrap().query_row(
            "SELECT payload,client_id,committed_at,expires_at FROM wc_job_archives WHERE job_id=?1 AND expires_at>?2 AND length(CAST(payload AS BLOB))<=?3",
            params![job_id, now, ARCHIVE_METADATA_MAX_BYTES as i64], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        let Some((payload, client_id, committed_at, expires_at)) = row else {
            return Ok(None);
        };
        let value: ArchivedJobReceipt = serde_json::from_str(&payload)?;
        value.validate(now).map_err(anyhow::Error::msg)?;
        ensure!(
            value.archive.job_id == job_id
                && value.archive.client_id == client_id
                && value.archive.committed_at == committed_at
                && committed_at.checked_add(ARCHIVE_RETENTION_SECS) == Some(expires_at),
            "archive identity mismatch"
        );
        Ok(Some(value))
    }

    pub(crate) fn check_files(&self) -> anyhow::Result<()> {
        check_ancestors(self.directory.parent().context("archive parent")?)?;
        let directory = fs::symlink_metadata(&self.directory)?;
        check_private(&directory, true)?;
        let mut allocated = allocated_bytes(&directory);
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let limit = if name == FILE {
                ARCHIVE_LOCATOR_DB_MAX_BYTES
            } else if name == "archives.sqlite3-journal" {
                2 * ARCHIVE_LOCATOR_DB_MAX_BYTES + 65536
            } else {
                anyhow::bail!("unexpected archive storage entry")
            };
            let metadata = fs::symlink_metadata(entry.path())?;
            check_private(&metadata, false)?;
            ensure!(
                metadata.len() <= limit,
                "archive file exceeds physical policy"
            );
            allocated = allocated
                .checked_add(allocated_bytes(&metadata))
                .context("archive size overflow")?;
        }
        ensure!(
            allocated <= ARCHIVE_SERVER_RESERVED_BYTES,
            "archive reservation exceeded"
        );
        Ok(())
    }

    fn check_identity(&self, connection: &ArchiveConnection) -> anyhow::Result<()> {
        ensure!(
            file_identity(&fs::symlink_metadata(&self.directory)?) == connection.directory_identity
                && file_identity(&fs::symlink_metadata(self.path())?) == connection.file_identity,
            "archive owner was replaced while open"
        );
        Ok(())
    }
}

#[cfg(unix)]
fn file_identity(metadata: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}
#[cfg(not(unix))]
fn file_identity(_: &fs::Metadata) -> (u64, u64) {
    (0, 0)
}

fn pragma_int(conn: &Connection, name: &str) -> rusqlite::Result<i64> {
    conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
}
fn pragma_text(conn: &Connection, name: &str) -> rusqlite::Result<String> {
    conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
}

#[cfg(unix)]
fn check_private(metadata: &fs::Metadata, directory: bool) -> anyhow::Result<()> {
    use std::os::unix::fs::MetadataExt;
    unsafe extern "C" {
        fn geteuid() -> u32;
    }
    ensure!(
        metadata.uid() == unsafe { geteuid() },
        "archive owner mismatch"
    );
    ensure!(
        !metadata.file_type().is_symlink()
            && if directory {
                metadata.is_dir()
            } else {
                metadata.is_file() && metadata.nlink() == 1
            },
        "unsafe archive file type or links"
    );
    ensure!(
        metadata.mode() & 0o777 == if directory { 0o700 } else { 0o600 },
        "archive permissions incompatible"
    );
    Ok(())
}
#[cfg(not(unix))]
fn check_private(_: &fs::Metadata, _: bool) -> anyhow::Result<()> {
    anyhow::bail!("private archive storage requires Unix ownership checks")
}

fn check_ancestors(path: &Path) -> anyhow::Result<()> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor)?;
        ensure!(
            metadata.is_dir() && !metadata.file_type().is_symlink(),
            "unsafe archive ancestor"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            // Shared writable ancestors require the sticky directory protection.
            ensure!(
                metadata.mode() & 0o022 == 0 || metadata.mode() & 0o1000 != 0,
                "writable archive ancestor"
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
fn allocated_bytes(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.blocks().saturating_mul(512)
}
#[cfg(not(unix))]
fn allocated_bytes(metadata: &fs::Metadata) -> u64 {
    metadata.len()
}
