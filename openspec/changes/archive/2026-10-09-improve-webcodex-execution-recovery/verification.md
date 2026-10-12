# Execution recovery implementation evidence

Status: functional native slice qualified; archive physical-quota and sandbox
positive gates remain HOLD. No candidate activation or all-five acceptance.

## Authority and baseline

- User authorized implementation, acceptance and the named local installation on
  2026-10-06. No public release, push/tag/version bump, research rerun, worker
  sandbox bypass or unrelated service retirement is included.
- Physical source: `/data/CoordExp/codex-tools/web-codex`, branch
  `coordexp/web-workflow`, baseline
  `d9005f204cfab4cfc4b7304ccd197fe31a4c7c63`. Initial tracked source was clean;
  only this approved OpenSpec change was untracked. Later source modifications
  belong to the assigned implementation workers.
- Authenticated baseline `runtime_status(client_id="coordexp-web-runner",
  compact=true)`: installed Server and Runner `0.4.3 / 004e108a7e5e`, clean,
  build alignment `exact`, protocol `compatible`, five online Projects, zero
  active/running/queued Jobs. Runner instance
  `28992bed-c888-4976-8c8e-bd4ea56b86bf`. Full trace remains enabled.
- Runtime/config/rollback evidence belongs to ignored `deployment/`; private
  configuration and raw output must not be committed or printed.
- Main `/data/CoordExp` has unrelated dirty hooks, agent instructions, agent
  integration documentation and two Pi-related OpenSpec changes. None belongs
  to this change; no staging, commit, restoration or cleanup there is authorized.
- Implementation ownership: selector/source fence, sandbox helper, lifecycle
  polling and Job archive have distinct writers. The lifecycle patch uses
  `gpt-6-luna` / max as the user's requested bounded implementation trial.
  Shared Cargo invocations have one coordinated owner.

## Lifecycle RED: actual producer-to-consumer reproduction

A read-only live `run_shell(command="pwd", shell="bash")` succeeded with
completed execution and exit zero (trace
`270f0763-45ba-4bfb-8d45-0c3b15aa8024`). The same harmless command using
nonexistent project-relative cwd
`.webcodex-lifecycle-readonly-missing-cwd-20261006` deterministically reproduced
`structured_job_lifecycle_invalid` / unknown outcome:

- Full trace `78e164b6-9ced-4243-a53a-da46a056239f`.
- Action event `96dc8023-fe36-46b3-a78a-bfecab0e0176`, duration 78 ms.
- Exactly one start request `ad56d11c-ce60-438c-acac-31c5bbe1ba0b` for
  Job `wc_job_BEyFqbL6otjF3bJC`.
- Sequenced native updates: sequence 1 `agent_queued`; sequence 2
  `failed / not_started`, duration zero and original cwd-access rejection.
  No `running` update or payload process was observed.
- Registry polling stamped `started_at` at dequeue for raw shell Jobs without
  structured metadata. The unchanged validator correctly rejected the later
  not-started update because the Server timestamp contradicted it.

This establishes a causal same-class producer correction. Historical errors
`8ed045de-c17c-486c-a83e-6adbcac504b2` and
`be981a06-aca4-4859-b053-2c3b6a6cff4b` still have no exact payloads; their
original cwd/arguments are not inferred from this prospective reproduction.
The narrow polling correction removes only the premature start timestamp; the
strict validator is unchanged. Baseline RED session `20826` exited 101 after
executing the named regression (`Some(started_at)` versus expected `None`).
Candidate GREEN session `24951` exited zero: both pre-start/no-redispatch and
the invalid converse (`running` then `not_started`) passed. The real disposable
dogfood caller in `native-recovery-2rmz4frw/receipt.json` and later receipts
returned truthful `not_started` and proved exactly one payload file append.

## Sandbox integration observation

The new non-model helper ran through the actual Runner `run_process` context
with explicit native Codex `0.159.2`, `:read-only` policy and managed
requirements. A host-readable checkout file and an independently created
private `/tmp` fixture both failed native mount setup:

- Trace `a05b3917-da7a-44f3-8f82-b571d3d2a888`: helper exit 2, native exit 1,
  diagnostic `native_mount_permission_denied`, 0.176 s, UID 0, mount namespace
  `4026540214`. Bounded diagnostic, no substantive worker or fallback.
- The actual Runner disposable-fixture harness verified the same blocked
  result in 0.132 s; its harness exit zero is not sandbox-read success.

Native positive-read qualification is HOLD under this host restriction. The
user explicitly chose on 2026-10-06 to retain the original positive gate,
finish the code, and **not install**. This supersedes activation authority for
this turn: no live application replacement or service restart will occur.
Controlled helper tests cannot replace native positive evidence. Local code,
disposable consumer checks and delivery records may proceed; tasks 7.3 and
7.4 remain pending for a later explicit installation request.

## Current focused consumer evidence

- Production-shaped isolated Server/Runner builds completed: Cargo dogfood
  session `7875`, exit zero. The early debug Server overflow and readiness/API
  fixture failures are retained, not reported as acceptance. A new task-local
  dispatch future increased default-stack pressure; a narrow inner-future box
  fixed the reproduced test overflow without stack-environment overrides.
- Selectors and source fences: Cargo session `24957` exit zero, 18 Work Result
  tests. Same compiled artifact: 13 source-fence, 11 validation-identity and six
  exact rollback/closeout/negative tests, all exit zero (48 total). The selector
  negative found implicit presentation recording (`10` events versus `0`);
  the presentation-only business-recorder suppression now passes, while explicit
  Session target authorization and independently explicit recording remain.
- Deployment helper/assets: unittest session `88497`, exit zero, 23 tests.
  Controlled positives are not native sandbox acceptance.
- Real model archive integration exposed a mismatched assumption: the admitted
  `project_cwd` is `.` while actual `cwd` is absolute. The capture/read gates
  have been corrected to bind trusted registry root and native incarnation;
  native paging/restart requalification remains pending. Archive metadata also
  now requires a typed fresh Runner root proof; timeout/malformed/legacy replies
  cannot disclose metadata while online.
- The rebuilt native archive slice (`native-recovery-jr7b5mvz/receipt.json`)
  subsequently exposed legitimate coalesced stdout (213,400 bytes) being discarded
  by a 64-KiB single-chunk guard: terminal exit remained zero, but retained
  prefix was zero with explicit backpressure. The writer must split a bounded
  UTF-8 prefix into existing capped queue messages; slowing the fixture would
  not retire this production-shaped gap. A bounded eight-message, UTF-8-safe
  nonblocking prefix split now replaces that assumption. Runner session `14773`
  exited zero (12 focused tests), including the single coalesced append, a forced
  full queue and the bounded eight-chunk ceiling. The subsequent real native
  receipt below retires the 213,400-byte prefix-loss counterexample.
- Archive focused tests: Runner session `95441` exit zero (12 tests, 10 owned),
  Store `28335` exit zero (2), registry archive `64059` exit zero (2), affected
  raw-shell/lifecycle 4 and receipts 16, all passed. A pre-existing cfg-only
  AtomicUsize `try_update` failed stable-1.93 test compilation before execution;
  replacing it with equivalent stable `fetch_update` is a test-only correction,
  not archive RED or a production behavior change.
- Tool-contract package session `52000`: exit zero, 236 tests, including parser,
  schema privacy/bounds and unchanged tool-surface admission. Descriptor length
  and semantics failures were corrected without weakening the budget/checks.
- Root Job reader/session consumers: compile/test session `96125` exit zero
  (9 internal ObserveJobs tests), then the same current artifact session `81370`
  exit zero (38 public-runtime ObserveJobs tests). The earlier output-schema
  fixture assumed an object instead of the new normal/minimal-fact `anyOf`;
  the corrected fixture checks both branches and exact three-field closure.
- Final source deployment tests session `51670`: exit zero, 23 tests in 2.881 s.
- Final scoped Runner failure controls session `96032`: exit zero, 15 focused
  tests. Real OS `EBADF` on a read-only regular output fd produces the explicit
  `storage_failure` reason while stderr remains exact; the JobManager preserves
  terminal status, exit zero and live output. Joined pre-commit owner loss,
  rejected `create_new(terminal.tmp)` and malformed final receipt cannot publish
  terminal proof. This is bounded write/crash-boundary evidence, not actual
  ENOSPC or power-cut filesystem testing.
- A final real continuation RED (`native-recovery-4z_r71a0/receipt.json`, session
  `77914`, exit 1) showed automatic archive selection repeating terminal output
  instead of the retained ordinary token delta. The correction keeps ordinary
  known-Job token reads on their native owner, selects archives for explicit
  pages/historical lookup, and parses tokens before archive or offline-fact
  recovery. Historical token recovery reports a reset, not live delta continuity.
  New receipt/token tests session `1110`: exit zero, 17 tests. Final root reader
  tests session `78986`: exit zero, 51 tests.
- The same current registry artifact's existing token/wait/delta module session
  `23307` exited zero, 17 tests. An earlier wrong module-name filter selected
  zero tests and is not acceptance evidence.
- Final live readback still shows installed `0.4.3 / 004e108a7e5e`, clean exact
  Server/Runner, the original Runner instance and full trace, zero Jobs. Project
  count is now 15 (the earlier baseline was 5); no Project registration was
  performed by this change. External runtime changes were preserved.

## Disposable native qualification

Dogfood Server/Runner/CLI builds `64829` and final corrected `37790` exited zero.
The successful full native invocation `89672` exited zero with receipt
`deployment/verification/execution-recovery-20261006/native-recovery-285x6l3a/receipt.json`.
It used only private generated CPU fixtures and 100 tool calls, never a live
Project or original research Job. All eight owned process exits were zero.

- Truthful missing-cwd `not_started`, strict invalid-transition rejection in
  focused tests, and exactly one native payload file append.
- All three authorized MCP Work Result selectors selected the same canonical
  explicit Session. Actual `cargo_check` and rejected revision-999 guarded edit
  preserved source bytes and the conservative `unproven` validation meaning.
- Same original Job recovered all 213,400 bytes across 11 progressing model
  pages; cursor reached 2201 without loss. Seventy additional terminal Jobs,
  transient Runner instance replacement, Server restart and explicitly simulated
  age of 25 hours did not lose its exact archived prefix. Ordinary receipt
  startup pruning was forced on the private database only.
- Repeated ordinary terminal-token observation returned `unchanged` with no
  repeated output. Malformed tokens failed both online historical lookup and
  offline minimal-fact recovery. After fresh Server/expired ordinary receipt
  recovery, the original valid token explicitly returned `reset`/`changed=true`.
  Runner inventory eviction alone is not Server observation-owner removal; the
  corrected fixture tests historical reset only at the actual missing-owner seam.
- Fresh Server/offline Runner returned exactly authenticated `job_id`, terminal
  `status` and recorded `exit_code`, with no output/metadata or read dispatch.
  Foreign credentials returned one failed item and zero successful items, with
  null output. An outer successful batch is not per-Job authorization: the first
  fixture misread that envelope and was corrected without relaxing disclosure.
- Online same-path/new-root-inode replacement denied metadata and bytes; current
  Session restoration succeeded before the deliberate root-replacement control.
- Native sandbox probe: host-readable fixture, native exit 1/helper exit 2,
  `native_mount_permission_denied`, 0.078 s, no worker and no fallback. This is
  negative/fail-closed evidence only; original positive gate remains HOLD.
- Retained previous Server **and Runner** read candidate-written Session state
  and performed their ordinary pruning without database restoration. All new
  archive locator IDs remained unchanged. The first offline old-Server fixture
  was rejected by its existing Project-authorization gate; the qualified check
  restores the current Project using the retained old pair, not an auth bypass
  or mixed-version-wire substitute.

Reproduction command (relative paths are from the verified WebCodex checkout):

```sh
python3 deploy/web-workflow/tests/execution_recovery_smoke.py \
  --server target/dogfood/webcodex-server \
  --runner target/dogfood/webcodex-runner \
  --previous-server deployment/app/bin/webcodex-server \
  --previous-runner deployment/app/bin/webcodex-runner \
  --evidence-root deployment/verification/execution-recovery-20261006 \
  --helper deploy/web-workflow/sandbox_preflight.py
```

The harness explicitly labels both physical-quota and sandbox-positive HOLDs;
its exit zero is functional-slice qualification, not release acceptance.

## Physical quota gap (advisor-checked)

The initial 32-KiB-per-terminal Server allowance does not prove the declared
1-GiB physical bound: a second SQLite connection can pin a WAL snapshot while
unlimited distinct archive commits append page versions despite 256 retained
rows. Runner file eviction does not reclaim Server row/WAL bytes. Current shared
SQLite WAL code is therefore HOLD at this named boundary, not logical-quota PASS.
A user question proposes an independently hard-capped archive-locator SQLite
file under the same managed owner, reusing the existing receipt interface and
leaving Session/ordinary receipts unchanged. Until answered, no new store or
reinterpreted bound is authorized.

## Acceptance ledger

| Boundary | Current status | Remaining consumer evidence |
| --- | --- | --- |
| Lifecycle | PASS on named producer/caller seam | no remaining named functional seam; activation deferred |
| Durable Job archive | HOLD: paging/restart/auth/downgrade/failure functional seams PASS | hard aggregate physical quota |
| Work Result selectors | PASS: focused GREEN and real three-selector MCP check | activation deferred |
| No-write source proof | PASS: focused GREEN and real guarded rejection/CPU validation | activation deferred |
| Native preflight | HOLD: real fail-closed detection verified | supported native setup/read positive gate; activation deferred |

## OpenSpec verification report

Using `openspec-verify-change` for this existing change, with all proposal,
design, task and delta-spec artifacts loaded. This maps the current evidence;
it does not restart the bounded review or claim archive/release readiness.

| Dimension | Result |
| --- | --- |
| Completeness | 13/21 tasks checked; eight tasks remain explicitly incomplete |
| Correctness | Six requirements mapped: three qualified, three partial/deferred |
| Coherence | Existing Job/receipt/resolver/source owners retained; no new model tool, bypass, retry, task store or alias |

Requirement mapping:

- Lifecycle: `polling.rs:698`, unchanged lifecycle validator, exact RED/GREEN
  and native single-effect/pre-start controls.
- Durable Job recovery: native `job_archive.rs:305` / `:388`, registry
  `archive.rs:28` / `:69`, existing SQLite receipt adapter. Paging, identity,
  root/auth/restart and failure controls qualified; physical aggregate bound
  remains unimplemented/unqualified in the shared WAL design.
- Work Result resolution: `work_result.rs:130`, dispatch presentation-recorder
  suppression, selector/negative fixtures and the real MCP three-selector slice.
- Validation truth: `validation_source.rs:34` / `:137` / `:161`, pre-first-write
  rejection constructors and retained rollback/concurrency/closeout consumers.
- Native preflight: `deploy/web-workflow/sandbox_preflight.py`, 13 helper
  controls and actual Runner blocked probe; native positive-read scope HOLD.
- Reproducible delivery: source templates/assets, actual matching dogfood build
  and disposable previous-pair state read/prune qualify the current candidate
  seams only. Complete clean package and live delivery remain deferred.

CRITICAL for final completion/archive (not a finding that authorizes extra work):

- Task 2.1: implement and qualify the hard physical aggregate bound after the
  pending storage-architecture ruling; retained rows/payload budgets are not it.
- Task 2.3: rerun quota/expiry/access acceptance against that final storage owner.
- Task 5.2: obtain real supported native setup/read success in the actual Runner
  context; do not replace the retained positive gate with a negative control.
- Task 5.3: qualify complete packaged helper/guidance assets and their consumer
  handoff after candidate freeze; source-only checks do not update the live app.
- Task 7.1: build/package the final clean committed candidate with exact complete
  asset identity once the physical storage design is settled.
- Task 7.2: apply downgrade qualification to that exact final candidate; current
  shared-database result is not reusable after a storage architecture change.
- Task 7.3: require renewed explicit installation authority and cleared gates.
- Task 7.4: perform real post-activation routing/identity/state readback only
  after that authorized installation; do not claim it from isolated fixtures.

No additional blocking style or speculative hardening findings were added.
Final assessment: not ready for archive or installation; keep the declared
HOLDs and user no-install override. Planning CLI `isComplete=true` means its
four artifacts exist, not that these implementation tasks or gates passed.

The reviewed source delivery is a local functional candidate only: owned Rust
formatting, whitespace and strict OpenSpec validation passed. The local commit
containing this evidence owner records the implementation and its explicit
HOLDs, not an installable release. No push, tag or upstream version bump is
authorized or performed.

Clean-identity complete dogfood package, activation, live smoke and rollback
locator remain pending. Downgrade qualification applies
to the current shared-database candidate only and must be repeated if the
pending physical-quota architecture ruling changes its storage. No existing
database was restored.

## Revised direct-only candidate (2026-10-06; current qualification)

The preceding sections remain the historical `f78ce3d2` report. The user subsequently removed native Codex readiness from scope and authorized push to the existing fork branch plus local installation, not a public Release. That resolves authority, not technical acceptance. No Codex CLI is invoked by the new workflow or qualification.

### Independent storage and physical quota

- Fixed private Server owner: `<state-db-parent>/job-archive-locator/archives.sqlite3`. Session and ordinary receipt WAL are unchanged; no shared-database archive fallback or migration exists for the never-installed candidate table. The existing JobReceiptStore API delegates to this lazy independent connection; archive failure does not prevent ordinary Database startup.
- Joint budget remains 1 GiB. Charge a fixed 16-MiB Server reservation independent of Runner deletion; restrict its database to 4096 × 1024 pages (4 MiB), DELETE rollback journal, FULL synchronization, memory temporary storage and disabled cache spilling. Read back each opened writer's policy, reject incompatible/oversized/WAL/SHM/unowned/replaced state, preserve immutable admission identity/deadline. The 256-row and seven-day values are ceilings, not availability promises.
- Runner allowance is 1008 MiB across all owned namespaces. On the qualified Linux filesystem, prepay two 16-MiB streams plus 1 MiB structural allowance before unlocked writes. Charge recognized incomplete creation fully; exact rollback closes handles. Trim, sync and close output and receipt handles before reservation release and eviction. Unsupported allocation/preallocation fails archive capture closed without changing process exit.
- Read-only `ask-advisor` follow-up identified the retained-reader WAL mechanism and the open-unlinked writer credit defect. Its ACCEPT_WITH_CORRECTIONS was design advice, not acceptance. The finite journal bound follows the pinned bundled SQLite restricted API and the named filesystem; it is not a universal arbitrary-filesystem claim.

Commands used a command-local `TMPDIR=deployment/verification/execution-recovery-20261006/physical-tmp.fYv0Zq` on the actual `/data` Linux filesystem (4096-byte allocation):

- `cargo test --locked -p webcodex-store --lib job_receipts_tests -- --nocapture --test-threads=1`: 13 passed, one explicitly ignored subprocess helper, exit 0. Full store `--lib`: 226 passed, that helper ignored, exit 0. The helper is directly executed by the crash test, not omitted qualification.
- Original-policy counterexample: 256 bounded rows but 66,342,912 allocated WAL bytes under a retained reader. Final store: actual SQLITE_FULL at additional insertion 245 after the original maximum-size row; original remained readable. Continuous allocated-byte sampling plus explicit in-transaction all-page journal inspection peaked at 8,380,416 bytes, below 16 MiB. Real child-only sync interception exited during COMMIT after a valid hot-journal header; reopening recovered the exact original. The first cache-flush fixture lacked hot magic and failed the strong assertion; it was replaced, not accepted as crash evidence.
- `cargo test -p webcodex-runner --bin webcodex-runner archive -- --nocapture`: 21 passed, exit 0, preserving existing native child-exit/storage-failure checks. The nearest-wrong original handle order failed the gate regression, exit 101, with three live output/receipt handles. Candidate had zero; prepaid bytes 33,558,528 remained constant after stream writes and trimmed to 12,288 on commit. Near-quota eviction preserved the other namespace's 33,558,528 active bytes.
- Existing structured-process and raw-shell terminal lifecycle consumers each passed once, exit 0. A missing archive-capability field in a core test fixture blocked compilation; adding only its false default restored it. Lead's full `cargo test --locked -p webcodex-core --lib`: 294 passed, exit 0; registry archive filter: three passed, exit 0. A previous zero-match core archive filter was compilation evidence only.
- `python3 -m unittest discover -s deploy/web-workflow/tests -p test_relocation.py -v`: ten passed, exit 0. Missing-helper startup regression was RED before retirement. Removed the helper and dedicated tests; installed startup and guidance retain ordinary Project/source/Session and process-vs-task-acceptance fences.
- `node --test plugins/web-workflow/plugin.test.mjs`: 29 passed, exit 0. Strict OpenSpec validation, Python syntax and Rust formatting checks passed.

Real revised Server/Runner integration, clean package identity, old-app pruning/readback, push, installation and live readback are still pending at this source-candidate checkpoint. Historical results do not close those gates.

### First revised native consumer qualification

`deployment/verification/execution-recovery-20261006/native-recovery-mnsfrxz2/receipt.json`: `candidate_native_direct_recovery_passed`, script exit 0; 107 real tool calls. Native Server/Runner source `c7d8b4b2d033`, clean, exact build match. Ten owned service children exited 0; the eleventh was the deliberately SIGKILLed disposable Runner (exit -9), not an unexplained service failure; its native payload was independently confirmed no longer running. No installed service or research workspace was mutated.

The direct guarded read/edit/check used the actual retained Session event's exit code 0. An earlier fixture incorrectly expected `exit_code` in cargo_check's intentionally compact model projection and failed; `native-recovery-3vwxgen3/receipt.json` is retained. That was fixed by asserting the native recorder field, not deleting the exit check or changing runtime output.

All revised caller seams passed: prestart rejection without invalid lifecycle; one side effect; authorized canonical/reference/short-name Work Result; pre-first-write guard rejection preserving unproven source; stale-revision and parent-cwd policy denial; full 213400-byte decoded output paged beyond the live tail; same terminal token unchanged/no repeated output; 70-Job inventory eviction; Runner and Server restart; simulated 25-hour ordinary receipt expiry; offline exact three-field terminal projection; malformed-token and foreign-auth rejection; same pathname/new inode refusal; current Session restoration; retained previous Server/Runner Session read and ordinary receipt pruning; unchanged independent locator IDs; final candidate reading the original archive after the old pair with no state restore. Codex CLI attempts stayed zero throughout. Losing a running Job's owner before terminal commit did not create a recovered verdict or repeat its one payload effect.

The final clean committed harness/package build and its matching native readback remain the activation gate. Push and live installation are not inferred from this isolated success.

### Final package, publication and installed acceptance

Final application source: `cb16155cd80a67b1acc7ac6d658bf0064d1144cf`, branch `coordexp/web-workflow`. Matching Server, Runner and WebCodex CLI built with the dogfood profile, both requested workspace-checkpoints features, Rust 1.93.0, Linux x86_64; embedded commit `cb16155cd80a`, dirty=false, built_at=1791310771. The complete direct-workflow assets and native checksums are in ignored `deployment/app/RELEASE.json`; no helper, compatibility link or alias is installed. Version remains upstream 0.4.3. No Codex CLI agent was invoked.

Exact final-binary native receipt: `deployment/verification/execution-recovery-20261006/native-recovery-q3elum5a/receipt.json`, script exit 0, 107 real calls, `candidate_native_direct_recovery_passed`. This repeats the independent-locator candidate-write/previous-pair Session read and pruning/final-candidate original archive read before activation, without a database restore. Prior receipt/header/projection failures remain untouched.

Publication: fetched only the existing origin branch and reviewed its range (99 final changed paths), whitespace and non-disclosing credential/runtime-state checks. `Pein2017/webcodex` is public, default main; only `coordexp/web-workflow` was pushed. Remote branch resolved exactly to the requested source after push. No tag, version bump, GitHub Release/npm publication, upstream merge or sibling branch mutation. A final documentation-only evidence commit updates this report/checklist and is pushed separately; the installed application source remains the exact clean commit above.

Activation: authenticated runtime active/running/queued Jobs were 0/0/0. Revalidated the exact named service binaries/PIDs, stopped only the dedicated deployment tmux session, confirmed all prior Server/Runner/WebCodex-CLI/Tunnel process IDs exited, and saved a consistent private Server/config/Project-registry copy plus prior application and operator entrypoints at `deployment/rollback/execution-recovery-20261006-cb16155c/`. Replaced the one real `app/` directory, refreshed its real operator entrypoints, and started that same named deployment. No state restoration or shared/research service stop.

Installed receipts: `deployment/verification/execution-recovery-20261006/installed-runtime-status.json` and `installed-acceptance.json`. Authenticated Tunnel MCP returned Server/Runner 0.4.3/cb16155cd80a clean, exact build/source match, compatible protocol, native archive capability true, no coding-agent providers, 15/15 online Projects and 0/0/0 Jobs after tests. Canonical root/infra/research paths are unchanged. All 100 old Session IDs and private Server/Runner/Tunnel config bytes match the stopped backup; an original refactor Session successfully restored through the gateway.

Real installed Runner Python resolved `/root/miniconda3/envs/ms/bin/python`. A new bounded native Job `wc_job_-cgKCkY7ob2Ngf84` completed exit 0 and its exact committed archive was read through the admitted gateway, cursor 2, no loss and archive_unavailable=false. Job `wc_job_Dbbrwdt9z1CU77Mo` ran the ten relocation tests through the actual installed Runner, observed the same Job to exit 0, and produced `installed-relocation.xml`. Native Plugin describe/binding/call parsed that actual JUnit report as 10 passed/0 failures/0 errors/0 skips, testsExecuted=false as required (the parser does not pretend to run tests). All four providers are ready; configured git-hygiene Skill load passed. Tool manifest now has 144 model-visible tools; the authenticated current MCP descriptor has 41 direct tools and the gateway supplies the remainder. Full trace remains full with unchanged 168-hour/2147483648-byte limits.

Host limitation, not hidden as PASS: this already-running Codex/ChatGPT-side tool declaration lacks observe_jobs' new since_stdout_line/since_stderr_line fields. Current authenticated Server tools/list demonstrably includes both fields, and exact gateway archive paging succeeded. Refresh/reload the WebCodex connector's tool definition to expose them directly; discovery alone cannot register them and starting a new chat is not proof of a refreshed descriptor. Existing direct coding tools and the generic admitted gateway remain usable. This does not require reinstallation or a new repository/connector identity.

The user-requested `codex-tools/pi-web-worktrees` parent was empty, not mounted, had no registered worktrees or process cwd/FD references; exact rmdir succeeded. Pi Web's actual checkout/worktree list and unrelated dirty work were preserved. No broad cleanup.

All 21 revised tasks are complete. This qualified named-Linux deployment does not claim universal filesystem bounds, power-loss testing, all unrelated long-tail functions, or retrospective diagnosis of the two untraced historical command arguments. The former native sandbox failure is historical/removed scope, not a successful native sandbox test.
