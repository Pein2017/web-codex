use super::*;
use webcodex_core::validation_source::{ObservedMutationFence, ValidationFreshness};

#[test]
fn source_fence_multiple_writers_remain_active_until_every_known_completion() {
    let registry = ValidationSourceRegistry::default();
    let first = registry.begin("p").unwrap();
    let second = registry.begin("p").unwrap();
    first.finish(&noop());
    let during = registry.capture("p").unwrap();
    assert!(!during.quiescent);
    assert_eq!(
        registry.observe("p", Some(&during)).observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
    second.finish(&noop());
    assert!(registry.capture("p").unwrap().quiescent);
    assert_eq!(
        registry.observe("p", Some(&during)).observed_mutation_fence,
        ObservedMutationFence::Crossed
    );
}

#[test]
fn source_fence_handoff_and_generation_exhaustion_cannot_manufacture_quiescence() {
    let registry = ValidationSourceRegistry::default();
    registry
        .begin("job")
        .unwrap()
        .finish(&crate::tool_runtime::ToolResult::ok(
            serde_json::json!({"job_id":"existing-job","state_changed":false}),
        ));
    assert!(!registry.capture("job").unwrap().quiescent);
    let state = registry.project("exhausted").unwrap();
    state.lock().unwrap().generation = MAX_SOURCE_GENERATION;
    let before = registry.capture("exhausted").unwrap();
    registry.begin("exhausted").unwrap().finish(&noop());
    let after = registry.capture("exhausted").unwrap();
    assert_eq!(after.generation, MAX_SOURCE_GENERATION);
    assert!(!after.quiescent);
    assert_eq!(
        registry
            .observe("exhausted", Some(&before))
            .observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
}

fn noop() -> crate::tool_runtime::ToolResult {
    crate::tool_runtime::ToolResult::ok(serde_json::json!({"state_changed": false}))
}

#[test]
fn source_fence_tracks_inflight_completed_and_noop_attempts_not_content_equality() {
    let registry = ValidationSourceRegistry::default();
    let before = registry.capture("p").unwrap();
    let writer = registry.begin("p").unwrap();
    let during = registry.capture("p").unwrap();
    assert!(!during.quiescent);
    assert_eq!(
        registry.observe("p", Some(&before)).freshness,
        ValidationFreshness::Unproven
    );
    assert_eq!(
        registry.observe("p", Some(&before)).observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
    writer.finish(&noop());
    assert_eq!(
        registry.observe("p", Some(&during)).freshness,
        ValidationFreshness::Stale
    );
    let after = registry.capture("p").unwrap();
    assert!(after.quiescent);
    assert_eq!(
        registry.observe("p", Some(&after)).observed_mutation_fence,
        ObservedMutationFence::Uncrossed
    );
    assert_eq!(
        registry.observe("p", Some(&after)).freshness,
        ValidationFreshness::Unproven
    );
}

#[test]
fn source_fence_cancellation_and_unknown_writers_never_reopen_clean_epoch() {
    let registry = ValidationSourceRegistry::default();
    drop(registry.begin("p"));
    let after = registry.capture("p").unwrap();
    assert!(!after.quiescent);
    registry.begin("p").unwrap().finish(&noop());
    assert!(!registry.capture("p").unwrap().quiescent);
    assert_eq!(
        registry
            .observe("p", Some(&registry.capture("p").unwrap()))
            .observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
}

#[test]
fn source_fence_project_and_restart_epochs_are_not_interchangeable() {
    let registry = ValidationSourceRegistry::default();
    let start = registry.capture("one").unwrap();
    registry.begin("two").unwrap().finish(&noop());
    assert_eq!(
        registry
            .observe("one", Some(&start))
            .observed_mutation_fence,
        ObservedMutationFence::Uncrossed
    );
    assert_eq!(
        registry
            .observe("two", Some(&start))
            .observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
    assert_eq!(
        ValidationSourceRegistry::default()
            .observe("one", Some(&start))
            .observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
}

#[test]
fn source_fence_capacity_does_not_evict_active_writers() {
    let registry = ValidationSourceRegistry::default();
    let writer = registry.begin("p").unwrap();
    for i in 1..MAX_TRACKED_PROJECTS {
        assert!(registry.capture(&i.to_string()).is_some());
    }
    assert!(registry.capture("overflow").is_none());
    assert!(!registry.capture("p").unwrap().quiescent);
    writer.finish(&noop());
    assert!(registry.capture("p").unwrap().quiescent);
}

fn rejected_before_write() -> crate::tool_runtime::ToolResult {
    reject_before_source_write(NoSourceWriteRejection::GuardedEditPreflight);
    crate::tool_runtime::ToolResult::err_with_output(
        "local guard rejected before enqueue",
        serde_json::json!({"execution_state":"not_started","state_changed":false}),
    )
}

#[tokio::test]
async fn source_fence_internal_prewrite_rejection_preserves_only_uncrossed_unproven_observation() {
    let registry = ValidationSourceRegistry::default();
    let before = registry.capture("p").unwrap();
    let guard = registry.begin("p").unwrap();
    let during = registry.capture("p").unwrap();
    run_observed_mutation(Some(guard), async { rejected_before_write() }).await;
    assert_eq!(registry.capture("p").unwrap(), before);
    let observed = registry.observe("p", Some(&before));
    assert_eq!(
        observed.observed_mutation_fence,
        ObservedMutationFence::Uncrossed
    );
    assert_eq!(observed.freshness, ValidationFreshness::Unproven);
    assert_eq!(
        registry.observe("p", Some(&during)).observed_mutation_fence,
        ObservedMutationFence::Unknown
    );
}

#[tokio::test]
async fn source_fence_noop_never_rolls_back_other_writers_or_clears_uncertainty() {
    let registry = ValidationSourceRegistry::default();
    let before = registry.capture("p").unwrap();
    let first = registry.begin("p").unwrap();
    let second = registry.begin("p").unwrap();
    second.finish(&crate::tool_runtime::ToolResult::ok(
        serde_json::json!({"state_changed":true}),
    ));
    let generation = registry.capture("p").unwrap().generation;
    run_observed_mutation(Some(first), async { rejected_before_write() }).await;
    assert_eq!(registry.capture("p").unwrap().generation, generation);
    assert_eq!(
        registry.observe("p", Some(&before)).observed_mutation_fence,
        ObservedMutationFence::Crossed
    );

    drop(registry.begin("p"));
    let uncertain = registry.capture("p").unwrap();
    run_observed_mutation(registry.begin("p"), async { rejected_before_write() }).await;
    assert_eq!(registry.capture("p").unwrap(), uncertain);
    assert!(!uncertain.quiescent);
}

#[tokio::test]
async fn source_fence_possible_write_permanently_revokes_prewrite_proof() {
    let registry = ValidationSourceRegistry::default();
    let before = registry.capture("p").unwrap();
    run_observed_mutation(registry.begin("p"), async {
        let _ = rejected_before_write();
        source_write_may_begin();
        // A later local-looking rejection may not revive the earlier proof.
        rejected_before_write()
    })
    .await;
    assert_eq!(
        registry.observe("p", Some(&before)).observed_mutation_fence,
        ObservedMutationFence::Crossed
    );
}

#[tokio::test]
async fn source_fence_serialized_noops_exit_zero_and_jobs_never_supply_proof() {
    let registry = ValidationSourceRegistry::default();
    for output in [
        serde_json::json!({"state_changed":false}),
        serde_json::json!({"command_completed":true,"exit_code":0}),
        serde_json::json!({"execution_state":"not_started","state_changed":false,"no_source_write":true}),
        serde_json::json!({"execution_state":"completed","state_changed":false,"rollback_complete":true}),
        serde_json::json!({"job_id":"job","state_changed":false}),
        serde_json::json!({"execution_state":"outcome_unknown","state_changed":false}),
        serde_json::json!({"execution_state":"running","state_changed":false}),
        serde_json::json!({"terminal":false,"state_changed":false}),
    ] {
        let before = registry.capture("p").unwrap();
        run_observed_mutation(registry.begin("p"), async {
            crate::tool_runtime::ToolResult::err_with_output("not an internal proof", output)
        })
        .await;
        assert!(registry.capture("p").unwrap().generation > before.generation);
    }
    assert!(!registry.capture("p").unwrap().quiescent);
    let before = registry.capture("p").unwrap();
    run_observed_mutation(registry.begin("p"), async {
        let _ = rejected_before_write();
        crate::tool_runtime::ToolResult::ok(
            serde_json::json!({"job_id":"job","state_changed":false}),
        )
    })
    .await;
    assert!(registry.capture("p").unwrap().generation > before.generation);
    assert!(!registry.capture("p").unwrap().quiescent);
}

#[tokio::test]
async fn source_fence_proof_is_isolated_across_nested_and_concurrent_dispatch() {
    let registry = ValidationSourceRegistry::default();
    let parent_before = registry.capture("parent").unwrap();
    let child_before = registry.capture("child").unwrap();
    run_observed_mutation(registry.begin("parent"), async {
        run_observed_mutation(registry.begin("child"), async { rejected_before_write() }).await;
        crate::tool_runtime::ToolResult::err_with_output(
            "unproven outer operation",
            serde_json::json!({"execution_state":"not_started","state_changed":false}),
        )
    })
    .await;
    assert_eq!(registry.capture("child").unwrap(), child_before);
    assert!(registry.capture("parent").unwrap().generation > parent_before.generation);

    let gate = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let before = registry.capture("p").unwrap();
    let (no_write, possible_write) = tokio::join!(
        run_observed_mutation(registry.begin("p"), async {
            let result = rejected_before_write();
            gate.wait().await;
            result
        }),
        run_observed_mutation(registry.begin("p"), async {
            gate.wait().await;
            noop()
        }),
    );
    assert!(!no_write.success);
    assert!(possible_write.success);
    assert_eq!(
        registry.capture("p").unwrap().generation,
        before.generation + 1
    );
    assert!(registry.capture("p").unwrap().quiescent);
}

#[tokio::test]
async fn source_fence_cancelled_scoped_proof_and_active_overflow_remain_unknown() {
    let registry = std::sync::Arc::new(ValidationSourceRegistry::default());
    let before = registry.capture("p").unwrap();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let task = tokio::spawn({
        let registry = registry.clone();
        async move {
            run_observed_mutation(registry.begin("p"), async {
                let result = rejected_before_write();
                started_tx.send(()).unwrap();
                std::future::pending::<()>().await;
                result
            })
            .await
        }
    });
    started_rx.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    let after = registry.capture("p").unwrap();
    assert!(after.generation > before.generation);
    assert!(!after.quiescent);
    let state = registry.project("overflow").unwrap();
    state.lock().unwrap().active = usize::MAX;
    assert!(registry.begin("overflow").is_none());
    assert!(!registry.capture("overflow").unwrap().quiescent);
}

#[test]
fn source_fence_commit_hooks_and_unknown_shell_keep_potential_effect_classification() {
    for tool in ["git_commit_paths", "run_shell"] {
        let args = if tool == "run_shell" {
            serde_json::json!({"project":"p","command":"true"})
        } else {
            serde_json::json!({"project":"p","expected_head":"a".repeat(40),"paths":["file"],"message":"commit"})
        };
        let call = crate::tool_runtime::ToolCall::from_tool_name(tool, args).unwrap();
        assert!(observes_potential_mutation(&call), "{tool}");
    }
}
