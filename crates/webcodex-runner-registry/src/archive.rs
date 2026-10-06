//! Exact-id historical evidence: never restores inventory or grants process control.
use crate::access_control::{assert_runner_access, job_visible_to_access};
use crate::jobs::job_view;
use crate::requests::{enqueue_pending_request_locked, next_request_id, notify_runner_locked};
use crate::{JobLogWait, JobLogWaitOutcome, RunnerAccess, RunnerRegistry, ShellJobLogObservation};
use tokio::sync::oneshot;
use webcodex_core::job_archive::{
    JobArchiveRead, JobArchiveReadResult, JobArchiveUnavailableReason,
};
use webcodex_core::runner_job_receipt::ArchivedJobReceipt;
use webcodex_core::runner_operation::{RunnerInvocationMetadata, RunnerOperation};
use webcodex_core::runner_protocol::{RunnerRequest, ShellJobInfo};

pub(crate) type ArchiveLog = (
    ShellJobInfo,
    Option<String>,
    Option<String>,
    usize,
    usize,
    ShellJobLogObservation,
);

pub use webcodex_core::job_archive::ArchivedTerminalFact;

impl RunnerRegistry {
    /// User-approved narrow fallback when current Project visibility cannot be
    /// established: only immutable terminal fact, never archive/root metadata.
    pub async fn archived_terminal_fact_for_auth(
        &self,
        auth: Option<&RunnerAccess>,
        job_id: &str,
    ) -> Result<Option<ArchivedTerminalFact>, String> {
        let Some(store) = self.inner.archive_store() else {
            return Ok(None);
        };
        let Some(value) = store.load_archive(job_id, crate::now_ts())? else {
            return Ok(None);
        };
        value.validate(crate::now_ts()).map_err(str::to_string)?;
        let inner = self.inner.lock().await;
        let job = archive_record(self, &value);
        if auth.is_none() || !job_visible_to_access(auth, &inner, &job) {
            return Err(format!("unknown shell job: {job_id}"));
        }
        if let Some(runner) = inner.runners.get(&value.receipt.client_id).filter(|r| {
            crate::now_ts().saturating_sub(r.last_seen) <= crate::RUNNER_ONLINE_WINDOW_SECS
        }) {
            assert_runner_access(auth, runner)?;
            let local_id = value
                .archive
                .project_id
                .strip_prefix(&format!("agent:{}:", value.archive.client_id))
                .ok_or("archive identity invalid")?;
            if !runner.projects.iter().any(|p| {
                !p.disabled
                    && p.id == local_id
                    && p.root_fingerprint.as_ref() == Some(&value.archive.project_root_fingerprint)
            }) {
                return Err(format!("unknown shell job: {job_id}"));
            }
            return Ok(None);
        }
        Ok(Some(ArchivedTerminalFact {
            job_id: value.receipt.snapshot.job_id,
            status: value.receipt.snapshot.status,
            exit_code: value.receipt.snapshot.exit_code,
        }))
    }
    pub(crate) async fn archive_log_for_auth(
        &self,
        auth: Option<&RunnerAccess>,
        job_id: &str,
        stdout: Option<usize>,
        stderr: Option<usize>,
        tail: Option<usize>,
        after: Option<&webcodex_core::job_observation::JobObservationToken>,
    ) -> Result<Option<ArchiveLog>, String> {
        let Some(store) = self.inner.archive_store() else {
            return Ok(None);
        };
        let Some(value) = store.load_archive(job_id, crate::now_ts())? else {
            return Ok(None);
        };
        value.validate(crate::now_ts()).map_err(str::to_string)?;
        let (job, rx) = {
            let mut inner = self.inner.lock().await;
            let job = archive_record(self, &value);
            if auth.is_none() || !job_visible_to_access(auth, &inner, &job) {
                return Err(format!("unknown shell job: {job_id}"));
            }
            let Some(runner) = inner.runners.get(&value.receipt.client_id) else {
                return Err("archive current Project visibility unavailable".into());
            };
            assert_runner_access(auth, runner)?;
            let local_id = value
                .archive
                .project_id
                .strip_prefix(&format!("agent:{}:", value.archive.client_id))
                .ok_or("archive project identity invalid")?;
            if !runner.projects.iter().any(|p| {
                !p.disabled
                    && p.id == local_id
                    && p.root_fingerprint.as_ref() == Some(&value.archive.project_root_fingerprint)
            }) {
                return Err("archive current Project identity unavailable".into());
            }
            let rx = if runner
                .runner_features
                .wire_capabilities()
                .job_output_archive
            {
                let request_id = next_request_id();
                let read = JobArchiveRead {
                    archive: value.archive.clone(),
                    since_stdout_line: stdout,
                    since_stderr_line: stderr,
                    tail_lines: tail,
                };
                let request = RunnerRequest::from_operation(
                    RunnerInvocationMetadata {
                        request_id: request_id.clone(),
                        client_id: value.receipt.client_id.clone(),
                        requested_by: "job_archive_reader".into(),
                        created_at: crate::now_ts(),
                    },
                    RunnerOperation::JobArchiveRead(read),
                )?;
                let (tx, rx) = oneshot::channel();
                if enqueue_pending_request_locked(
                    self.telemetry.as_ref(),
                    &mut inner,
                    &value.receipt.client_id,
                    request_id,
                    request,
                    Some(tx),
                    None,
                )
                .is_ok()
                {
                    notify_runner_locked(&inner, &value.receipt.client_id);
                    Some(rx)
                } else {
                    None
                }
            } else {
                None
            };
            (job, rx)
        };
        // No response is not root proof. In particular, a timeout, old Runner,
        // malformed reply or incarnation rejection must disclose no descriptor.
        let outcome = if let Some(rx) = rx {
            match tokio::time::timeout(std::time::Duration::from_secs(3), rx).await {
                Ok(Ok(response)) if response.success && response.exit_code == Some(0) => {
                    verified_outcome(response.stdout.as_deref(), &value.archive)
                }
                _ => None,
            }
        } else {
            None
        }
        .ok_or_else(|| format!("unknown shell job: {job_id}"))?;
        // Recheck visibility and current root after the asynchronous read, not
        // only before dispatch; a revocation cannot leak the returned page.
        {
            let inner = self.inner.lock().await;
            if !job_visible_to_access(auth, &inner, &job) {
                return Err(format!("unknown shell job: {job_id}"));
            }
            let runner = inner
                .runners
                .get(&value.receipt.client_id)
                .ok_or("archive current Project visibility unavailable")?;
            assert_runner_access(auth, runner)?;
            let id = value
                .archive
                .project_id
                .strip_prefix(&format!("agent:{}:", value.archive.client_id))
                .ok_or("archive identity invalid")?;
            if !runner.projects.iter().any(|p| {
                !p.disabled
                    && p.id == id
                    && p.root_fingerprint.as_ref() == Some(&value.archive.project_root_fingerprint)
            }) {
                return Err("archive current Project identity unavailable".into());
            }
        }
        let mut observation = ShellJobLogObservation::default();
        let reset = after.is_some() && stdout.is_none() && stderr.is_none();
        observation.wait = JobLogWait {
            wait_outcome: JobLogWaitOutcome::Terminal,
            changed: reset,
            terminal: true,
            ..Default::default()
        };
        if reset {
            // Historical recovery has no retained live cursor authority. Mark
            // the bounded baseline explicitly; never pretend it is a delta.
            observation.log_delta_status = webcodex_core::job_observation::JobLogDeltaStatus::Reset;
            observation.stdout_delta_reset = true;
            observation.stderr_delta_reset = true;
        }
        observation.archive = Some(value.archive.clone());
        let mut view = job_view(&job);
        view.stdout_retained_from_line = Some(1);
        view.stderr_retained_from_line = Some(1);
        view.stdout_log_truncated = value.archive.stdout.loss_reason.is_some();
        view.stderr_log_truncated = value.archive.stderr.loss_reason.is_some();
        if let JobArchiveReadResult::Available { page } = outcome {
            observation.stdout_returned_lines = page.stdout.lines().count();
            observation.stderr_returned_lines = page.stderr.lines().count();
            observation.analysis_stdout = page.stdout.clone();
            observation.analysis_stderr = page.stderr.clone();
            observation.analysis_truncated = true;
            observation.stdout_truncated = value.archive.stdout.loss_reason.is_some()
                || page.next_stdout_line < value.archive.stdout.next_line;
            observation.stderr_truncated = value.archive.stderr.loss_reason.is_some()
                || page.next_stderr_line < value.archive.stderr.next_line;
            Ok(Some((
                view,
                Some(page.stdout),
                Some(page.stderr),
                page.next_stdout_line,
                page.next_stderr_line,
                observation,
            )))
        } else {
            observation.archive_unavailable = true;
            observation.stdout_truncated = true;
            observation.stderr_truncated = true;
            observation.analysis_truncated = true;
            let JobArchiveReadResult::Unavailable { reason, .. } = outcome else {
                unreachable!()
            };
            observation.archive_unavailable_reason = Some(
                match reason {
                    JobArchiveUnavailableReason::LineExceedsPageBound => "line_exceeds_page_bound",
                    JobArchiveUnavailableReason::Expired => "archive_expired",
                    JobArchiveUnavailableReason::StorageUnavailable => "archive_unavailable",
                }
                .into(),
            );
            Ok(Some((
                view,
                None,
                None,
                stdout.unwrap_or(1),
                stderr.unwrap_or(1),
                observation,
            )))
        }
    }
}

fn verified_outcome(
    payload: Option<&str>,
    expected: &webcodex_core::job_archive::JobArchiveDescriptor,
) -> Option<JobArchiveReadResult> {
    // JSON escaping can expand the bounded two streams by six times.
    let payload = payload.filter(|s| s.len() <= 1024 * 1024)?;
    let outcome: JobArchiveReadResult = serde_json::from_str(payload).ok()?;
    match &outcome {
        JobArchiveReadResult::Available { page }
            if page.archive == *expected
                && page.stdout.len() <= webcodex_core::job_archive::ARCHIVE_READ_MAX_BYTES
                && page.stderr.len() <= webcodex_core::job_archive::ARCHIVE_READ_MAX_BYTES
                && (1..=expected.stdout.next_line).contains(&page.next_stdout_line)
                && (1..=expected.stderr.next_line).contains(&page.next_stderr_line) =>
        {
            Some(outcome)
        }
        JobArchiveReadResult::Unavailable { archive, .. } if archive == expected => Some(outcome),
        _ => None,
    }
}

fn archive_record(
    registry: &RunnerRegistry,
    value: &ArchivedJobReceipt,
) -> crate::state::ShellJobRecord {
    let mut job = crate::reconciliation::record_from_snapshot(
        &value.receipt.client_id,
        &value.receipt.runner_instance_id,
        value.receipt.auth_group.clone(),
        registry.observation_epoch.clone(),
        None,
        None,
        &value.receipt.snapshot,
        crate::now_ts(),
    );
    job.owner_at_admission = value.receipt.owner_at_admission.clone();
    job.kind = value.receipt.kind.clone();
    job.observation.receipt_expires_at = None;
    job.archive = Some(value.archive.clone());
    job
}

#[cfg(test)]
mod tests {
    use super::*;
    use webcodex_core::job_archive::{JobArchiveDescriptor, JobArchivePage, JobArchiveStream};

    fn descriptor() -> JobArchiveDescriptor {
        JobArchiveDescriptor {
            job_id: "job".into(),
            request_id: "request".into(),
            client_id: "client".into(),
            runner_instance_id: "instance".into(),
            project_id: "agent:client:p".into(),
            project_root_fingerprint: format!("wc_projroot_{}", "a".repeat(64)),
            root_incarnation: "b".repeat(64),
            committed_at: 1,
            stdout: JobArchiveStream {
                retained_bytes: 4,
                next_line: 2,
                loss_reason: None,
            },
            stderr: JobArchiveStream {
                retained_bytes: 0,
                next_line: 1,
                loss_reason: None,
            },
        }
    }

    #[test]
    fn archive_metadata_requires_matching_typed_runner_root_proof() {
        let expected = descriptor();
        assert!(
            verified_outcome(None, &expected).is_none(),
            "timeout/missing capability is not root proof"
        );
        assert!(verified_outcome(Some("archive project incarnation changed"), &expected).is_none());
        let page = JobArchivePage {
            archive: expected.clone(),
            stdout: "old\n".into(),
            stderr: String::new(),
            next_stdout_line: 2,
            next_stderr_line: 1,
        };
        assert!(
            verified_outcome(Some(&serde_json::to_string(&page).unwrap()), &expected).is_none(),
            "legacy untyped page cannot authorize archive metadata"
        );
        let available = JobArchiveReadResult::Available { page };
        assert!(
            verified_outcome(Some(&serde_json::to_string(&available).unwrap()), &expected)
                .is_some()
        );
        let unavailable = JobArchiveReadResult::Unavailable {
            archive: expected.clone(),
            reason: JobArchiveUnavailableReason::StorageUnavailable,
        };
        assert!(verified_outcome(
            Some(&serde_json::to_string(&unavailable).unwrap()),
            &expected
        )
        .is_some());
        let mut replaced = expected.clone();
        replaced.root_incarnation = "c".repeat(64);
        assert!(
            verified_outcome(
                Some(&serde_json::to_string(&unavailable).unwrap()),
                &replaced
            )
            .is_none(),
            "root proof is pinned to this committed execution"
        );
        let mut bad = match available {
            JobArchiveReadResult::Available { page } => page,
            _ => unreachable!(),
        };
        bad.next_stdout_line = 0;
        assert!(verified_outcome(
            Some(
                &serde_json::to_string(&JobArchiveReadResult::Available { page: bad.clone() })
                    .unwrap()
            ),
            &expected
        )
        .is_none());
        bad.next_stdout_line = 2;
        bad.stdout = "x".repeat(webcodex_core::job_archive::ARCHIVE_READ_MAX_BYTES + 1);
        assert!(verified_outcome(
            Some(&serde_json::to_string(&JobArchiveReadResult::Available { page: bad }).unwrap()),
            &expected
        )
        .is_none());
    }
}
