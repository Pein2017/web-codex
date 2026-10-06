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
    db.upsert_job_archive(&replacement, now).unwrap();
    assert_eq!(db.load_job_archive("job", now).unwrap(), Some(original));
    db.conn_for_tests()
        .execute(
            "UPDATE wc_job_archives SET payload='{}' WHERE job_id='job'",
            [],
        )
        .unwrap();
    assert!(db.load_job_archive("job", now).is_err());
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
