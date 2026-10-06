//! Small same-execution archive contracts. Locators never contain filesystem paths.
use serde::{Deserialize, Serialize};

pub const ARCHIVE_STREAM_MAX_BYTES: u64 = 16 * 1024 * 1024;
pub const ARCHIVE_TOTAL_MAX_BYTES: u64 = 1024 * 1024 * 1024;
pub const ARCHIVE_MAX_TERMINAL: usize = 256;
pub const ARCHIVE_RETENTION_SECS: i64 = 7 * 24 * 60 * 60;
pub const ARCHIVE_METADATA_MAX_BYTES: usize = 16 * 1024;
pub const ARCHIVE_READ_MAX_BYTES: usize = 64 * 1024;

#[derive(Debug, Serialize)]
pub struct ArchivedTerminalFact {
    pub job_id: String,
    pub status: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobArchiveStream {
    pub retained_bytes: u64,
    pub next_line: usize,
    pub loss_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobArchiveDescriptor {
    pub job_id: String,
    pub request_id: String,
    pub client_id: String,
    pub runner_instance_id: String,
    pub project_id: String,
    pub project_root_fingerprint: String,
    pub root_incarnation: String,
    pub committed_at: i64,
    pub stdout: JobArchiveStream,
    pub stderr: JobArchiveStream,
}

impl JobArchiveDescriptor {
    pub fn validate(&self) -> Result<(), &'static str> {
        for id in [
            &self.job_id,
            &self.request_id,
            &self.client_id,
            &self.runner_instance_id,
            &self.project_id,
        ] {
            if id.is_empty() || id.len() > 128 || id.chars().any(char::is_control) {
                return Err("invalid archive identity");
            }
        }
        if self.committed_at <= 0
            || !self.project_root_fingerprint.starts_with("wc_projroot_")
            || self.project_root_fingerprint.len() != 76
            || self.root_incarnation.len() != 64
            || !self.root_incarnation.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("invalid archive root identity");
        }
        for stream in [&self.stdout, &self.stderr] {
            if stream.retained_bytes > ARCHIVE_STREAM_MAX_BYTES
                || stream.next_line == 0
                || stream.loss_reason.as_deref().is_some_and(|r| {
                    !matches!(
                        r,
                        "stream_limit" | "quota" | "backpressure" | "storage_failure"
                    )
                })
            {
                return Err("invalid archive stream bounds");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobArchiveRead {
    pub archive: JobArchiveDescriptor,
    pub since_stdout_line: Option<usize>,
    pub since_stderr_line: Option<usize>,
    pub tail_lines: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobArchivePage {
    pub archive: JobArchiveDescriptor,
    pub stdout: String,
    pub stderr: String,
    pub next_stdout_line: usize,
    pub next_stderr_line: usize,
}

/// Unavailable output is emitted only after the Runner has revalidated the
/// admitted Project/root before and after the attempted read. Other failures
/// supply no archive metadata proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobArchiveReadResult {
    Available {
        page: JobArchivePage,
    },
    Unavailable {
        archive: JobArchiveDescriptor,
        reason: JobArchiveUnavailableReason,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobArchiveUnavailableReason {
    LineExceedsPageBound,
    Expired,
    StorageUnavailable,
}
