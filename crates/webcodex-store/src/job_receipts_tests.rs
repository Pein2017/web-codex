use crate::Database;
use webcodex_core::runner_job_receipt::{
    RetainedJobReceipt, RunnerAccessGroup, JOB_RECEIPT_PAYLOAD_MAX_BYTES,
};
use webcodex_core::runner_protocol::{
    JOB_INVENTORY_MAX_TERMINAL_JOBS, JOB_TERMINAL_RETENTION_SECS,
};

fn receipt(now: i64, id: &str) -> RetainedJobReceipt {
    RetainedJobReceipt {
        client_id: "receipt-runner".into(), runner_instance_id: "old-instance".into(), auth_group: None, owner_at_admission: Some("alice".into()), kind: "shell".into(), terminal_observed_at: now, expires_at: now + JOB_TERMINAL_RETENTION_SECS,
        snapshot: serde_json::from_value(serde_json::json!({
            "job_id": id, "request_id": format!("req-{id}"), "status": "completed", "update_seq": 3,
            "created_at": now - 2, "started_at": now - 1, "ended_at": now, "exit_code": 0,
            "context": {"command_preview": "echo done"},
            "stdout": {"tail": "done\n", "first_retained_line": 8, "next_line": 9, "truncated": true}
        })).unwrap(),
    }
}

fn archived(now: i64, id: &str) -> webcodex_core::runner_job_receipt::ArchivedJobReceipt {
    let mut value = receipt(now, id);
    value.snapshot.context.runtime_project_id = Some("agent:receipt-runner:p".into());
    value.snapshot.context.project_cwd = Some("/project".into());
    value.snapshot.stdout.tail.clear();
    value.snapshot.stdout.first_retained_line = value.snapshot.stdout.next_line;
    webcodex_core::runner_job_receipt::ArchivedJobReceipt {
        archive: webcodex_core::job_archive::JobArchiveDescriptor {
            job_id: id.into(),
            request_id: format!("req-{id}"),
            client_id: value.client_id.clone(),
            runner_instance_id: value.runner_instance_id.clone(),
            project_id: "agent:receipt-runner:p".into(),
            project_root_fingerprint: format!("wc_projroot_{}", "1".repeat(64)),
            root_incarnation: "2".repeat(64),
            committed_at: now,
            stdout: webcodex_core::job_archive::JobArchiveStream {
                retained_bytes: 5,
                next_line: 2,
                loss_reason: None,
            },
            stderr: webcodex_core::job_archive::JobArchiveStream {
                retained_bytes: 0,
                next_line: 1,
                loss_reason: None,
            },
        },
        receipt: value,
    }
}

#[test]
fn archived_receipt_exact_id_survives_ordinary_expiry_inventory_pruning_and_reopen() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("receipts.db");
    let db = Database::open(&path).unwrap();
    let now = chrono::Utc::now().timestamp();
    let original = archived(now - 2 * JOB_TERMINAL_RETENTION_SECS, "original");
    db.upsert_job_archive(&original, now).unwrap();
    for n in 0..70 {
        db.upsert_job_receipt(&receipt(now, &format!("new-{n:03}")), now)
            .unwrap();
    }
    db.prune_job_receipts(now).unwrap();
    assert_eq!(
        db.load_job_receipts(now).unwrap().len(),
        JOB_INVENTORY_MAX_TERMINAL_JOBS
    );
    assert_eq!(
        db.load_job_archive("original", now).unwrap(),
        Some(original.clone())
    );
    assert!(db.load_job_archive("unknown", now).unwrap().is_none());
    drop(db);
    let reopened = Database::open(&path).unwrap();
    assert_eq!(
        reopened.load_job_archive("original", now).unwrap(),
        Some(original.clone())
    );
    assert!(reopened
        .load_job_archive(
            "original",
            original.archive.committed_at + webcodex_core::job_archive::ARCHIVE_RETENTION_SECS
        )
        .unwrap()
        .is_none());
}

#[test]
fn archived_receipt_replay_cannot_replace_identity_and_malformed_metadata_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("receipts.db")).unwrap();
    let now = chrono::Utc::now().timestamp();
    let original = archived(now, "job");
    db.upsert_job_archive(&original, now).unwrap();
    let mut replacement = original.clone();
    replacement.receipt.owner_at_admission = Some("mallory".into());
    replacement.archive.committed_at += 1;
    db.upsert_job_archive(&replacement, now + 1).unwrap();
    assert_eq!(
        db.load_job_archive("job", now).unwrap(),
        Some(original.clone())
    );
    assert!(db
        .load_job_archive(
            "job",
            original.archive.committed_at + webcodex_core::job_archive::ARCHIVE_RETENTION_SECS
        )
        .unwrap()
        .is_none());
    db.job_archives
        .connection()
        .unwrap()
        .as_ref()
        .unwrap()
        .execute(
            "UPDATE wc_job_archives SET payload='{}' WHERE job_id='job'",
            [],
        )
        .unwrap();
    assert!(db.load_job_archive("job", now).is_err());
}

#[cfg(unix)]
fn archive_allocated(path: &std::path::Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    let directory = path.parent().unwrap();
    let mut bytes = std::fs::metadata(directory).unwrap().blocks() * 512;
    for entry in std::fs::read_dir(directory).unwrap() {
        match std::fs::symlink_metadata(entry.unwrap().path()) {
            Ok(metadata) => bytes += metadata.blocks() * 512,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("{error}"),
        }
    }
    bytes
}

fn maximum_archive(now: i64, id: &str) -> webcodex_core::runner_job_receipt::ArchivedJobReceipt {
    let mut value = archived(now, id);
    value.receipt.snapshot.error = Some(String::new());
    let overhead = serde_json::to_vec(&value).unwrap().len();
    value.receipt.snapshot.error =
        Some("x".repeat(webcodex_core::job_archive::ARCHIVE_METADATA_MAX_BYTES - overhead));
    assert_eq!(
        serde_json::to_vec(&value).unwrap().len(),
        webcodex_core::job_archive::ARCHIVE_METADATA_MAX_BYTES
    );
    value.validate(now).unwrap();
    value
}

#[test]
#[cfg(unix)]
fn archive_legacy_wal_retained_reader_is_a_physical_bound_counterexample() {
    use rusqlite::{params, Connection};
    use std::os::unix::fs::MetadataExt;
    use webcodex_core::job_archive::ARCHIVE_SERVER_RESERVED_BYTES;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("shared.db");
    let writer = Connection::open(&path).unwrap();
    writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; CREATE TABLE wc_job_archives(job_id TEXT PRIMARY KEY,payload TEXT)").unwrap();
    writer
        .execute(
            "INSERT INTO wc_job_archives VALUES ('original','original')",
            [],
        )
        .unwrap();
    let reader = Connection::open(&path).unwrap();
    reader.execute_batch("BEGIN").unwrap();
    reader
        .query_row("SELECT payload FROM wc_job_archives", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();
    for n in 0..1400 {
        writer
            .execute(
                "INSERT INTO wc_job_archives VALUES (?1,?2)",
                params![format!("job-{n:04}"), "x".repeat(16384)],
            )
            .unwrap();
        writer.execute("DELETE FROM wc_job_archives WHERE job_id IN (SELECT job_id FROM wc_job_archives ORDER BY job_id DESC LIMIT -1 OFFSET 256)", []).unwrap();
    }
    let rows: i64 = writer
        .query_row("SELECT count(*) FROM wc_job_archives", [], |r| r.get(0))
        .unwrap();
    let wal_bytes = std::fs::metadata(temp.path().join("shared.db-wal"))
        .unwrap()
        .blocks()
        * 512;
    assert_eq!(rows, 256);
    assert!(
        wal_bytes > ARCHIVE_SERVER_RESERVED_BYTES,
        "legacy WAL {wal_bytes} must falsify reserve despite bounded rows"
    );
    eprintln!("legacy counterexample: {rows} rows, pinned WAL allocated={wal_bytes}");
}

#[test]
#[cfg(unix)]
fn archive_reader_busy_keeps_original_and_shared_receipts_then_repeated_writes_stay_bounded() {
    use rusqlite::{Connection, ErrorCode};
    use webcodex_core::job_archive::{ARCHIVE_MAX_TERMINAL, ARCHIVE_SERVER_RESERVED_BYTES};
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("state.db")).unwrap();
    let now = chrono::Utc::now().timestamp();
    let original = archived(now, "original");
    db.upsert_job_archive(&original, now).unwrap();
    let reader = Connection::open(db.job_archive_path()).unwrap();
    reader.execute_batch("BEGIN").unwrap();
    reader
        .query_row("SELECT payload FROM wc_job_archives", [], |r| {
            r.get::<_, String>(0)
        })
        .unwrap();
    let start = std::time::Instant::now();
    for n in 0..3 {
        let error = db
            .upsert_job_archive(&maximum_archive(now, &format!("blocked-{n}")), now)
            .unwrap_err();
        assert_eq!(
            error
                .downcast_ref::<rusqlite::Error>()
                .unwrap()
                .sqlite_error_code(),
            Some(ErrorCode::DatabaseBusy)
        );
        assert!(archive_allocated(&db.job_archive_path()) <= ARCHIVE_SERVER_RESERVED_BYTES);
        assert_eq!(
            db.load_job_archive("original", now).unwrap(),
            Some(original.clone())
        );
    }
    assert!(start.elapsed() < std::time::Duration::from_secs(3));
    db.upsert_job_receipt(&receipt(now, "ordinary"), now)
        .unwrap();
    assert_eq!(db.load_job_receipts(now).unwrap().len(), 1);
    assert_eq!(
        db.conn_for_tests()
            .query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "wal"
    );
    assert_eq!(
        db.conn_for_tests()
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name='wc_job_archives'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    reader.execute_batch("ROLLBACK").unwrap();
    for n in 0..600 {
        db.upsert_job_archive(&archived(now, &format!("job-{n:04}")), now)
            .unwrap();
        assert!(archive_allocated(&db.job_archive_path()) <= ARCHIVE_SERVER_RESERVED_BYTES);
    }
    let guard = db.job_archives.connection().unwrap();
    let count: i64 = guard
        .as_ref()
        .unwrap()
        .query_row("SELECT count(*) FROM wc_job_archives", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, ARCHIVE_MAX_TERMINAL as i64);
}

#[test]
#[cfg(unix)]
fn archive_sqlite_full_and_live_rollback_journal_respect_physical_reserve() {
    use std::sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    };
    use webcodex_core::job_archive::{
        ARCHIVE_LOCATOR_DB_MAX_BYTES, ARCHIVE_MAX_TERMINAL, ARCHIVE_SERVER_RESERVED_BYTES,
    };
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("state.db")).unwrap();
    let now = chrono::Utc::now().timestamp();
    let original = maximum_archive(now, "original");
    db.upsert_job_archive(&original, now).unwrap();
    let running = Arc::new(AtomicBool::new(true));
    let peak = Arc::new(AtomicU64::new(0));
    let sampler = {
        let running = running.clone();
        let peak = peak.clone();
        let path = db.job_archive_path();
        std::thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let allocated = archive_allocated(&path);
                peak.fetch_max(allocated, Ordering::Relaxed);
                std::thread::yield_now();
            }
        })
    };
    let mut full_at = None;
    for n in 0..ARCHIVE_MAX_TERMINAL {
        match db.upsert_job_archive(&maximum_archive(now, &format!("full-{n:04}")), now) {
            Ok(()) => {}
            Err(error) => {
                assert_eq!(
                    error
                        .downcast_ref::<rusqlite::Error>()
                        .unwrap()
                        .sqlite_error_code(),
                    Some(rusqlite::ErrorCode::DiskFull)
                );
                full_at = Some(n);
                break;
            }
        }
    }
    assert!(full_at.is_some_and(|n| n < ARCHIVE_MAX_TERMINAL));
    assert_eq!(
        db.load_job_archive("original", now).unwrap(),
        Some(original.clone())
    );
    // Touch all original pages inside one transaction and inspect its journal
    // before rollback, not merely the smaller post-commit footprint.
    {
        let mut guard = db.job_archives.connection().unwrap();
        let tx = guard.as_mut().unwrap().transaction().unwrap();
        tx.execute(
            "UPDATE wc_job_archives SET payload=replace(payload,'x','y')",
            [],
        )
        .unwrap();
        tx.cache_flush().unwrap();
        let journal = db
            .job_archive_path()
            .with_file_name("archives.sqlite3-journal");
        assert!(std::fs::metadata(&journal).unwrap().len() > ARCHIVE_LOCATOR_DB_MAX_BYTES / 2);
        let during = archive_allocated(&db.job_archive_path());
        peak.fetch_max(during, Ordering::Relaxed);
        assert!(during <= ARCHIVE_SERVER_RESERVED_BYTES);
        tx.rollback().unwrap();
    }
    running.store(false, Ordering::Relaxed);
    sampler.join().unwrap();
    assert!(peak.load(Ordering::Relaxed) <= ARCHIVE_SERVER_RESERVED_BYTES);
    assert!(
        std::fs::metadata(db.job_archive_path()).unwrap().len() <= ARCHIVE_LOCATOR_DB_MAX_BYTES
    );
    eprintln!(
        "bounded full: failed insertion={full_at:?}, peak allocated={} bytes",
        peak.load(Ordering::Relaxed)
    );
    drop(db);
    let reopened = Database::open(&temp.path().join("state.db")).unwrap();
    assert_eq!(
        reopened.load_job_archive("original", now).unwrap(),
        Some(original)
    );
    let fresh = archived(
        now + webcodex_core::job_archive::ARCHIVE_RETENTION_SECS,
        "fresh",
    );
    reopened
        .upsert_job_archive(&fresh, fresh.archive.committed_at)
        .unwrap();
    assert!(reopened
        .load_job_archive("original", fresh.archive.committed_at)
        .unwrap()
        .is_none());
    assert!(archive_allocated(&reopened.job_archive_path()) <= ARCHIVE_SERVER_RESERVED_BYTES);
}

#[test]
#[cfg(unix)]
fn archive_private_storage_policy_unknown_files_links_and_oversize_fail_archive_only() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    for case in [
        "wal",
        "shm",
        "unknown",
        "permissions",
        "oversize",
        "symlink",
        "hardlink",
        "directory-link",
        "pages",
        "schema",
        "vacuum",
        "wal-policy",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("state.db");
        let db = Database::open(&path).unwrap();
        let now = chrono::Utc::now().timestamp();
        db.upsert_job_archive(&archived(now, "original"), now)
            .unwrap();
        let archive_path = db.job_archive_path();
        let directory = archive_path.parent().unwrap().to_path_buf();
        assert_eq!(
            std::fs::metadata(&archive_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        drop(db);
        match case {
            "wal" | "shm" | "unknown" => {
                std::fs::File::create(directory.join(if case == "wal" {
                    "archives.sqlite3-wal"
                } else if case == "shm" {
                    "archives.sqlite3-shm"
                } else {
                    "unknown"
                }))
                .unwrap();
            }
            "permissions" => {
                std::fs::set_permissions(&archive_path, std::fs::Permissions::from_mode(0o644))
                    .unwrap()
            }
            "oversize" => std::fs::OpenOptions::new()
                .write(true)
                .open(&archive_path)
                .unwrap()
                .set_len(webcodex_core::job_archive::ARCHIVE_LOCATOR_DB_MAX_BYTES + 4096)
                .unwrap(),
            "symlink" => {
                std::fs::rename(&archive_path, temp.path().join("actual.db")).unwrap();
                symlink(temp.path().join("actual.db"), &archive_path).unwrap();
            }
            "hardlink" => std::fs::hard_link(&archive_path, temp.path().join("other.db")).unwrap(),
            "directory-link" => {
                let moved = temp.path().join("moved");
                std::fs::rename(&directory, &moved).unwrap();
                symlink(moved, &directory).unwrap();
            }
            "pages" | "vacuum" => {
                // Deliberately incompatible operator modification; VACUUM is
                // confined to this negative fixture, never the archive writer.
                let conn = rusqlite::Connection::open(&archive_path).unwrap();
                conn.execute_batch(if case == "pages" {
                    "PRAGMA page_size=8192; VACUUM"
                } else {
                    "PRAGMA auto_vacuum=FULL; VACUUM"
                })
                .unwrap();
            }
            "schema" => {
                rusqlite::Connection::open(&archive_path)
                    .unwrap()
                    .execute_batch("CREATE TABLE extra(value TEXT)")
                    .unwrap();
            }
            "wal-policy" => {
                rusqlite::Connection::open(&archive_path)
                    .unwrap()
                    .execute_batch("PRAGMA journal_mode=WAL")
                    .unwrap();
            }
            _ => unreachable!(),
        }
        let db = Database::open(&path).unwrap();
        assert!(db.load_job_archive("original", now).is_err(), "case {case}");
        db.upsert_job_receipt(&receipt(now, "ordinary"), now)
            .unwrap();
        assert_eq!(db.load_job_receipts(now).unwrap().len(), 1);
    }
}

#[test]
#[cfg(unix)]
fn archive_concurrent_writers_share_one_owner_and_replaced_open_file_fails_closed() {
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;
    let temp = tempfile::tempdir().unwrap();
    let db = Arc::new(Database::open(&temp.path().join("state.db")).unwrap());
    let now = chrono::Utc::now().timestamp();
    let original = archived(now, "original");
    db.upsert_job_archive(&original, now).unwrap();
    let other = Database::open(&temp.path().join("other-state.db")).unwrap();
    assert_eq!(db.job_archive_path(), other.job_archive_path());
    assert_eq!(
        other.load_job_archive("original", now).unwrap(),
        Some(original.clone())
    );
    let workers: Vec<_> = (0..8)
        .map(|worker| {
            let db = db.clone();
            let mut value = original.clone();
            value.receipt.owner_at_admission = Some(format!("writer-{worker}"));
            std::thread::spawn(move || {
                for n in 0..40 {
                    db.upsert_job_archive(&value, now).unwrap();
                    db.upsert_job_archive(&archived(now, &format!("job-{worker}-{n:03}")), now)
                        .unwrap();
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(
        db.load_job_archive("original", now).unwrap(),
        Some(original)
    );
    assert!(
        archive_allocated(&db.job_archive_path())
            <= webcodex_core::job_archive::ARCHIVE_SERVER_RESERVED_BYTES
    );
    let archive_path = db.job_archive_path();
    std::fs::rename(&archive_path, temp.path().join("replaced.db")).unwrap();
    let replacement = std::fs::File::create(&archive_path).unwrap();
    replacement
        .set_permissions(std::fs::Permissions::from_mode(0o600))
        .unwrap();
    assert!(db.load_job_archive("original", now).is_err());
    assert!(db
        .upsert_job_archive(&archived(now, "after-replace"), now)
        .is_err());
    db.upsert_job_receipt(&receipt(now, "ordinary"), now)
        .unwrap();
}

#[test]
#[ignore = "real child crash fixture; called by archive_hot_journal_recovery_preserves_original"]
fn archive_hot_journal_child() {
    let path =
        std::env::var_os("WEBCODEX_ARCHIVE_CRASH_FIXTURE").expect("child-local fixture path");
    let db = Database::open(&std::path::PathBuf::from(path)).unwrap();
    let guard = db.job_archives.connection().unwrap();
    let conn = guard.as_ref().unwrap();
    conn.execute_batch(
        "BEGIN IMMEDIATE; UPDATE wc_job_archives SET payload=replace(payload,'x','y')",
    )
    .unwrap();
    // The child-local fsync interceptor exits at the journal's durable hot
    // header during this real COMMIT, before it can remove the journal.
    conn.execute_batch("COMMIT").unwrap();
    panic!("crash interceptor did not run");
}

#[test]
#[cfg(unix)]
fn archive_hot_journal_recovery_preserves_original() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("state.db");
    let now = chrono::Utc::now().timestamp();
    let original = maximum_archive(now, "original");
    let db = Database::open(&path).unwrap();
    db.upsert_job_archive(&original, now).unwrap();
    let archive_path = db.job_archive_path();
    drop(db);
    // Fault injection at the actual SQLite durability boundary. Intercept only
    // this private journal and only after real sync succeeds with hot magic;
    // no timing race, production hook, process-global environment or retry.
    let preload = temp.path().join("crash.so");
    let mut compiler = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-x", "c", "-", "-o"])
        .arg(&preload)
        .arg("-ldl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        compiler.stdin.take().unwrap().write_all(br#"
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
static void crash_hot_journal(int fd, int result) {
    char link[64], path[4096];
    unsigned char bytes[8], magic[8] = {0xd9,0xd5,0x05,0xf9,0x20,0xa1,0x63,0xd7};
    snprintf(link, sizeof(link), "/proc/self/fd/%d", fd);
    ssize_t n = readlink(link, path, sizeof(path)-1);
    if (result != 0 || n < 0) return;
    path[n] = 0;
    const char *suffix = "/job-archive-locator/archives.sqlite3-journal";
    size_t len = strlen(suffix);
    if ((size_t)n >= len && !strcmp(path+n-len,suffix) && pread(fd,bytes,8,0)==8 && !memcmp(bytes,magic,8)) _exit(73);
}
int fsync(int fd) {
    int (*real)(int) = dlsym(RTLD_NEXT,"fsync");
    int result = real(fd); crash_hot_journal(fd,result); return result;
}
int fdatasync(int fd) {
    int (*real)(int) = dlsym(RTLD_NEXT,"fdatasync");
    int result = real(fd); crash_hot_journal(fd,result); return result;
}
"#).unwrap();
    }
    let compiled = compiler.wait_with_output().unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "job_receipts_tests::archive_hot_journal_child",
            "--nocapture",
        ])
        .env("WEBCODEX_ARCHIVE_CRASH_FIXTURE", &path)
        .env("LD_PRELOAD", &preload)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(73),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let journal = archive_path.with_file_name("archives.sqlite3-journal");
    let bytes = std::fs::read(&journal).unwrap();
    assert_eq!(
        &bytes[..8],
        &[0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7]
    );
    assert!(
        archive_allocated(&archive_path)
            <= webcodex_core::job_archive::ARCHIVE_SERVER_RESERVED_BYTES
    );
    let reopened = Database::open(&path).unwrap();
    assert_eq!(
        reopened.load_job_archive("original", now).unwrap(),
        Some(original)
    );
    assert!(!journal.exists());
}

#[test]
fn job_receipts_upgrade_preserves_legacy_deadline_without_accepting_arbitrary_ttl() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("legacy.db");
    let now = chrono::Utc::now().timestamp();
    let db = Database::open(&path).unwrap();
    let mut legacy = receipt(now - 890, "legacy");
    db.upsert_job_receipt(&legacy, now).unwrap();
    // Reproduce the old binary's persisted 15-minute contract, independently
    // of the current writer and its 24-hour constant.
    legacy.expires_at = legacy.terminal_observed_at + 900;
    db.conn_for_tests()
        .execute(
            "UPDATE wc_job_receipts SET expires_at = terminal_observed_at + 900",
            [],
        )
        .unwrap();
    drop(db);
    let db = Database::open(&path).unwrap();
    assert_eq!(db.load_job_receipts(now).unwrap(), vec![legacy.clone()]);
    for ttl in [899, 901, JOB_TERMINAL_RETENTION_SECS + 1] {
        let mut invalid = receipt(now, "invalid");
        invalid.expires_at = now + ttl;
        assert!(db.upsert_job_receipt(&invalid, now).is_err());
    }
    assert!(db.load_job_receipts(legacy.expires_at).unwrap().is_empty());
}

#[test]
fn job_receipts_schema_additive_reopen_first_write_and_fixed_expiry() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("receipts.db");
    // Current pre-feature schema opens additively, including existing data.
    let db = Database::open(&path).unwrap();
    db.conn_for_tests().execute_batch("DROP TABLE wc_job_receipts; CREATE TABLE legacy_marker(value TEXT); INSERT INTO legacy_marker VALUES ('preserved')").unwrap();
    drop(db);
    let now = chrono::Utc::now().timestamp();
    let original = receipt(now - JOB_TERMINAL_RETENTION_SECS + 10, "job-reopen");
    let db = Database::open(&path).unwrap();
    db.upsert_job_receipt(&original, now).unwrap();
    let mut replay = original.clone();
    replay.terminal_observed_at = now;
    replay.expires_at = now + JOB_TERMINAL_RETENTION_SECS;
    replay.snapshot.status = "failed".into();
    replay.snapshot.exit_code = Some(7);
    replay.owner_at_admission = Some("mallory".into());
    db.upsert_job_receipt(&replay, now).unwrap();
    drop(db);
    for _ in 0..3 {
        let db = Database::open(&path).unwrap();
        assert_eq!(db.load_job_receipts(now).unwrap(), vec![original.clone()]);
        assert_eq!(
            db.conn_for_tests()
                .query_row("SELECT value FROM legacy_marker", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "preserved"
        );
    }
    let db = Database::open(&path).unwrap();
    assert_eq!(db.prune_job_receipts(original.expires_at).unwrap(), 1);
    assert_eq!(db.prune_job_receipts(original.expires_at).unwrap(), 0);
    assert!(db
        .load_job_receipts(original.expires_at)
        .unwrap()
        .is_empty());
}

#[test]
fn job_receipts_storage_bounded_per_logical_runner_and_pruned_on_open() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("receipts.db");
    let db = Database::open(&path).unwrap();
    let now = chrono::Utc::now().timestamp();
    for i in 0..JOB_INVENTORY_MAX_TERMINAL_JOBS + 5 {
        let mut value = receipt(now - 100 + i as i64, &format!("job-{i:03}"));
        value.runner_instance_id = format!("instance-{i}");
        db.upsert_job_receipt(&value, now).unwrap();
    }
    let rows = db.load_job_receipts(now).unwrap();
    assert_eq!(rows.len(), JOB_INVENTORY_MAX_TERMINAL_JOBS);
    assert_eq!(rows[0].snapshot.job_id, "job-005");
    db.conn_for_tests()
        .execute("UPDATE wc_job_receipts SET expires_at = ?1", [now])
        .unwrap();
    drop(db);
    let reopened = Database::open(&path).unwrap();
    assert!(reopened.load_job_receipts(now).unwrap().is_empty());
}

#[test]
fn job_receipts_malformed_oversized_active_and_authorization_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let db = Database::open(&temp.path().join("receipts.db")).unwrap();
    let now = chrono::Utc::now().timestamp();
    for id in [
        "valid",
        "bad-json",
        "bad-auth",
        "bad-owner",
        "active",
        "oversized",
        "wrong-id",
        "argv",
        "detached",
        "cursor",
    ] {
        db.upsert_job_receipt(&receipt(now, id), now).unwrap();
    }
    {
        let conn = db.conn_for_tests();
        conn.execute_batch("UPDATE wc_job_receipts SET snapshot = '{' WHERE job_id='bad-json';
            UPDATE wc_job_receipts SET auth_kind = 'unknown' WHERE job_id='bad-auth';
            UPDATE wc_job_receipts SET owner_at_admission = NULL WHERE job_id='bad-owner';
            UPDATE wc_job_receipts SET snapshot = json_set(snapshot, '$.status', 'running') WHERE job_id='active';
            UPDATE wc_job_receipts SET snapshot = json_set(snapshot, '$.job_id', 'other') WHERE job_id='wrong-id';
            UPDATE wc_job_receipts SET snapshot = json_set(snapshot, '$.context.validation', json('{}')) WHERE job_id='argv';
            UPDATE wc_job_receipts SET kind = 'run_detached_process' WHERE job_id='detached';
            UPDATE wc_job_receipts SET snapshot = json_set(snapshot, '$.stdout.next_line', 1) WHERE job_id='cursor';").unwrap();
        conn.execute(
            "UPDATE wc_job_receipts SET snapshot = ?1 WHERE job_id='oversized'",
            ["x".repeat(JOB_RECEIPT_PAYLOAD_MAX_BYTES + 1)],
        )
        .unwrap();
    }
    assert_eq!(
        db.load_job_receipts(now).unwrap(),
        vec![receipt(now, "valid")]
    );
    let mut bad = receipt(now, "reject");
    bad.snapshot.status = "running".into();
    assert!(db.upsert_job_receipt(&bad, now).is_err());
    bad.snapshot.status = "completed".into();
    bad.auth_group = Some(RunnerAccessGroup::SharedKey("plaintext".into()));
    assert!(db.upsert_job_receipt(&bad, now).is_err());
}

#[test]
fn job_receipts_failed_result_roundtrips_all_explicit_authorization_partitions() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("receipts.db");
    let now = chrono::Utc::now().timestamp();
    let mut expected = Vec::new();
    let db = Database::open(&path).unwrap();
    for (index, group) in [
        None,
        Some(RunnerAccessGroup::SharedKey("a".repeat(64))),
        Some(RunnerAccessGroup::ProjectGrant("grant".into())),
        Some(RunnerAccessGroup::OpenAnonymous),
    ]
    .into_iter()
    .enumerate()
    {
        let mut value = receipt(now, &format!("failed-{index}"));
        value.auth_group = group;
        value.owner_at_admission = None;
        value.snapshot.status = "failed".into();
        value.snapshot.exit_code = Some(7);
        db.upsert_job_receipt(&value, now).unwrap();
        expected.push(value);
    }
    drop(db);
    assert_eq!(
        Database::open(&path)
            .unwrap()
            .load_job_receipts(now)
            .unwrap(),
        expected
    );
}
