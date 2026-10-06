//! Private bounded output prefix retention. A full queue drops evidence, never pipe drainage.
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use webcodex_core::job_archive::*;
use webcodex_core::runner_protocol::ShellJobContext;

const SERVER_RECEIPT_OVERHEAD_BYTES: u64 = 2 * ARCHIVE_METADATA_MAX_BYTES as u64;
const RESERVATION_BYTES: u64 = 2 * ARCHIVE_STREAM_MAX_BYTES + 4 * ARCHIVE_METADATA_MAX_BYTES as u64;
const MAX_CHUNK: usize = 64 * 1024;
const CAPTURE_QUEUE_CHUNKS: usize = 8;
const RECEIPT: &str = "terminal.json";

#[derive(Debug, Clone)]
pub(crate) struct ArchiveStore {
    pub(crate) root: PathBuf,
    namespace: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ArchiveCapture {
    tx: mpsc::SyncSender<Message>,
    loss: Arc<Mutex<Option<String>>>,
}

#[derive(Debug)]
enum Message {
    Chunk(bool, String),
    Finish(mpsc::Sender<Option<JobArchiveDescriptor>>),
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(unix)]
fn incarnation(root: &Path) -> Result<String, String> {
    use std::os::unix::fs::MetadataExt;
    let canonical = root
        .canonicalize()
        .map_err(|_| "archive root unavailable")?;
    let metadata = fs::metadata(&canonical).map_err(|_| "archive root unavailable")?;
    if !metadata.is_dir() {
        return Err("archive root unavailable".into());
    }
    Ok(digest(
        format!(
            "webcodex-job-root-v1\0{}\0{}\0{}",
            canonical.display(),
            metadata.dev(),
            metadata.ino()
        )
        .as_bytes(),
    ))
}
#[cfg(not(unix))]
fn incarnation(_: &Path) -> Result<String, String> {
    Err("archive root incarnation unsupported".into())
}

fn secure_dir(path: &Path) -> Result<(), String> {
    #[cfg(not(unix))]
    {
        let _ = path;
        return Err("private archive unavailable".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        if !path.exists() {
            let parent = path.parent().ok_or("archive root unavailable")?;
            if !parent.exists() {
                secure_dir(parent)?;
            }
            fs::DirBuilder::new()
                .mode(0o700)
                .create(path)
                .map_err(|_| "archive directory creation failed")?;
        }
        owned_dir(path)
    }
}

fn owned_dir(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let m = fs::symlink_metadata(path).map_err(|_| "archive directory unavailable")?;
        if !m.is_dir()
            || m.file_type().is_symlink()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o077 != 0
        {
            return Err("archive directory ownership invalid".into());
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err("private archive unavailable".into())
    }
}

fn allocated_bytes(metadata: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.len().max(metadata.blocks().saturating_mul(512))
    }
    #[cfg(not(unix))]
    {
        metadata.len()
    }
}

fn private_file(path: &Path, create: bool) -> Result<File, String> {
    #[cfg(not(unix))]
    {
        let _ = (path, create);
        return Err("private archive unavailable".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let file = OpenOptions::new()
            .read(true)
            .write(create)
            .create_new(create)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)
            .map_err(|_| "archive file unavailable")?;
        let m = file.metadata().map_err(|_| "archive file unavailable")?;
        if !m.is_file()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o077 != 0
            || m.nlink() != 1
        {
            return Err("archive file ownership invalid".into());
        }
        Ok(file)
    }
}

impl ArchiveStore {
    pub(crate) fn default_for_runner(client_id: &str, server_url: &str) -> Result<Self, String> {
        let endpoint = server_url.trim().trim_end_matches('/');
        if endpoint.is_empty() || client_id.is_empty() {
            return Err("archive namespace unavailable".into());
        }
        Ok(Self {
            root: webcodex_runner_config::paths::default_client_state_base_dir()?
                .join("runner-job-archives-v1"),
            namespace: digest(
                format!("webcodex-job-archive-v1\0{endpoint}\0{client_id}").as_bytes(),
            ),
        })
    }

    fn job_dir(&self, job: &str) -> PathBuf {
        self.root.join(&self.namespace).join(digest(job.as_bytes()))
    }

    fn lock(&self) -> Result<File, String> {
        for ancestor in self.root.ancestors() {
            if let Ok(metadata) = fs::symlink_metadata(ancestor) {
                if metadata.file_type().is_symlink() {
                    return Err("archive root symlink rejected".into());
                }
            }
        }
        secure_dir(&self.root)?;
        let path = self.root.join("quota.lock");
        let file = match private_file(&path, true) {
            Ok(f) => f,
            Err(_) => private_file(&path, false)?,
        };
        file.try_lock_exclusive()
            .map_err(|_| "archive quota busy")?;
        Ok(file)
    }

    // Count all namespaces, active reservations, receipt/temp overhead and bytes.
    // Unknown entries fail closed; cleanup only accepts verified committed entries.
    fn account_and_prune(&self, now: i64, extra: u64) -> Result<(), String> {
        let mut used = allocated_bytes(
            &fs::symlink_metadata(&self.root).map_err(|_| "archive accounting failed")?,
        );
        let mut terminal = Vec::new();
        let mut inspected = 0usize;
        for namespace in fs::read_dir(&self.root).map_err(|_| "archive accounting failed")? {
            inspected += 1;
            if inspected > 4096 {
                return Err("archive accounting bound exceeded".into());
            }
            let path = namespace.map_err(|_| "archive accounting failed")?.path();
            if path.file_name().is_some_and(|n| n == "quota.lock") {
                used += allocated_bytes(
                    &private_file(&path, false)?
                        .metadata()
                        .map_err(|_| "archive accounting failed")?,
                );
                continue;
            }
            owned_dir(&path)?;
            used = used.saturating_add(allocated_bytes(
                &fs::symlink_metadata(&path).map_err(|_| "archive accounting failed")?,
            ));
            if !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.len() == 64 && n.bytes().all(|b| b.is_ascii_hexdigit()))
            {
                return Err("unowned archive entry".into());
            }
            for job in fs::read_dir(path).map_err(|_| "archive accounting failed")? {
                inspected += 1;
                if inspected > 4096 {
                    return Err("archive accounting bound exceeded".into());
                }
                let job = job.map_err(|_| "archive accounting failed")?.path();
                owned_dir(&job)?;
                if !job
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.len() == 64 && n.bytes().all(|b| b.is_ascii_hexdigit()))
                {
                    return Err("unowned archive entry".into());
                }
                let mut bytes = allocated_bytes(
                    &fs::symlink_metadata(&job).map_err(|_| "archive accounting failed")?,
                );
                let mut reservation = false;
                for file in fs::read_dir(&job).map_err(|_| "archive accounting failed")? {
                    inspected += 1;
                    if inspected > 4096 {
                        return Err("archive accounting bound exceeded".into());
                    }
                    let path = file.map_err(|_| "archive accounting failed")?.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .ok_or("unowned archive entry")?;
                    if !matches!(
                        name,
                        "stdout" | "stderr" | "reservation" | "terminal.json" | "terminal.tmp"
                    ) {
                        return Err("unowned archive entry".into());
                    }
                    let f = private_file(&path, false)?;
                    bytes = bytes.saturating_add(allocated_bytes(
                        &f.metadata().map_err(|_| "archive accounting failed")?,
                    ));
                    reservation |= name == "reservation";
                }
                if reservation {
                    bytes = bytes.max(RESERVATION_BYTES);
                } else if job.join(RECEIPT).exists() {
                    bytes = bytes.saturating_add(SERVER_RECEIPT_OVERHEAD_BYTES);
                    if let Ok(record) = read_descriptor(&job.join(RECEIPT)) {
                        if digest(record.job_id.as_bytes())
                            != job.file_name().unwrap().to_string_lossy()
                        {
                            return Err("archive identity mismatch".into());
                        }
                        terminal.push((record.committed_at, job.clone(), bytes));
                    } else {
                        return Err("invalid terminal archive".into());
                    }
                }
                used = used.saturating_add(bytes);
            }
        }
        terminal.sort_by_key(|entry| entry.0);
        let mut count = terminal.len();
        for (ended, path, bytes) in terminal {
            if ended.saturating_add(ARCHIVE_RETENTION_SECS) <= now
                || count > ARCHIVE_MAX_TERMINAL
                || used.saturating_add(extra) > ARCHIVE_TOTAL_MAX_BYTES
            {
                // Exact known files only; never recursive removal of an unresolved target.
                for name in ["stdout", "stderr", RECEIPT] {
                    let file = path.join(name);
                    if file.exists() {
                        private_file(&file, false)?;
                        fs::remove_file(file).map_err(|_| "archive cleanup failed")?;
                    }
                }
                fs::remove_dir(path).map_err(|_| "archive cleanup failed")?;
                used = used.saturating_sub(bytes);
                count -= 1;
            }
        }
        if used.saturating_add(extra) > ARCHIVE_TOTAL_MAX_BYTES {
            return Err("archive quota exceeded".into());
        }
        Ok(())
    }

    pub(crate) fn capture(
        &self,
        context: &ShellJobContext,
        registry: &Path,
        job: &str,
        request: &str,
        client: &str,
        instance: &str,
    ) -> Result<ArchiveCapture, String> {
        let id = context
            .runtime_project_id
            .as_deref()
            .and_then(|p| p.strip_prefix(&format!("agent:{client}:")))
            .ok_or("archive project unavailable")?;
        let project = super::projects::find_project_shell_context_by_id(registry, id)
            .ok_or("archive project unavailable")?;
        let root = PathBuf::from(&project.path)
            .canonicalize()
            .map_err(|_| "archive root unavailable")?;
        // project_cwd is the admitted Project-relative selection (commonly
        // "."), not its registered absolute root. The execution cwd has
        // already been matched to the native operation by context validation.
        // Neither can select a different root than the trusted Project id.
        let selection = context
            .project_cwd
            .as_deref()
            .ok_or("archive admitted root unavailable")?;
        let selection = root
            .join(selection)
            .canonicalize()
            .map_err(|_| "archive admitted root unavailable")?;
        let cwd = context
            .cwd
            .as_deref()
            .ok_or("archive admitted cwd unavailable")?;
        let cwd = Path::new(cwd)
            .canonicalize()
            .map_err(|_| "archive admitted cwd unavailable")?;
        if !webcodex_runner_config::paths::path_is_within(&selection, &root)
            || !webcodex_runner_config::paths::path_is_within(&cwd, &selection)
        {
            return Err("archive admitted root mismatch".into());
        }
        let descriptor = JobArchiveDescriptor {
            job_id: job.into(),
            request_id: request.into(),
            client_id: client.into(),
            runner_instance_id: instance.into(),
            project_id: context.runtime_project_id.clone().unwrap(),
            project_root_fingerprint: super::projects::project_root_fingerprint(&root),
            root_incarnation: incarnation(&root)?,
            committed_at: 1,
            stdout: empty_stream(),
            stderr: empty_stream(),
        };
        descriptor.validate().map_err(str::to_string)?;
        let _lock = self.lock()?;
        self.account_and_prune(chrono::Utc::now().timestamp(), RESERVATION_BYTES)?;
        secure_dir(&self.root.join(&self.namespace))?;
        let dir = self.job_dir(job);
        if dir.exists() {
            return Err("archive identity already exists".into());
        }
        secure_dir(&dir)?;
        let reservation = private_file(&dir.join("reservation"), true)?;
        reservation
            .sync_all()
            .map_err(|_| "archive reservation failed")?;
        let stdout = private_file(&dir.join("stdout"), true)?;
        let stderr = private_file(&dir.join("stderr"), true)?;
        let (tx, rx) = mpsc::sync_channel(CAPTURE_QUEUE_CHUNKS);
        let loss = Arc::new(Mutex::new(None));
        let worker_loss = loss.clone();
        let store = self.clone();
        std::thread::Builder::new()
            .name("webcodex-job-archive".into())
            .spawn(move || {
                archive_writer(store, dir, descriptor, stdout, stderr, rx, worker_loss);
            })
            .map_err(|_| "archive writer unavailable")?;
        Ok(ArchiveCapture { tx, loss })
    }

    pub(crate) fn read(
        &self,
        read: &JobArchiveRead,
        registry: &Path,
    ) -> Result<JobArchivePage, String> {
        read.archive.validate().map_err(str::to_string)?;
        if read
            .archive
            .committed_at
            .saturating_add(ARCHIVE_RETENTION_SECS)
            <= chrono::Utc::now().timestamp()
        {
            return Err("archive expired".into());
        }
        let id = read
            .archive
            .project_id
            .strip_prefix(&format!("agent:{}:", read.archive.client_id))
            .ok_or("archive identity mismatch")?;
        let project = super::projects::find_project_shell_context_by_id(registry, id)
            .ok_or("archive project unavailable")?;
        let canonical = Path::new(&project.path)
            .canonicalize()
            .map_err(|_| "archive project unavailable")?;
        if super::projects::project_root_fingerprint(&canonical)
            != read.archive.project_root_fingerprint
            || incarnation(Path::new(&project.path))? != read.archive.root_incarnation
        {
            return Err("archive project incarnation changed".into());
        }
        let _lock = self.lock()?;
        let dir = self.job_dir(&read.archive.job_id);
        owned_dir(&self.root.join(&self.namespace))?;
        owned_dir(&dir)?;
        if read_descriptor(&dir.join(RECEIPT))? != read.archive {
            return Err("archive identity mismatch".into());
        }
        let (stdout, next_stdout_line) = read_stream(
            &dir.join("stdout"),
            read.since_stdout_line,
            read.tail_lines,
            &read.archive.stdout,
        )?;
        let (stderr, next_stderr_line) = read_stream(
            &dir.join("stderr"),
            read.since_stderr_line,
            read.tail_lines,
            &read.archive.stderr,
        )?;
        if incarnation(Path::new(&project.path))? != read.archive.root_incarnation {
            return Err("archive project incarnation changed".into());
        }
        let current = super::projects::find_project_shell_context_by_id(registry, id)
            .ok_or("archive project unavailable")?;
        if current.path != project.path {
            return Err("archive project identity changed".into());
        }
        Ok(JobArchivePage {
            archive: read.archive.clone(),
            stdout,
            stderr,
            next_stdout_line,
            next_stderr_line,
        })
    }

    pub(crate) fn read_verified(
        &self,
        read: &JobArchiveRead,
        registry: &Path,
    ) -> Result<JobArchiveReadResult, String> {
        let root = verified_project_root(read, registry)?;
        let result = self.read(read, registry);
        // Always recheck, including storage-error paths. Timeout or lost proof
        // never authorizes metadata disclosure at the Server.
        let current = verified_project_root(read, registry)?;
        if current != root {
            return Err("archive project identity changed".into());
        }
        match result {
            Ok(page) => Ok(JobArchiveReadResult::Available { page }),
            Err(error)
                if matches!(
                    error.as_str(),
                    "archive project incarnation changed"
                        | "archive project unavailable"
                        | "archive project identity changed"
                        | "archive root unavailable"
                        | "archive identity mismatch"
                ) =>
            {
                Err(error)
            }
            Err(error) => {
                let reason = match error.as_str() {
                    "archive line exceeds response bound" => {
                        JobArchiveUnavailableReason::LineExceedsPageBound
                    }
                    "archive expired" => JobArchiveUnavailableReason::Expired,
                    _ => JobArchiveUnavailableReason::StorageUnavailable,
                };
                Ok(JobArchiveReadResult::Unavailable {
                    archive: read.archive.clone(),
                    reason,
                })
            }
        }
    }
}

fn verified_project_root(read: &JobArchiveRead, registry: &Path) -> Result<PathBuf, String> {
    read.archive.validate().map_err(str::to_string)?;
    let id = read
        .archive
        .project_id
        .strip_prefix(&format!("agent:{}:", read.archive.client_id))
        .ok_or("archive identity mismatch")?;
    let project = super::projects::find_project_shell_context_by_id(registry, id)
        .ok_or("archive project unavailable")?;
    let root = Path::new(&project.path)
        .canonicalize()
        .map_err(|_| "archive project unavailable")?;
    if super::projects::project_root_fingerprint(&root) != read.archive.project_root_fingerprint
        || incarnation(&root)? != read.archive.root_incarnation
    {
        return Err("archive project incarnation changed".into());
    }
    Ok(root)
}

fn empty_stream() -> JobArchiveStream {
    JobArchiveStream {
        retained_bytes: 0,
        next_line: 1,
        loss_reason: None,
    }
}

impl ArchiveCapture {
    #[cfg(test)]
    pub(crate) fn delayed_finish_for_test() -> (Self, mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (tx, rx) = mpsc::sync_channel(CAPTURE_QUEUE_CHUNKS);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                if let Message::Finish(reply) = message {
                    let _ = started_tx.send(());
                    let _ = release_rx.recv_timeout(Duration::from_secs(3));
                    let _ = reply.send(None);
                    break;
                }
            }
        });
        (
            Self {
                tx,
                loss: Arc::new(Mutex::new(None)),
            },
            started_rx,
            release_tx,
        )
    }

    pub(crate) fn append(&self, stdout: bool, chunk: Option<&str>) {
        let Some(chunk) = chunk.filter(|s| !s.is_empty()) else {
            return;
        };
        let mut loss = self.loss.lock().unwrap();
        if loss.is_some() {
            return;
        }
        let mut remaining = chunk;
        // Native updates can coalesce several pipe reads into one large delta.
        // Preserve its prefix, but bound work under the Jobs mutex to at most
        // one queue's allowance. Never wait for disk or queue capacity here.
        for _ in 0..CAPTURE_QUEUE_CHUNKS {
            if remaining.is_empty() {
                return;
            }
            let mut end = remaining.len().min(MAX_CHUNK);
            while !remaining.is_char_boundary(end) {
                end -= 1;
            }
            if self
                .tx
                .try_send(Message::Chunk(stdout, remaining[..end].to_string()))
                .is_err()
            {
                *loss = Some("backpressure".into());
                return;
            }
            remaining = &remaining[end..];
        }
        if !remaining.is_empty() {
            *loss = Some("backpressure".into());
        }
    }
    pub(crate) fn finish(&self) -> Option<JobArchiveDescriptor> {
        let (tx, rx) = mpsc::channel();
        // Finishing occurs after child pipes are drained. This bounded wait never changes child exit truth.
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        let mut message = Message::Finish(tx);
        loop {
            match self.tx.try_send(message) {
                Ok(()) => break,
                Err(mpsc::TrySendError::Full(next)) if std::time::Instant::now() < deadline => {
                    message = next;
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => return None,
            }
        }
        rx.recv_timeout(Duration::from_secs(1)).ok().flatten()
    }
}

fn archive_writer(
    store: ArchiveStore,
    dir: PathBuf,
    mut descriptor: JobArchiveDescriptor,
    mut stdout: File,
    mut stderr: File,
    rx: mpsc::Receiver<Message>,
    loss: Arc<Mutex<Option<String>>>,
) {
    let mut newline = [0usize; 2];
    let mut last_newline = [true; 2];
    while let Ok(message) = rx.recv() {
        match message {
            Message::Chunk(out, chunk) => {
                let index = usize::from(!out);
                let (file, stream) = if out {
                    (&mut stdout, &mut descriptor.stdout)
                } else {
                    (&mut stderr, &mut descriptor.stderr)
                };
                if stream.loss_reason.is_some() {
                    continue;
                }
                let mut count = (ARCHIVE_STREAM_MAX_BYTES - stream.retained_bytes) as usize;
                count = count.min(chunk.len());
                while !chunk.is_char_boundary(count) {
                    count -= 1;
                }
                if file.write_all(chunk[..count].as_bytes()).is_err() {
                    stream.loss_reason = Some("storage_failure".into());
                    continue;
                }
                stream.retained_bytes += count as u64;
                newline[index] += chunk[..count].bytes().filter(|b| *b == b'\n').count();
                if count > 0 {
                    last_newline[index] = chunk.as_bytes()[count - 1] == b'\n';
                }
                stream.next_line = 1 + newline[index] + usize::from(!last_newline[index]);
                if count < chunk.len() {
                    stream.loss_reason = Some("stream_limit".into());
                }
            }
            Message::Finish(reply) => {
                if let Some(reason) = loss.lock().unwrap().clone() {
                    descriptor.stdout.loss_reason.get_or_insert(reason.clone());
                    descriptor.stderr.loss_reason.get_or_insert(reason);
                }
                descriptor.committed_at = chrono::Utc::now().timestamp();
                let committed = (|| -> Result<(), String> {
                    stdout.sync_all().map_err(|_| "archive sync failed")?;
                    stderr.sync_all().map_err(|_| "archive sync failed")?;
                    let _lock = store.lock()?;
                    let bytes =
                        serde_json::to_vec(&descriptor).map_err(|_| "archive receipt failed")?;
                    if bytes.len() > ARCHIVE_METADATA_MAX_BYTES {
                        return Err("archive receipt oversized".into());
                    }
                    let mut temp = private_file(&dir.join("terminal.tmp"), true)?;
                    temp.write_all(&bytes)
                        .map_err(|_| "archive receipt failed")?;
                    temp.sync_all().map_err(|_| "archive receipt failed")?;
                    fs::rename(dir.join("terminal.tmp"), dir.join(RECEIPT))
                        .map_err(|_| "archive receipt failed")?;
                    File::open(&dir)
                        .and_then(|f| f.sync_all())
                        .map_err(|_| "archive directory sync failed")?;
                    fs::remove_file(dir.join("reservation"))
                        .map_err(|_| "archive reservation failed")?;
                    store.account_and_prune(descriptor.committed_at, 0)?;
                    Ok(())
                })()
                .is_ok();
                let _ = reply.send(committed.then_some(descriptor));
                return;
            }
        }
    }
    // No terminal commit on owner loss. Partial bytes never establish process survival.
}

fn read_descriptor(path: &Path) -> Result<JobArchiveDescriptor, String> {
    let file = private_file(path, false)?;
    if file
        .metadata()
        .map_err(|_| "archive receipt unavailable")?
        .len()
        > ARCHIVE_METADATA_MAX_BYTES as u64
    {
        return Err("archive receipt oversized".into());
    }
    let descriptor: JobArchiveDescriptor =
        serde_json::from_reader(file).map_err(|_| "archive receipt invalid")?;
    descriptor.validate().map_err(str::to_string)?;
    Ok(descriptor)
}

fn read_stream(
    path: &Path,
    since: Option<usize>,
    tail: Option<usize>,
    stream: &JobArchiveStream,
) -> Result<(String, usize), String> {
    let file = private_file(path, false)?;
    if file
        .metadata()
        .map_err(|_| "archive stream unavailable")?
        .len()
        != stream.retained_bytes
    {
        return Err("archive stream identity mismatch".into());
    }
    let start = since.map(|n| n.max(1)).unwrap_or_else(|| {
        tail.map(|n| stream.next_line.saturating_sub(n.min(2000)).max(1))
            .unwrap_or(1)
    });
    let mut reader = BufReader::new(file.take(ARCHIVE_STREAM_MAX_BYTES));
    let mut output = String::new();
    let mut line = 1usize;
    let mut next = start.min(stream.next_line);
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        // Bound every individual allocation even for a single enormous line.
        let read = (&mut reader)
            .take(ARCHIVE_READ_MAX_BYTES as u64 + 1)
            .read_until(b'\n', &mut bytes)
            .map_err(|_| "archive stream unavailable")?;
        if read == 0 {
            break;
        }
        if read > ARCHIVE_READ_MAX_BYTES {
            return Err("archive line exceeds response bound".into());
        }
        if line >= start {
            if output.len() + read > ARCHIVE_READ_MAX_BYTES {
                break;
            }
            output.push_str(std::str::from_utf8(&bytes).map_err(|_| "archive decoding invalid")?);
            next = line + 1;
            if output.lines().count() >= tail.unwrap_or(2000).min(2000) {
                break;
            }
        }
        line += 1;
    }
    Ok((output, next))
}

#[cfg(all(test, unix))]
pub(crate) mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, ArchiveStore, PathBuf, ShellJobContext) {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        fs::create_dir(&project).unwrap();
        let registry = temp.path().join("projects");
        fs::create_dir(&registry).unwrap();
        fs::write(
            registry.join("p.toml"),
            format!("id = \"p\"\npath = {:?}\n", project.to_str().unwrap()),
        )
        .unwrap();
        let mut context = super::super::job_manager::test_job_context(&project, Vec::new());
        context.runtime_project_id = Some("agent:client:p".into());
        context.project_cwd = Some(".".into());
        let store = ArchiveStore {
            root: temp.path().join("archives"),
            namespace: digest(b"test-endpoint-client"),
        };
        (temp, store, registry, context)
    }

    pub(crate) struct WriterFixture {
        pub(crate) temp: tempfile::TempDir,
        pub(crate) store: ArchiveStore,
        pub(crate) registry: PathBuf,
        pub(crate) context: ShellJobContext,
        pub(crate) capture: ArchiveCapture,
        pub(crate) worker: std::thread::JoinHandle<()>,
        pub(crate) descriptor: JobArchiveDescriptor,
    }

    fn writer_fixture(job: &str, reject_stdout_write: bool) -> WriterFixture {
        let (temp, store, registry, context) = fixture();
        // Use the existing capture owner to mint the Project/incarnation
        // descriptor; only the isolated writer's file handle is varied below.
        let seed = store
            .capture(
                &context,
                &registry,
                "seed",
                "request-seed",
                "client",
                "instance",
            )
            .unwrap();
        let mut descriptor = seed.finish().unwrap();
        descriptor.job_id = job.into();
        descriptor.request_id = format!("request-{job}");
        let dir = store.job_dir(job);
        secure_dir(&dir).unwrap();
        private_file(&dir.join("reservation"), true).unwrap();
        let stdout = private_file(&dir.join("stdout"), true).unwrap();
        let stdout = if reject_stdout_write {
            drop(stdout);
            private_file(&dir.join("stdout"), false).unwrap()
        } else {
            stdout
        };
        if reject_stdout_write {
            let mut probe = stdout.try_clone().unwrap();
            assert_eq!(
                probe.write_all(b"cannot write").unwrap_err().raw_os_error(),
                Some(libc::EBADF)
            );
            stdout.sync_all().unwrap();
        }
        let stderr = private_file(&dir.join("stderr"), true).unwrap();
        let (tx, rx) = mpsc::sync_channel(CAPTURE_QUEUE_CHUNKS);
        let loss = Arc::new(Mutex::new(None));
        let capture = ArchiveCapture {
            tx,
            loss: loss.clone(),
        };
        let worker_store = store.clone();
        let worker_descriptor = descriptor.clone();
        let worker = std::thread::spawn(move || {
            archive_writer(
                worker_store,
                dir,
                worker_descriptor,
                stdout,
                stderr,
                rx,
                loss,
            )
        });
        WriterFixture {
            temp,
            store,
            registry,
            context,
            capture,
            worker,
            descriptor,
        }
    }

    pub(crate) fn rejected_stream_writer_fixture() -> WriterFixture {
        writer_fixture("rejected-write", true)
    }

    #[test]
    fn archive_owner_loss_and_invalid_temp_receipt_never_publish_terminal_proof() {
        let owner_lost = writer_fixture("owner-lost", false);
        owner_lost
            .capture
            .append(true, Some("partial bytes before owner loss\n"));
        drop(owner_lost.capture);
        owner_lost.worker.join().unwrap();
        let dir = owner_lost.store.job_dir("owner-lost");
        assert_eq!(
            fs::read_to_string(dir.join("stdout")).unwrap(),
            "partial bytes before owner loss\n"
        );
        assert!(!dir.join(RECEIPT).exists());
        assert!(dir.join("reservation").exists());
        assert!(
            read_descriptor(&dir.join(RECEIPT)).is_err(),
            "drained bytes without Finish do not mint a terminal receipt"
        );

        let poison = writer_fixture("poison-temp", false);
        let dir = poison.store.job_dir("poison-temp");
        let mut temp_receipt = private_file(&dir.join("terminal.tmp"), true).unwrap();
        temp_receipt
            .write_all(b"incomplete pre-commit receipt")
            .unwrap();
        temp_receipt.sync_all().unwrap();
        drop(temp_receipt);
        poison
            .capture
            .append(true, Some("drained but uncommitted\n"));
        assert!(poison.capture.finish().is_none());
        poison.worker.join().unwrap();
        assert!(
            !dir.join(RECEIPT).exists(),
            "failed create-new temp admission cannot publish partial terminal metadata"
        );
        assert_eq!(
            fs::read(dir.join("terminal.tmp")).unwrap(),
            b"incomplete pre-commit receipt"
        );
        assert!(dir.join("reservation").exists());
        let mut invalid = private_file(&dir.join(RECEIPT), true).unwrap();
        invalid.write_all(b"{invalid final receipt").unwrap();
        invalid.sync_all().unwrap();
        assert!(read_descriptor(&dir.join(RECEIPT)).is_err());
        assert!(
            matches!(
                poison
                    .store
                    .read_verified(
                        &JobArchiveRead {
                            archive: poison.descriptor,
                            since_stdout_line: Some(1),
                            since_stderr_line: Some(1),
                            tail_lines: None
                        },
                        &poison.registry
                    )
                    .unwrap(),
                JobArchiveReadResult::Unavailable {
                    reason: JobArchiveUnavailableReason::StorageUnavailable,
                    ..
                }
            ),
            "malformed committed-path bytes never become available output evidence"
        );
    }

    #[test]
    fn archive_rejected_stream_write_retains_exact_storage_failure_reason() {
        let rejected = rejected_stream_writer_fixture();
        rejected
            .capture
            .append(true, Some("native stdout that the real OS rejects\n"));
        rejected
            .capture
            .append(false, Some("independent stderr survives\n"));
        let descriptor = rejected.capture.finish().unwrap();
        rejected.worker.join().unwrap();
        assert_eq!(descriptor.stdout.retained_bytes, 0);
        assert_eq!(descriptor.stdout.next_line, 1);
        assert_eq!(
            descriptor.stdout.loss_reason.as_deref(),
            Some("storage_failure")
        );
        assert!(descriptor.stderr.loss_reason.is_none());
        let page = rejected
            .store
            .read(
                &JobArchiveRead {
                    archive: descriptor,
                    since_stdout_line: Some(1),
                    since_stderr_line: Some(1),
                    tail_lines: None,
                },
                &rejected.registry,
            )
            .unwrap();
        assert_eq!(page.stdout, "");
        assert_eq!(page.stderr, "independent stderr survives\n");
        assert_eq!(
            page.archive.stdout.loss_reason.as_deref(),
            Some("storage_failure"),
            "empty rejected evidence must never be described as complete"
        );
    }

    #[test]
    fn archive_prefix_pages_survive_new_owner_and_preserve_exact_unicode() {
        let (_temp, store, registry, context) = fixture();
        let capture = store
            .capture(
                &context,
                &registry,
                "job",
                "request",
                "client",
                "instance-1",
            )
            .unwrap();
        let mut expected = String::new();
        for group in 0..6 {
            let chunk: String = (group * 1000..(group + 1) * 1000)
                .map(|n| format!("{n:05} exact █\n"))
                .collect();
            expected.push_str(&chunk);
            capture.append(true, Some(&chunk));
        }
        assert!(expected.len() > 64 * 1024);
        let descriptor = capture.finish().unwrap();
        assert!(descriptor.stdout.loss_reason.is_none());
        let restarted = store.clone();
        let mut cursor = 1;
        let mut actual = String::new();
        loop {
            let page = restarted
                .read(
                    &JobArchiveRead {
                        archive: descriptor.clone(),
                        since_stdout_line: Some(cursor),
                        since_stderr_line: Some(1),
                        tail_lines: Some(200),
                    },
                    &registry,
                )
                .unwrap();
            if page.stdout.is_empty() {
                break;
            }
            assert!(page.stdout.len() <= ARCHIVE_READ_MAX_BYTES);
            assert!(page.next_stdout_line > cursor);
            actual.push_str(&page.stdout);
            cursor = page.next_stdout_line;
        }
        assert_eq!(actual, expected);
        assert_eq!(cursor, descriptor.stdout.next_line);
        assert_eq!(descriptor.runner_instance_id, "instance-1");
    }

    #[test]
    fn archive_symlink_and_replaced_project_are_rejected_without_retargting() {
        let (temp, store, registry, context) = fixture();
        let capture = store
            .capture(&context, &registry, "job", "request", "client", "instance")
            .unwrap();
        capture.append(true, Some("private exact evidence\n"));
        let descriptor = capture.finish().unwrap();
        let read = JobArchiveRead {
            archive: descriptor,
            since_stdout_line: Some(1),
            since_stderr_line: None,
            tail_lines: None,
        };
        let project = PathBuf::from(context.cwd.unwrap());
        fs::rename(&project, temp.path().join("original-project")).unwrap();
        fs::create_dir(&project).unwrap();
        assert!(
            store
                .read_verified(&read, &registry)
                .unwrap_err()
                .contains("incarnation"),
            "unproven root cannot emit even unavailable archive metadata"
        );
        fs::remove_dir(&project).unwrap();
        fs::rename(temp.path().join("original-project"), &project).unwrap();
        let stdout = store.job_dir("job").join("stdout");
        fs::remove_file(&stdout).unwrap();
        std::os::unix::fs::symlink(temp.path().join("unrelated"), &stdout).unwrap();
        assert!(store.read(&read, &registry).is_err());
        assert!(
            matches!(
                store.read_verified(&read, &registry).unwrap(),
                JobArchiveReadResult::Unavailable {
                    reason: JobArchiveUnavailableReason::StorageUnavailable,
                    ..
                }
            ),
            "proved unchanged root can report missing evidence without changing child truth"
        );
    }

    #[test]
    fn archive_coalesced_output_keeps_prefix_and_full_queue_is_explicit_nonblocking_loss() {
        let (_temp, store, registry, context) = fixture();
        let capture = store
            .capture(&context, &registry, "job", "request", "client", "instance")
            .unwrap();
        let coalesced: String = (0..2200)
            .map(|n| format!("{n:05} exact UTF-8 █{}\n", "x".repeat(80)))
            .collect();
        assert!(coalesced.len() > 200 * 1024);
        capture.append(true, Some(&coalesced));
        let descriptor = capture.finish().unwrap();
        assert_eq!(descriptor.stdout.retained_bytes, coalesced.len() as u64);
        assert_eq!(descriptor.stdout.next_line, 2201);
        assert!(descriptor.stdout.loss_reason.is_none());
        assert_eq!(
            fs::read_to_string(store.job_dir("job").join("stdout")).unwrap(),
            coalesced
        );
        // No receiver drains this queue until the bounded append deadline.
        // Disconnect on failure so a nearest-wrong blocking producer settles.
        let (tx, rx) = mpsc::sync_channel(1);
        let full = ArchiveCapture {
            tx,
            loss: Arc::new(Mutex::new(None)),
        };
        let (done_tx, done_rx) = mpsc::channel();
        let blocked_chunk = coalesced.clone();
        let producer = std::thread::spawn(move || {
            full.append(true, Some(&blocked_chunk));
            let _ = done_tx.send(full);
        });
        let full = match done_rx.recv_timeout(Duration::from_millis(500)) {
            Ok(full) => full,
            Err(error) => {
                drop(rx);
                let _ = producer.join();
                panic!("archive append blocked pipe consumption: {error}");
            }
        };
        producer.join().unwrap();
        assert_eq!(full.loss.lock().unwrap().as_deref(), Some("backpressure"));
        let Message::Chunk(true, prefix) = rx.try_recv().unwrap() else {
            panic!("prefix must be queued before loss");
        };
        assert!(prefix.len() <= MAX_CHUNK);
        assert!(coalesced.starts_with(&prefix));
        full.append(true, Some("must not skip lost bytes\n"));
        assert!(
            rx.try_recv().is_err(),
            "prefix retention stops permanently after its first gap"
        );
        let (tx, rx) = mpsc::sync_channel(CAPTURE_QUEUE_CHUNKS * 2);
        let bounded = ArchiveCapture {
            tx,
            loss: Arc::new(Mutex::new(None)),
        };
        bounded.append(
            true,
            Some(&"x".repeat(MAX_CHUNK * (CAPTURE_QUEUE_CHUNKS + 1))),
        );
        assert_eq!(
            rx.try_iter().count(),
            CAPTURE_QUEUE_CHUNKS,
            "even an available queue cannot exceed one bounded append allowance"
        );
        assert_eq!(
            bounded.loss.lock().unwrap().as_deref(),
            Some("backpressure")
        );
        let capture = store
            .capture(
                &context,
                &registry,
                "active",
                "active-request",
                "client",
                "instance",
            )
            .unwrap();
        capture.append(true, Some("partial\n"));
        drop(capture);
        assert!(!store.job_dir("active").join(RECEIPT).exists());
        assert!(store.job_dir("active").join("reservation").exists());
    }

    #[test]
    fn archive_stream_cap_and_single_line_page_failure_are_explicit() {
        let (_temp, store, registry, context) = fixture();
        let capture = store
            .capture(&context, &registry, "job", "request", "client", "instance")
            .unwrap();
        let chunk = "x".repeat(MAX_CHUNK);
        // A blocking producer is confined to this writer-unit fixture so the
        // stream-cap discriminator is deterministic, independent of queue load.
        for _ in 0..(ARCHIVE_STREAM_MAX_BYTES as usize / MAX_CHUNK + 1) {
            capture
                .tx
                .send(Message::Chunk(true, chunk.clone()))
                .unwrap();
        }
        let descriptor = capture.finish().unwrap();
        assert_eq!(descriptor.stdout.retained_bytes, ARCHIVE_STREAM_MAX_BYTES);
        assert_eq!(
            descriptor.stdout.loss_reason.as_deref(),
            Some("stream_limit")
        );
        let read = JobArchiveRead {
            archive: descriptor,
            since_stdout_line: Some(1),
            since_stderr_line: None,
            tail_lines: Some(200),
        };
        let error = store.read(&read, &registry).unwrap_err();
        assert_eq!(error, "archive line exceeds response bound");
        assert!(matches!(
            store.read_verified(&read, &registry).unwrap(),
            JobArchiveReadResult::Unavailable {
                reason: JobArchiveUnavailableReason::LineExceedsPageBound,
                ..
            }
        ));
    }

    #[test]
    fn archive_eviction_read_does_not_recreate_empty_owned_directories() {
        let (_temp, store, registry, context) = fixture();
        let capture = store
            .capture(&context, &registry, "job", "request", "client", "instance")
            .unwrap();
        let descriptor = capture.finish().unwrap();
        let dir = store.job_dir("job");
        for name in ["stdout", "stderr", RECEIPT] {
            fs::remove_file(dir.join(name)).unwrap();
        }
        fs::remove_dir(&dir).unwrap();
        let result = store
            .read_verified(
                &JobArchiveRead {
                    archive: descriptor,
                    since_stdout_line: Some(1),
                    since_stderr_line: None,
                    tail_lines: None,
                },
                &registry,
            )
            .unwrap();
        assert!(matches!(
            result,
            JobArchiveReadResult::Unavailable {
                reason: JobArchiveUnavailableReason::StorageUnavailable,
                ..
            }
        ));
        assert!(
            !dir.exists(),
            "historical readers cannot grow the owned namespace or reset retention"
        );
    }

    #[test]
    fn archive_accounting_bounds_empty_namespaces_as_well_as_files() {
        let (_temp, store, _registry, _context) = fixture();
        let _lock = store.lock().unwrap();
        for n in 0..4097 {
            secure_dir(&store.root.join(digest(format!("namespace-{n}").as_bytes()))).unwrap();
        }
        assert_eq!(
            store
                .account_and_prune(chrono::Utc::now().timestamp(), 0)
                .unwrap_err(),
            "archive accounting bound exceeded"
        );
        assert_eq!(
            fs::read_dir(&store.root).unwrap().count(),
            4098,
            "bounded accounting must not prune active or empty unrelated namespaces"
        );
    }

    #[test]
    fn archive_quota_counts_active_reservations_in_other_namespaces_without_deleting_them() {
        let (temp, store, _registry, _context) = fixture();
        let _lock = store.lock().unwrap();
        for namespace in 0..4 {
            let ns = store
                .root
                .join(digest(format!("namespace-{namespace}").as_bytes()));
            secure_dir(&ns).unwrap();
            for job in 0..8 {
                let dir = ns.join(digest(format!("active-{job}").as_bytes()));
                secure_dir(&dir).unwrap();
                private_file(&dir.join("reservation"), true).unwrap();
            }
        }
        fs::write(temp.path().join("unrelated-state"), b"preserve").unwrap();
        assert_eq!(
            store
                .account_and_prune(chrono::Utc::now().timestamp(), 0)
                .unwrap_err(),
            "archive quota exceeded"
        );
        assert_eq!(
            fs::read(temp.path().join("unrelated-state")).unwrap(),
            b"preserve"
        );
        let count: usize = fs::read_dir(&store.root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| fs::read_dir(e.path()).unwrap().count())
            .sum();
        assert_eq!(count, 32, "quota cannot evict any active reservation");
    }

    #[test]
    fn archive_root_symlink_cannot_redirect_creation_into_unrelated_state() {
        let (temp, mut store, registry, context) = fixture();
        let unrelated = temp.path().join("unrelated");
        fs::create_dir(&unrelated).unwrap();
        fs::write(unrelated.join("sentinel"), b"preserve").unwrap();
        store.root = temp.path().join("linked-archive");
        std::os::unix::fs::symlink(&unrelated, &store.root).unwrap();
        assert!(store
            .capture(&context, &registry, "job", "request", "client", "instance")
            .is_err());
        assert_eq!(fs::read_dir(&unrelated).unwrap().count(), 1);
    }

    #[test]
    fn archive_relative_project_selection_cannot_retarget_another_root() {
        let (temp, store, registry, mut context) = fixture();
        let unrelated = temp.path().join("unrelated");
        fs::create_dir(&unrelated).unwrap();
        context.project_cwd = Some("../unrelated".into());
        context.cwd = Some(unrelated.to_string_lossy().into_owned());
        assert_eq!(
            store
                .capture(&context, &registry, "job", "request", "client", "instance")
                .unwrap_err(),
            "archive admitted root mismatch"
        );
        context.project_cwd = Some(".".into());
        assert_eq!(
            store
                .capture(&context, &registry, "job", "request", "client", "instance")
                .unwrap_err(),
            "archive admitted root mismatch"
        );
        assert!(
            !store.root.exists(),
            "rejected admission cannot create output state in either root"
        );
    }
}
