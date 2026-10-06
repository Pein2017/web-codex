//! Process-local observation of canonical potential mutation dispatches.
//! Unlike the Code Mode serialization fence this includes direct calls and all
//! Sessions. It is NOT a filesystem watcher, write lock, or source snapshot.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use webcodex_core::validation_source::{
    ValidationSourceFence, ValidationSourceState, MAX_SOURCE_GENERATION,
};

const MAX_TRACKED_PROJECTS: usize = 4096;

#[derive(Clone, Copy, Default)]
struct WriteAttempt {
    rejected_before_write: bool,
    write_may_have_begun: bool,
}

tokio::task_local! {
    // One scope per canonical dispatch, independent of concurrent requests and
    // nested dispatch. Never serialized or accepted from a Runner/tool payload.
    static WRITE_ATTEMPT: Cell<WriteAttempt>;
}

pub(crate) enum NoSourceWriteRejection {
    GuardedEditPreflight,
    ReadRevision,
}

/// Called only by named local rejection constructors before mutation enqueue.
/// A later possible write permanently disqualifies this invocation's proof.
pub(crate) fn reject_before_source_write(_reason: NoSourceWriteRejection) {
    let _ = WRITE_ATTEMPT.try_with(|attempt| {
        let mut state = attempt.get();
        if !state.write_may_have_begun {
            state.rejected_before_write = true;
            attempt.set(state);
        }
    });
}

pub(crate) fn source_write_may_begin() {
    let _ = WRITE_ATTEMPT.try_with(|attempt| {
        attempt.set(WriteAttempt {
            rejected_before_write: false,
            write_may_have_begun: true,
        });
    });
}

pub(crate) async fn run_observed_mutation<F>(
    guard: Option<MutationObservationGuard>,
    operation: F,
) -> super::ToolResult
where
    F: Future<Output = super::ToolResult>,
{
    WRITE_ATTEMPT
        .scope(Cell::new(WriteAttempt::default()), async move {
            let result = operation.await;
            if let Some(guard) = guard {
                guard.finish(&result);
            }
            result
        })
        .await
}

#[derive(Debug)]
struct ProjectObservation {
    epoch: String,
    generation: u64,
    active: usize,
    uncertain: bool,
}

impl ProjectObservation {
    fn advance(&mut self) {
        if self.generation < MAX_SOURCE_GENERATION {
            self.generation += 1;
        } else {
            self.uncertain = true;
        }
    }

    fn snapshot(&self) -> ValidationSourceFence {
        ValidationSourceFence {
            epoch: self.epoch.clone(),
            generation: self.generation,
            quiescent: self.active == 0 && !self.uncertain,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ValidationSourceRegistry {
    projects: Mutex<BTreeMap<String, Arc<Mutex<ProjectObservation>>>>,
}

impl ValidationSourceRegistry {
    fn project(&self, project: &str) -> Option<Arc<Mutex<ProjectObservation>>> {
        let mut projects = self.projects.lock().ok()?;
        if let Some(state) = projects.get(project) {
            return Some(Arc::clone(state));
        }
        // Never evict an active/uncertain writer and silently recreate a clean
        // baseline. Capacity exhaustion makes new Projects unproven instead.
        if projects.len() >= MAX_TRACKED_PROJECTS {
            return None;
        }
        let state = Arc::new(Mutex::new(ProjectObservation {
            epoch: uuid::Uuid::new_v4().simple().to_string(),
            generation: 0,
            active: 0,
            uncertain: false,
        }));
        projects.insert(project.to_string(), Arc::clone(&state));
        Some(state)
    }

    pub(crate) fn capture(&self, project: &str) -> Option<ValidationSourceFence> {
        let state = self.project(project)?;
        let state = state.lock().ok()?;
        Some(state.snapshot())
    }

    pub(crate) fn observe(
        &self,
        project: &str,
        start: Option<&ValidationSourceFence>,
    ) -> ValidationSourceState {
        ValidationSourceState::observe(start, self.capture(project).as_ref())
    }

    pub(crate) fn begin(&self, project: &str) -> Option<MutationObservationGuard> {
        let state = self.project(project)?;
        {
            let mut state = state.lock().ok()?;
            let Some(active) = state.active.checked_add(1) else {
                state.advance();
                state.uncertain = true;
                return None;
            };
            state.active = active;
        }
        Some(MutationObservationGuard {
            state,
            completed: false,
        })
    }
}

pub(crate) struct MutationObservationGuard {
    state: Arc<Mutex<ProjectObservation>>,
    completed: bool,
}

impl MutationObservationGuard {
    pub(crate) fn finish(mut self, result: &super::ToolResult) {
        // Returning a Job, losing delivery, or lacking mutation truth cannot
        // prove that the potential writer stopped. Keep this epoch uncertain.
        let output = &result.output;
        let execution_state = output
            .get("execution_state")
            .and_then(serde_json::Value::as_str);
        let known_completion = matches!(execution_state, None | Some("not_started" | "completed"))
            && output.get("terminal").and_then(serde_json::Value::as_bool) != Some(false)
            && (output
                .get("command_completed")
                .and_then(serde_json::Value::as_bool)
                != Some(false)
                || output
                    .get("command_started")
                    .and_then(serde_json::Value::as_bool)
                    == Some(false))
            && output
                .get("failure_kind")
                .and_then(serde_json::Value::as_str)
                != Some("outcome_unknown")
            && output.get("job_id").filter(|id| !id.is_null()).is_none()
            && (output
                .get("state_changed")
                .and_then(serde_json::Value::as_bool)
                .is_some()
                || output
                    .get("command_completed")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                || output
                    .get("command_started")
                    .and_then(serde_json::Value::as_bool)
                    == Some(false));
        let no_write = known_completion
            && !result.success
            && execution_state == Some("not_started")
            && output
                .get("command_started")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            && output
                .get("command_completed")
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            && output
                .get("state_changed")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
            && WRITE_ATTEMPT
                .try_with(|attempt| {
                    let attempt = attempt.get();
                    attempt.rejected_before_write && !attempt.write_may_have_begun
                })
                .unwrap_or(false);
        if let Ok(mut state) = self.state.lock() {
            // Completion truth, generation and active count commit together.
            // No-op never rolls a generation back or clears prior uncertainty.
            if !no_write {
                state.advance();
            }
            state.active = state.active.saturating_sub(1);
            state.uncertain |= !known_completion;
            self.completed = true;
        }
    }
}

impl Drop for MutationObservationGuard {
    fn drop(&mut self) {
        if !self.completed {
            if let Ok(mut state) = self.state.lock() {
                state.advance();
                state.active = state.active.saturating_sub(1);
                state.uncertain = true;
            }
        }
    }
}

/// Conservative potential-source-effect classification, derived from canonical
/// metadata. Wrappers do not write: their canonical children are observed here.
/// Read-only structured validators can themselves run arbitrary build/test code;
/// that is deliberately OUTSIDE this dispatch fence, hence freshness is unproven.
pub(crate) fn observes_potential_mutation(call: &super::ToolCall) -> bool {
    let name = call.tool_name();
    if matches!(
        name,
        "code_mode_exec"
            | "code_mode_exec_effectful"
            | "code_mode_exec_mutating"
            | "cargo_check"
            | "cargo_test"
            | "go_test"
    ) {
        return false;
    }
    if matches!(
        call,
        super::ToolCall::CargoFmt {
            check: Some(true),
            ..
        }
    ) {
        return false;
    }
    let metadata = webcodex_tool_contracts::runtime_tool_metadata(name);
    metadata.requires_project
        && metadata.effect != webcodex_tool_contracts::ToolEffect::Observe
        && (metadata.shell_like
            || matches!(
                metadata.risk,
                webcodex_tool_contracts::ToolRisk::ProjectWrite
                    | webcodex_tool_contracts::ToolRisk::JobRun
                    | webcodex_tool_contracts::ToolRisk::CheckpointManage
            ))
}

#[cfg(test)]
#[path = "tests/validation_source.rs"]
mod tests;
