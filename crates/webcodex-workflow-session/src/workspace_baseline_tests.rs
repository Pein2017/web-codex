use crate::*;
use webcodex_core::workflow_session_contract::SessionMode;

const PROJECT: &str = "agent:oe:baseline-fixture";
const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const ROOT_KEY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

// Frozen synthetic v2 shape from 50281db4:model.rs. These are fixture IDs and
// bodies, independent of current writers and of any production Session state.
const LOCAL_V2_LEDGER: &str = r#"{
  "version": 2,
  "sessions": [{
    "session_id": "wc_sess_0123456789abcdef0123456789abcdef",
    "project": "agent:oe:baseline-fixture",
    "owner_authority_fingerprint": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    "title": "frozen local v2 fixture",
    "mode": "normal",
    "guards": {"deny_write_tools": false, "deny_shell_tools": false},
    "execution_context": {},
    "workspace_baseline": {
      "project": "agent:oe:baseline-fixture",
      "repository_key": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "head": "0123456789abcdef0123456789abcdef01234567",
      "complete": true, "files_total": 1,
      "entries": [{"path": "existing.txt", "status": "modified"}]
    },
    "lifecycle": "active", "created_at": 1, "updated_at": 2,
    "context_revision": 17, "events_observed": 1,
    "events": [{
      "event_id": "evt_0123456789abcdef0123456789abcdef",
      "session_id": "wc_sess_0123456789abcdef0123456789abcdef",
      "kind": "tool_call_finished", "context_revision": 16,
      "timestamp": 2, "transport": "api", "tool_name": "show_changes",
      "project": "agent:oe:baseline-fixture",
      "resolved_project": "agent:oe:baseline-fixture", "risk_class": "low",
      "read_like": true, "write_like": false, "shell_like": false,
      "git_like": true, "change_summary_like": true,
      "started_at": 1, "finished_at": 2, "duration_ms": 1,
      "status": "succeeded", "changed_paths": []
    }],
    "messages": [{
      "message_id": "wc_msg_0123456789abcdef0123456789abcdef",
      "session_id": "wc_sess_0123456789abcdef0123456789abcdef",
      "created_at": 1, "kind": "note", "status": "open",
      "priority": "normal", "message": "preserve this fixture message", "tags": []
    }],
    "message_observation_revision": 1, "message_observation_floor": 0,
    "message_observation_revisions": {"wc_msg_0123456789abcdef0123456789abcdef": 1},
    "assignment_history_floors": {}, "assignment_history_tracking_complete": true,
    "completion_assignment_fence_fingerprints": {},
    "completion_assignment_fence_tracking_complete": true
  }]
}"#;

#[test]
fn local_v2_workspace_baseline_fixture_preserves_session_events_and_messages() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("sessions.json");
    std::fs::write(&path, LOCAL_V2_LEDGER).unwrap();
    let frozen: serde_json::Value = serde_json::from_str(LOCAL_V2_LEDGER).unwrap();
    let row = &frozen["sessions"][0];
    let id = row["session_id"].as_str().unwrap();
    let store = SessionStore::with_persistence(&path, 10, 10);
    let restored = store
        .summary(id, Some(10))
        .expect("local v2 row must survive restore");
    assert_eq!(restored.project.as_deref(), row["project"].as_str());
    assert_eq!(restored.title.as_deref(), row["title"].as_str());
    assert_eq!(restored.events.len(), 1);
    assert_eq!(
        restored.events[0].event_id,
        row["events"][0]["event_id"].as_str().unwrap()
    );
    // Exact resume forces a current write; the frozen event and message remain.
    store
        .ensure_coding_session(request(Some(id.to_string())))
        .unwrap();
    store.flush_persistence();
    let rewritten = std::fs::read_to_string(&path).unwrap();
    assert!(!rewritten.contains("\"context_revision\""));
    let rewritten: serde_json::Value = serde_json::from_str(&rewritten).unwrap();
    let saved = &rewritten["sessions"][0];
    assert_eq!(saved["session_id"], row["session_id"]);
    assert_eq!(
        saved["owner_authority_fingerprint"],
        row["owner_authority_fingerprint"]
    );
    assert_eq!(saved["workspace_baseline"], row["workspace_baseline"]);
    assert_eq!(
        saved["messages"][0]["message_id"],
        row["messages"][0]["message_id"]
    );
    assert_eq!(
        saved["messages"][0]["message"],
        row["messages"][0]["message"]
    );
    assert_eq!(saved["events"][0]["event_id"], row["events"][0]["event_id"]);
}

fn request(resume_session_id: Option<String>) -> CodingSessionRequest {
    CodingSessionRequest {
        project: PROJECT.to_string(),
        authority_fingerprint: TEST_ONLY_PROJECT_SESSION_AUTHORITY_FINGERPRINT.to_string(),
        resume_session_id,
        instruction: Some("observe workspace".to_string()),
        mode: SessionMode::Normal,
        guards: SessionGuards::default(),
        execution_context: None,
        project_instructions: None,
        transport: SessionTransport::Api,
        context_refreshed: true,
        write_scope_verified: true,
    }
}

fn baseline(path: &str) -> WorkspaceBaseline {
    WorkspaceBaseline {
        project: PROJECT.to_string(),
        repository_key: ROOT_KEY.to_string(),
        head: Some(HEAD.to_string()),
        complete: true,
        files_total: Some(1),
        entries: vec![WorkspaceBaselineEntry {
            path: path.to_string(),
            status: "modified".to_string(),
        }],
    }
}

#[test]
fn workspace_baseline_is_immutable_across_resume_restart_and_missing_legacy() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("sessions.json");
    let store = SessionStore::with_persistence(path.clone(), 10, 10);
    let created = store
        .ensure_coding_session_with_workspace_baseline(
            request(None),
            None,
            Some(baseline("existing.txt")),
        )
        .unwrap();
    let id = created.summary.session_id.clone();
    assert_eq!(
        created.summary.workspace_baseline,
        Some(baseline("existing.txt"))
    );
    let generic_summary = serde_json::to_value(&created.summary).unwrap();
    assert!(
        generic_summary.get("workspace_baseline").is_none(),
        "generic Session projection must not serialize internal paths/root hash"
    );
    let resumed = store
        .ensure_coding_session_with_workspace_baseline(
            request(Some(id.clone())),
            None,
            Some(baseline("replacement.txt")),
        )
        .unwrap();
    assert_eq!(
        resumed.summary.workspace_baseline,
        Some(baseline("existing.txt"))
    );

    store.flush_persistence();
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(raw.contains("existing.txt"));
    assert!(!raw.contains("replacement.txt"));
    let restored = SessionStore::with_persistence(path, 10, 10);
    assert_eq!(
        restored.summary(&id, None).unwrap().workspace_baseline,
        Some(baseline("existing.txt"))
    );

    let legacy = restored.ensure_coding_session(request(None)).unwrap();
    assert!(legacy.summary.workspace_baseline.is_none());
    let legacy_resumed = restored
        .ensure_coding_session_with_workspace_baseline(
            request(Some(legacy.summary.session_id)),
            None,
            Some(baseline("late.txt")),
        )
        .unwrap();
    assert!(
        legacy_resumed.summary.workspace_baseline.is_none(),
        "missing original baseline must not be recaptured"
    );
}

#[test]
fn workspace_baseline_rejects_oversized_and_unsafe_restore_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("sessions.json");
    let store = SessionStore::with_persistence(path.clone(), 10, 10);
    let invalid = WorkspaceBaseline {
        entries: vec![WorkspaceBaselineEntry {
            path: "x".repeat(4_097),
            status: "modified".to_string(),
        }],
        ..baseline("safe.txt")
    };
    let created = store
        .ensure_coding_session_with_workspace_baseline(request(None), None, Some(invalid))
        .unwrap();
    let retained_baseline = created.summary.workspace_baseline.unwrap();
    assert!(!retained_baseline.complete);
    assert!(retained_baseline.entries.is_empty());
    store.flush_persistence();
    assert!(!std::fs::read_to_string(&path)
        .unwrap()
        .contains(&"x".repeat(4_097)));
    let restored = SessionStore::with_persistence(path, 10, 10);
    let retained = restored
        .summary(&created.summary.session_id, None)
        .unwrap()
        .workspace_baseline
        .unwrap();
    assert!(!retained.complete);
    assert!(retained.entries.is_empty());

    let mismatched = WorkspaceBaseline {
        project: "other-target".to_string(),
        ..baseline("safe.txt")
    };
    let created = restored
        .ensure_coding_session_with_workspace_baseline(request(None), None, Some(mismatched))
        .unwrap();
    assert!(created
        .summary
        .workspace_baseline
        .unwrap()
        .entries
        .is_empty());

    let unsafe_root = WorkspaceBaseline {
        repository_key: "z".repeat(50_000),
        ..baseline("safe.txt")
    };
    let created = restored
        .ensure_coding_session_with_workspace_baseline(request(None), None, Some(unsafe_root))
        .unwrap();
    let retained = created.summary.workspace_baseline.unwrap();
    assert!(retained.entries.is_empty());
    assert_eq!(retained.repository_key.len(), 64);
}
