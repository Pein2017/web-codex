//! Informational creation/closeout Git observations. No file reads, hashes,
//! command wrapping, or actor/Session authorship inference.

use super::git::SHOW_CHANGES_MAX_STATUS_FILES;
use super::ToolResult;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use webcodex_workflow_session::{
    WorkspaceBaseline, WorkspaceBaselineEntry, MAX_WORKSPACE_BASELINE_METADATA_BYTES,
    MAX_WORKSPACE_BASELINE_PATH_BYTES,
};

/// The Git source already bounded every status record to 4 KiB and 200 files.
/// Presentation has a separate ceiling: no long path or huge dirty worktree
/// makes a compact finish response grow without limit.
const MAX_PRESENTED_PATH_BYTES: usize = 512;
const MAX_PRESENTED_ITEMS_PER_GROUP: usize = 16;
const MAX_PRESENTED_PATH_BYTES_TOTAL: usize = 12 * 1024;

pub(super) fn capture_workspace_baseline(
    result: &ToolResult,
    project: &str,
    repository_key: &str,
) -> WorkspaceBaseline {
    let unavailable = || WorkspaceBaseline::unavailable(project, repository_key);
    let output = &result.output;
    if !result.success
        || output.get("project").and_then(Value::as_str) != Some(project)
        || output.get("git_available").and_then(Value::as_bool) != Some(true)
        || output.get("transport_safe").and_then(Value::as_bool) != Some(true)
        || output
            .pointer("/status_observation/status")
            .and_then(Value::as_str)
            != Some("observed")
        || output.get("head_exit").and_then(Value::as_i64) != Some(0)
    {
        return unavailable();
    }
    let Some(files) = output.get("files").and_then(Value::as_array) else {
        return unavailable();
    };
    let (Some(total), Some(returned), Some(truncated)) = (
        output.get("files_total").and_then(Value::as_u64),
        output.get("files_returned").and_then(Value::as_u64),
        output.get("files_truncated").and_then(Value::as_bool),
    ) else {
        return unavailable();
    };
    if returned as usize != files.len()
        || files.len() > SHOW_CHANGES_MAX_STATUS_FILES
        || total < returned
        || truncated != (total > returned)
        || output.get("files_limit").and_then(Value::as_u64)
            != Some(SHOW_CHANGES_MAX_STATUS_FILES as u64)
    {
        return unavailable();
    }
    // Streaming status category counts include all entries even when the
    // returned paths were producer-truncated. Presence and exact sum matter;
    // success/clean alone cannot prove completeness or safe absence inference.
    let total_categories = [
        "modified",
        "added",
        "deleted",
        "renamed",
        "copied",
        "untracked",
        "conflicted",
    ]
    .into_iter()
    .try_fold(0_u64, |sum, category| {
        sum.checked_add(output.pointer(&format!("/counts/{category}"))?.as_u64()?)
    });
    if total_categories != Some(total) {
        return unavailable();
    }
    let mut entries = Vec::with_capacity(files.len());
    let mut retained_bytes = 0usize;
    let mut metadata_truncated = false;
    for file in files {
        let (Some(path), Some(status)) = (
            file.get("path").and_then(Value::as_str),
            file.get("status").and_then(Value::as_str),
        ) else {
            return unavailable();
        };
        if path.len() > MAX_WORKSPACE_BASELINE_PATH_BYTES {
            return unavailable();
        }
        let entry_bytes = path.len() + status.len() + 32;
        if retained_bytes + entry_bytes > MAX_WORKSPACE_BASELINE_METADATA_BYTES {
            metadata_truncated = true;
            continue;
        }
        retained_bytes += entry_bytes;
        entries.push(WorkspaceBaselineEntry {
            path: path.to_string(),
            status: status.to_string(),
        });
    }
    WorkspaceBaseline {
        project: project.to_string(),
        repository_key: repository_key.to_string(),
        head: output
            .pointer("/head/commit")
            .and_then(Value::as_str)
            .map(str::to_string),
        complete: !truncated && !metadata_truncated,
        files_total: usize::try_from(total).ok(),
        entries,
    }
    .validated(project)
}

pub(super) fn startup_baseline_projection(baseline: Option<&WorkspaceBaseline>) -> Value {
    let Some(baseline) = baseline else {
        return json!({"status": "legacy_missing", "pre_existing_dirty_count": null, "files_total": null});
    };
    let available = observation_status(baseline) != "unavailable";
    json!({
        "status": observation_status(baseline),
        "pre_existing_dirty_count": available.then_some(baseline.entries.len()),
        "files_total": baseline.files_total,
    })
}

fn observation_status(baseline: &WorkspaceBaseline) -> &'static str {
    if baseline.head.is_none() || baseline.files_total.is_none() {
        "unavailable"
    } else if baseline.complete {
        "complete"
    } else {
        "partial"
    }
}

fn observation_details(baseline: Option<&WorkspaceBaseline>) -> Value {
    match baseline {
        Some(baseline) => json!({
            "status": observation_status(baseline),
            "head": baseline.head,
            "complete": baseline.complete && baseline.head.is_some(),
            "files_total": baseline.files_total,
            "observed_count": (observation_status(baseline) != "unavailable").then_some(baseline.entries.len()),
        }),
        None => json!({
            "status": "legacy_missing", "head": null, "complete": false,
            "files_total": null, "observed_count": null,
        }),
    }
}

/// Same root and unchanged HEAD allow bounded status differences, not content
/// sameness, effects outside Git, or attribution to the Workflow Session.
pub(super) fn compare_workspace_observations(
    initial: Option<&WorkspaceBaseline>,
    final_result: &ToolResult,
    project: &str,
    repository_key: &str,
) -> Value {
    let final_observation = capture_workspace_baseline(final_result, project, repository_key);
    let target_changed = initial.is_some_and(|baseline| {
        baseline.project != project || baseline.repository_key != repository_key
    }) || final_result
        .output
        .get("project")
        .and_then(Value::as_str)
        .is_some_and(|reported| reported != project);
    let status = if initial.is_none() {
        "legacy_missing"
    } else if target_changed {
        "target_changed"
    } else if initial.is_some_and(|baseline| observation_status(baseline) == "unavailable")
        || observation_status(&final_observation) == "unavailable"
    {
        "unavailable"
    } else if initial.and_then(|baseline| baseline.head.as_deref())
        != final_observation.head.as_deref()
    {
        "head_changed"
    } else if initial.is_some_and(|baseline| baseline.complete) && final_observation.complete {
        "comparable"
    } else {
        "partial"
    };

    let old: BTreeMap<&str, &WorkspaceBaselineEntry> = initial
        .into_iter()
        .flat_map(|baseline| &baseline.entries)
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let new: BTreeMap<&str, &WorkspaceBaselineEntry> = final_observation
        .entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let comparable = matches!(status, "comparable" | "partial");
    let mut pre_existing = old
        .values()
        .map(|entry| json!({"path": entry.path, "status": entry.status}))
        .collect::<Vec<_>>();
    let mut newly_dirty = if comparable && initial.is_some_and(|baseline| baseline.complete) {
        new.iter()
            .filter(|(path, _)| !old.contains_key(**path))
            .map(|(_, entry)| json!({"path": entry.path, "status": entry.status}))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let mut cleared = if status == "comparable" {
        old.iter()
            .filter(|(path, _)| !new.contains_key(**path))
            .map(|(_, entry)| json!({"path": entry.path, "status": entry.status}))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let mut overlapping = if comparable {
        old.iter().filter_map(|(path, entry)| new.get(path).map(|last| {
            json!({"path": entry.path, "startup_status": entry.status, "finish_status": last.status})
        })).collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let counts = json!({
        "pre_existing_dirty": initial.and_then(|baseline| (observation_status(baseline) != "unavailable" && (baseline.complete || !pre_existing.is_empty())).then_some(pre_existing.len())),
        "newly_dirty": (comparable && initial.is_some_and(|baseline| baseline.complete) && (final_observation.complete || !newly_dirty.is_empty())).then_some(newly_dirty.len()),
        "cleared": (status == "comparable").then_some(cleared.len()),
        "overlapping_dirty": (comparable && (status == "comparable" || !overlapping.is_empty())).then_some(overlapping.len()),
    });
    let overlap_content_unknown = !overlapping.is_empty();
    let total_presentable =
        pre_existing.len() + newly_dirty.len() + cleared.len() + overlapping.len();
    let mut remaining_bytes = MAX_PRESENTED_PATH_BYTES_TOTAL;
    let mut returned_count = 0;
    for group in [
        &mut pre_existing,
        &mut newly_dirty,
        &mut cleared,
        &mut overlapping,
    ] {
        let mut allowed = Vec::new();
        for item in group.iter() {
            let Some(path) = item.get("path").and_then(Value::as_str) else {
                continue;
            };
            let bytes = serde_json::to_vec(item).map_or(usize::MAX, |encoded| encoded.len());
            if path.len() <= MAX_PRESENTED_PATH_BYTES
                && allowed.len() < MAX_PRESENTED_ITEMS_PER_GROUP
                && bytes <= remaining_bytes
            {
                remaining_bytes -= bytes;
                allowed.push(item.clone());
            }
        }
        returned_count += allowed.len();
        *group = allowed;
    }
    json!({
        "status": status,
        "startup": observation_details(initial),
        "finish": observation_details(Some(&final_observation)),
        "counts": counts,
        "display": {"truncated": returned_count < total_presentable, "returned_count": returned_count},
        "pre_existing_dirty": pre_existing,
        "newly_dirty": newly_dirty,
        "cleared": cleared,
        "overlapping_dirty": overlapping,
        "overlap_content_unknown": overlap_content_unknown,
    })
}
