use super::support::*;
use crate::tool_runtime::workspace_baseline::{
    capture_workspace_baseline, compare_workspace_observations,
};
use crate::tool_runtime::{ToolCall, ToolResult, ToolRuntime};
use serde_json::{json, Value};
use std::fs;
use std::time::{Duration, Instant};

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const ROOT_KEY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn observation(project: &str, head: &str, files: &[(&str, &str)], total: usize) -> ToolResult {
    let count = |status: &str| files.iter().filter(|(_, kind)| *kind == status).count();
    ToolResult::ok(json!({
        "project": project,
        "git_available": true,
        "transport_safe": true,
        "status_observation": {"status": "observed", "exit_code": 0},
        "head": {"commit": head},
        "head_exit": 0,
        "files": files.iter().map(|(path, status)| json!({"path": path, "status": status})).collect::<Vec<_>>(),
        "files_total": total,
        "files_returned": files.len(),
        "files_truncated": total > files.len(),
        "files_limit": 200,
        "counts": {
            "modified": count("modified") + total.saturating_sub(files.len()),
            "added": count("added"), "deleted": count("deleted"),
            "renamed": count("renamed"), "copied": count("copied"),
            "untracked": count("untracked"), "conflicted": count("conflicted"),
        },
    }))
}

fn paths(value: &Value, field: &str) -> Vec<String> {
    value[field]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["path"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn coding_task_output_schemas_declare_bounded_baseline_and_comparison() {
    let start = crate::tool_runtime::registry::output_schema_for_tool("work_on_project");
    let finish = crate::tool_runtime::registry::output_schema_for_tool("finish_coding_task");
    assert_eq!(
        start["properties"]["output"]["properties"]["workspace_baseline"]["properties"]["status"]
            ["enum"],
        json!(["complete", "partial", "unavailable", "legacy_missing"])
    );
    assert_eq!(
        finish["properties"]["output"]["properties"]["workspace_observations"]["properties"]
            ["newly_dirty"]["maxItems"],
        16
    );
    assert_eq!(
        finish["properties"]["output"]["properties"]["workspace_observations"]["properties"]
            ["status"]["enum"],
        json!([
            "comparable",
            "partial",
            "unavailable",
            "legacy_missing",
            "target_changed",
            "head_changed"
        ])
    );
}

#[test]
fn comparison_only_infers_absence_when_both_snapshots_prove_it() {
    let initial = capture_workspace_baseline(
        &observation("demo", HEAD, &[("existing.txt", "modified")], 1),
        "demo",
        ROOT_KEY,
    );
    let final_state = observation(
        "demo",
        HEAD,
        &[("existing.txt", "modified"), ("new.txt", "untracked")],
        2,
    );
    let verdict = compare_workspace_observations(Some(&initial), &final_state, "demo", ROOT_KEY);
    assert_eq!(verdict["status"], "comparable");
    assert_eq!(paths(&verdict, "pre_existing_dirty"), vec!["existing.txt"]);
    assert_eq!(paths(&verdict, "newly_dirty"), vec!["new.txt"]);
    assert!(paths(&verdict, "cleared").is_empty());
    assert_eq!(paths(&verdict, "overlapping_dirty"), vec!["existing.txt"]);
    assert_eq!(verdict["overlap_content_unknown"], true);

    let partial_start = capture_workspace_baseline(
        &observation("demo", HEAD, &[("existing.txt", "modified")], 2),
        "demo",
        ROOT_KEY,
    );
    let verdict =
        compare_workspace_observations(Some(&partial_start), &final_state, "demo", ROOT_KEY);
    assert_eq!(verdict["status"], "partial");
    assert!(
        paths(&verdict, "newly_dirty").is_empty(),
        "missing baseline record must not prove newly dirty"
    );
    assert!(paths(&verdict, "cleared").is_empty());
    assert_eq!(paths(&verdict, "overlapping_dirty"), vec!["existing.txt"]);

    let partial_end = observation("demo", HEAD, &[("new.txt", "untracked")], 2);
    let verdict = compare_workspace_observations(Some(&initial), &partial_end, "demo", ROOT_KEY);
    assert_eq!(verdict["status"], "partial");
    assert_eq!(paths(&verdict, "newly_dirty"), vec!["new.txt"]);
    assert!(
        paths(&verdict, "cleared").is_empty(),
        "missing finish record must not prove cleared"
    );

    let clean_end = observation("demo", HEAD, &[], 0);
    let verdict = compare_workspace_observations(Some(&initial), &clean_end, "demo", ROOT_KEY);
    assert_eq!(paths(&verdict, "cleared"), vec!["existing.txt"]);
}

#[test]
fn unavailable_legacy_target_or_head_change_never_infer_cleared_or_new() {
    let initial = capture_workspace_baseline(
        &observation("demo", HEAD, &[("existing.txt", "modified")], 1),
        "demo",
        ROOT_KEY,
    );
    let final_state = observation("demo", HEAD, &[("new.txt", "modified")], 1);
    for (label, baseline, finish, project) in [
        ("legacy_missing", None, final_state, "demo"),
        (
            "unavailable",
            Some(&initial),
            ToolResult::err("Runner inspection unavailable"),
            "demo",
        ),
        (
            "target_changed",
            Some(&initial),
            observation("other", HEAD, &[("new.txt", "modified")], 1),
            "demo",
        ),
        (
            "head_changed",
            Some(&initial),
            observation("demo", &"a".repeat(40), &[("new.txt", "modified")], 1),
            "demo",
        ),
    ] {
        let verdict = compare_workspace_observations(baseline, &finish, project, ROOT_KEY);
        assert_eq!(verdict["status"], label);
        assert!(paths(&verdict, "cleared").is_empty());
        assert!(paths(&verdict, "newly_dirty").is_empty());
    }
    let missing_count = ToolResult::ok(json!({
        "project": "demo", "git_available": true, "transport_safe": true,
        "status_observation": {"status": "observed"},
        "head": {"commit": HEAD}, "head_exit": 0,
        "files": [], "files_total": 0, "files_returned": 0,
        "files_truncated": false, "counts": {"modified": 1},
    }));
    let verdict = compare_workspace_observations(Some(&initial), &missing_count, "demo", ROOT_KEY);
    assert_ne!(
        verdict["status"], "comparable",
        "count metadata cannot falsely prove cleanliness"
    );
    assert!(paths(&verdict, "cleared").is_empty());
    let same_id_new_root = compare_workspace_observations(
        Some(&initial),
        &observation("demo", HEAD, &[("new.txt", "modified")], 1),
        "demo",
        &"b".repeat(64),
    );
    assert_eq!(same_id_new_root["status"], "target_changed");
    assert!(paths(&same_id_new_root, "newly_dirty").is_empty());
}

#[test]
fn status_storage_and_model_presentation_have_independent_limits() {
    let files = (0..200)
        .map(|idx| (format!("{idx:03}-{}", "x".repeat(380)), "modified"))
        .collect::<Vec<_>>();
    let references = files
        .iter()
        .map(|(path, status)| (path.as_str(), *status))
        .collect::<Vec<_>>();
    let source = observation("demo", HEAD, &references, 200);
    let stored = capture_workspace_baseline(&source, "demo", ROOT_KEY);
    assert!(
        !stored.complete,
        "64 KiB ledger budget must force partial observation"
    );
    assert!(stored.entries.len() < 200);
    assert!(
        stored
            .entries
            .iter()
            .map(|entry| entry.path.len() + entry.status.len() + 32)
            .sum::<usize>()
            <= 64 * 1024
    );
    let projected = compare_workspace_observations(Some(&stored), &source, "demo", ROOT_KEY);
    assert_eq!(projected["status"], "partial");
    assert!(paths(&projected, "newly_dirty").is_empty());
    assert!(paths(&projected, "cleared").is_empty());
    assert_eq!(projected["display"]["truncated"], true);
    assert!(projected["display"]["returned_count"].as_u64().unwrap() <= 32);
    assert!(serde_json::to_vec(&projected).unwrap().len() < 16 * 1024);

    let single_very_long = observation("demo", HEAD, &[("y".repeat(700).as_str(), "untracked")], 1);
    let stored = capture_workspace_baseline(&single_very_long, "demo", ROOT_KEY);
    assert!(
        stored.complete,
        "producer and persistence retained the single path"
    );
    let projected =
        compare_workspace_observations(Some(&stored), &single_very_long, "demo", ROOT_KEY);
    assert_eq!(projected["counts"]["pre_existing_dirty"], 1);
    assert_eq!(
        projected["pre_existing_dirty"],
        json!([]),
        "presentation must not expose overlong path"
    );
    assert_eq!(projected["display"]["truncated"], true);
}

async fn service_until_complete(
    runtime: &ToolRuntime,
    client: &str,
    task: &tokio::task::JoinHandle<ToolResult>,
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !task.is_finished() {
        assert!(
            Instant::now() < deadline,
            "coding-task real-entry fixture timed out"
        );
        if let Some(request) = probe_patch_agent_request(runtime, client).await {
            complete_agent_request_by_running_locally(runtime, client, request).await;
        } else {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

#[tokio::test]
async fn real_coding_entry_retains_startup_dirty_paths_through_resume_and_finish() {
    let temp = tempfile::tempdir().unwrap();
    init_git_repo(temp.path());
    commit_file(temp.path(), "existing.txt", "original\n", "add fixture");
    fs::write(temp.path().join("existing.txt"), "dirty before Session\n").unwrap();
    let runtime = test_runtime();
    let client = "baseline-coding";
    let project = register_runner_project_at_path(&runtime, client, "demo", temp.path()).await;
    let auth = auth_context(None, true);
    let start = tokio::spawn({
        let runtime = runtime.clone();
        let project = project.clone();
        let auth = auth.clone();
        async move {
            runtime
                .dispatch_with_auth(
                    ToolCall::WorkOnProject {
                        project,
                        client_id: None,
                        path: None,
                        mode: None,
                        base_ref: None,
                        instruction: "preserve baseline".into(),
                        session_id: None,
                        guidance_profile: Default::default(),
                        include_extension_catalog: false,
                    },
                    Some(&auth),
                )
                .await
        }
    });
    service_until_complete(&runtime, client, &start).await;
    let start = start.await.unwrap();
    assert!(start.success, "{:?}", start.error);
    let session_id = start.output["session_id"].as_str().unwrap().to_string();
    assert_eq!(start.output["workspace_baseline"]["status"], "complete");
    assert_eq!(
        runtime
            .sessions
            .summary(&session_id, None)
            .unwrap()
            .workspace_baseline
            .unwrap()
            .entries[0]
            .path,
        "existing.txt"
    );
    fs::write(
        temp.path().join("existing.txt"),
        "dirty after Session too\n",
    )
    .unwrap();
    fs::write(temp.path().join("new.txt"), "created later\n").unwrap();

    let resume = tokio::spawn({
        let runtime = runtime.clone();
        let project = project.clone();
        let auth = auth.clone();
        let session_id = session_id.clone();
        async move {
            runtime
                .dispatch_with_auth(
                    ToolCall::WorkOnProject {
                        project,
                        client_id: None,
                        path: None,
                        mode: None,
                        base_ref: None,
                        instruction: "resume".into(),
                        session_id: Some(session_id),
                        guidance_profile: Default::default(),
                        include_extension_catalog: false,
                    },
                    Some(&auth),
                )
                .await
        }
    });
    service_until_complete(&runtime, client, &resume).await;
    assert!(resume.await.unwrap().success);
    assert_eq!(
        runtime
            .sessions
            .summary(&session_id, None)
            .unwrap()
            .workspace_baseline
            .unwrap()
            .entries
            .len(),
        1
    );

    let finish = tokio::spawn({
        let runtime = runtime.clone();
        let project = project.clone();
        let auth = auth.clone();
        let session_id = session_id.clone();
        async move {
            runtime
                .dispatch_with_auth(
                    ToolCall::FinishCodingTask {
                        project,
                        session_id,
                        summary_only: true,
                        include_diff: Some(false),
                        include_workspace: Some(true),
                        include_hygiene: Some(false),
                        include_handoff: Some(false),
                        include_validation_summary: Some(false),
                    },
                    Some(&auth),
                )
                .await
        }
    });
    service_until_complete(&runtime, client, &finish).await;
    let finish = finish.await.unwrap();
    assert!(finish.success, "{:?}", finish.error);
    assert_eq!(
        finish.output["workspace_observations"]["status"],
        "comparable"
    );
    assert_eq!(
        paths(
            &finish.output["workspace_observations"],
            "pre_existing_dirty"
        ),
        vec!["existing.txt"]
    );
    assert_eq!(
        paths(&finish.output["workspace_observations"], "newly_dirty"),
        vec!["new.txt"]
    );
    assert_eq!(
        paths(
            &finish.output["workspace_observations"],
            "overlapping_dirty"
        ),
        vec!["existing.txt"]
    );
    assert_eq!(
        finish.output["workspace_observations"]["overlap_content_unknown"],
        true
    );
}
