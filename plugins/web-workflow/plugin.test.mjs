import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const PROTOCOL_VERSION = "webcodex-plugin-v1";
const here = path.dirname(fileURLToPath(import.meta.url));
const pluginPath = path.join(here, "plugin.mjs");
const ADMITTED_SCHEMA_KEYS = new Set([
  "type",
  "title",
  "description",
  "properties",
  "required",
  "additionalProperties",
  "enum",
  "const",
  "minLength",
  "maxLength",
  "minItems",
  "maxItems",
  "items",
]);

function tempRoot() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "webcodex-web-workflow-"));
}

function writeExecutable(target, source) {
  fs.writeFileSync(target, source, { mode: 0o755 });
  fs.chmodSync(target, 0o755);
}

function createFixture() {
  const root = tempRoot();
  const project = path.join(root, "project");
  const memory = path.join(root, "memory");
  const history = path.join(root, "history");
  const outside = path.join(root, "outside");
  fs.mkdirSync(path.join(project, "reports"), { recursive: true });
  fs.mkdirSync(path.join(project, "src"), { recursive: true });
  fs.mkdirSync(path.join(project, "tests"), { recursive: true });
  fs.mkdirSync(path.join(memory, "rollouts"), { recursive: true });
  fs.mkdirSync(history, { recursive: true });
  fs.mkdirSync(outside, { recursive: true });
  fs.writeFileSync(path.join(project, "src", "alpha.ts"), "export function alpha() {}\n");
  fs.writeFileSync(path.join(project, "src", "Beta.ts"), "export class Beta {}\n");
  fs.writeFileSync(path.join(project, "src", "source256.ts"), "export function source256() {}\n");
  fs.mkdirSync(path.join(project, "workers"), { recursive: true });
  fs.writeFileSync(path.join(project, "workers", "worker.ts"), "export function source256Worker() {}\n");
  fs.writeFileSync(path.join(project, "tests", "alpha.test.ts"), "test('alpha', () => {});\n");
  fs.writeFileSync(path.join(memory, "MEMORY.md"), "alpha memory line\nsecond line\n" + "z".repeat(200));
  fs.writeFileSync(path.join(memory, "rollouts", "one.jsonl"), '{"note":"alpha rollout"}\n');
  fs.writeFileSync(
    path.join(history, "thread-001.jsonl"),
    [
      JSON.stringify({
        type: "response_item",
        thread_id: "thread-001",
        id: "msg-1",
        payload: { type: "message", role: "user", content: [{ type: "input_text", text: "Synthetic source256 question" }] },
      }),
      JSON.stringify({
        type: "response_item",
        thread_id: "thread-001",
        id: "analysis-1",
        payload: { type: "analysis", role: "assistant", content: [{ type: "text", text: "Synthetic private reasoning" }] },
      }),
      JSON.stringify({
        type: "event_msg",
        thread_id: "thread-001",
        payload: { type: "agent_message", id: "msg-2", phase: "commentary", message: "Synthetic api_key=sk-proj-testnotreal123456" },
      }),
      JSON.stringify({
        type: "response_item",
        thread_id: "thread-001",
        id: "msg-2",
        payload: { type: "message", role: "assistant", phase: "commentary", content: [{ type: "output_text", text: "Synthetic api_key=sk-proj-testnotreal123456" }] },
      }),
      JSON.stringify({
        type: "response_item",
        thread_id: "thread-001",
        id: "tool-1",
        payload: { type: "function_call", role: "assistant", name: "read_file", arguments: "Synthetic tool payload" },
      }),
      JSON.stringify({
        type: "event_msg",
        thread_id: "thread-001",
        payload: { type: "user_message", id: "msg-3", message: "Synthetic follow-up" },
      }),
      JSON.stringify({
        type: "system",
        thread_id: "thread-001",
        id: "hidden-system",
        payload: { type: "message", role: "assistant", phase: "final", content: "Synthetic hidden system content" },
      }),
    ].join("\n") + "\n",
  );
  fs.writeFileSync(path.join(outside, "secret.md"), "alpha must not escape\n");
  fs.symlinkSync(path.join(outside, "secret.md"), path.join(memory, "linked-secret.md"));

  const codegraphEntry = path.join(root, "codegraph-entry.js");
  fs.writeFileSync(codegraphEntry, "// fake CodeGraph entry marker\n");
  const codegraphRuntime = path.join(root, "fake-codegraph-runtime.mjs");
  writeExecutable(
    codegraphRuntime,
    `#!/usr/bin/env node
const argv = process.argv.slice(2);
if (argv[0] !== "--liftoff-only" || argv[1] !== "--disable-warning=ExperimentalWarning") process.exit(9);
const command = argv.slice(3);
if (command[0] === "status") {
  process.stdout.write(JSON.stringify({
    initialized: true,
    version: "1.6.0-test",
    lastIndexed: "2026-09-15T00:00:00.000Z",
    pendingChanges: { added: 0, modified: 1, removed: 0 },
    worktreeMismatch: null,
    index: { state: "complete", reindexRecommended: false }
  }));
} else if (command[0] === "query") {
  let nodes = [
    { node: { name: "alpha", qualifiedName: "alpha", kind: "function", filePath: "src/alpha.ts", language: "typescript", startLine: 1, endLine: 1 }, score: 10 },
    { node: { name: "Beta", qualifiedName: "Beta", kind: "class", filePath: "src/Beta.ts", language: "typescript", startLine: 1, endLine: 1 }, score: 9 },
    { node: { name: "alpha test", qualifiedName: "alpha test", kind: "function", filePath: "tests/alpha.test.ts", language: "typescript", startLine: 1, endLine: 1 }, score: 8 }
  ];
  if (command[command.length - 1] === "source256") nodes = [
    { node: { name: "source256", qualifiedName: "source256", kind: "function", filePath: "src/source256.ts", language: "typescript", startLine: 1, endLine: 1 }, score: 11 },
    { node: { name: "source256Worker", qualifiedName: "source256Worker", kind: "function", filePath: "workers/worker.ts", language: "typescript", startLine: 1, endLine: 1 }, score: 10 }
  ];
  const kindAt = command.indexOf("--kind");
  if (kindAt >= 0) nodes = nodes.filter((item) => item.node.kind === command[kindAt + 1]);
  process.stdout.write(JSON.stringify(nodes));
} else {
  process.exit(8);
}
`,
  );
  const env = {
    ...process.env,
    WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON: JSON.stringify({ fixture: project }),
    WEBCODEX_WEB_WORKFLOW_MEMORY_ROOT: memory,
    WEBCODEX_WEB_WORKFLOW_HISTORY_ROOT: history,
    WEBCODEX_WEB_WORKFLOW_CODEGRAPH_RUNTIME: codegraphRuntime,
    WEBCODEX_WEB_WORKFLOW_CODEGRAPH_ENTRY: codegraphEntry,
    WEBCODEX_WEB_WORKFLOW_PYTHON: process.env.PYTHON || "python3",
  };
  return { root, project, memory, history, outside, env };
}

async function runProtocol(env, requests) {
  const child = spawn(process.execPath, [pluginPath], {
    stdio: ["pipe", "pipe", "pipe"],
    windowsHide: true,
    shell: false,
    env: { ...env, NODE_NO_WARNINGS: "1" },
  });
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (chunk) => {
    stdout += chunk;
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  for (const request of requests) child.stdin.write(`${JSON.stringify(request)}\n`);
  child.stdin.end();
  const exitCode = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      child.kill();
      reject(new Error("web-workflow protocol test timed out"));
    }, 15_000);
    child.once("exit", (code) => {
      clearTimeout(timer);
      resolve(code);
    });
  });
  assert.equal(exitCode, 0, stderr);
  assert.equal(stderr, "");
  const lines = stdout.split(/\r?\n/u).filter(Boolean);
  assert.equal(lines.length, requests.length, `unexpected protocol output: ${stdout}`);
  return lines.map((line) => JSON.parse(line));
}

function initialize(id = 1) {
  return { jsonrpc: "2.0", id, method: "initialize", params: { protocolVersion: PROTOCOL_VERSION } };
}

function listTools(id = 2) {
  return { jsonrpc: "2.0", id, method: "tools/list", params: {} };
}

function call(name, arguments_, id = 2) {
  return { jsonrpc: "2.0", id, method: "tools/call", params: { name, arguments: arguments_ } };
}

function assertAdmittedSchemaSubset(schema, location) {
  for (const keyword of Object.keys(schema)) {
    assert.ok(ADMITTED_SCHEMA_KEYS.has(keyword), `${location} uses unsupported schema keyword ${keyword}`);
  }
  if (schema.properties !== undefined) {
    for (const [name, child] of Object.entries(schema.properties)) {
      assertAdmittedSchemaSubset(child, `${location}.properties.${name}`);
    }
  }
  if (schema.items !== undefined) assertAdmittedSchemaSubset(schema.items, `${location}.items`);
  if (schema.type === "integer") {
    assert.match(schema.description, /Runtime accepts integers from .* through .* inclusive\./u, location);
  }
}

test("actual Plugin protocol exposes only configured bounded tools", async () => {
  const fixture = createFixture();
  try {
    const responses = await runProtocol(fixture.env, [initialize(), listTools()]);
    assert.equal(responses[0].result.protocolVersion, PROTOCOL_VERSION);
    assert.deepEqual(
      responses[1].result.tools.map((tool) => tool.name),
      ["pytest_report_summary", "memory_search", "memory_read", "codegraph_scoped_query", "public_history_read"],
    );
    for (const tool of responses[1].result.tools) {
      assert.deepEqual(tool.annotations, {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      });
      assert.equal(tool.inputSchema.additionalProperties, false);
    }
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("tool schemas stay within the current Native Plugin admission subset", async () => {
  const fixture = createFixture();
  try {
    const [response] = await runProtocol(fixture.env, [listTools()]);
    for (const tool of response.result.tools) {
      assertAdmittedSchemaSubset(tool.inputSchema, `${tool.name}.inputSchema`);
      if (tool.outputSchema !== undefined) {
        assertAdmittedSchemaSubset(tool.outputSchema, `${tool.name}.outputSchema`);
      }
    }
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("runtime validators retain numeric bounds and CodeGraph kind restrictions", async () => {
  const fixture = createFixture();
  try {
    fs.writeFileSync(path.join(fixture.project, "reports", "junit.xml"), "<testsuite />");
    const responses = await runProtocol(fixture.env, [
      call(
        "pytest_report_summary",
        { project: "fixture", path: "reports/junit.xml", maxFailures: 0 },
        1,
      ),
      call("memory_search", { query: "alpha", maxResults: 51 }, 2),
      call("memory_read", { path: "MEMORY.md", offset: -1 }, 3),
      call("memory_read", { path: "MEMORY.md", maxBytes: 63 }, 4),
      call("codegraph_scoped_query", { project: "fixture", search: "alpha", limit: 51 }, 5),
      call("codegraph_scoped_query", { project: "fixture", search: "alpha", kind: "--help" }, 6),
    ]);
    assert.deepEqual(
      responses.map((response) => ({
        isError: response.result.isError,
        errorCode: response.result.structuredContent.errorCode,
      })),
      Array.from({ length: 6 }, () => ({ isError: true, errorCode: "invalid_arguments" })),
    );
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("pytest summary preserves pass, failure, error, and skip counts without claiming execution", async () => {
  const fixture = createFixture();
  try {
    const report = `<?xml version="1.0"?>
<testsuites><testsuite name="suite">
  <testcase classname="pkg.test_mod" name="test_pass" />
  <testcase classname="pkg.test_mod" name="test_fail"><failure message="assert 1 == 2">trace</failure></testcase>
  <testcase classname="pkg.test_mod" name="test_error"><error message="fixture error">trace</error></testcase>
  <testcase classname="pkg.test_mod" name="test_skip"><skipped message="not supported" /></testcase>
</testsuite></testsuites>`;
    fs.writeFileSync(path.join(fixture.project, "reports", "junit.xml"), report);
    const [response] = await runProtocol(fixture.env, [
      call("pytest_report_summary", { project: "fixture", path: "reports/junit.xml", maxFailures: 1 }),
    ]);
    assert.equal(response.result.isError, false);
    assert.equal(response.result.structuredContent.testsExecuted, false);
    assert.equal(response.result.structuredContent.parserStatus, "parsed");
    assert.deepEqual(response.result.structuredContent.counts, {
      total: 4,
      passed: 1,
      failed: 1,
      errors: 1,
      skipped: 1,
    });
    assert.deepEqual(response.result.structuredContent.failures, [
      { identity: "pkg.test_mod::test_fail", kind: "failure", message: "assert 1 == 2" },
    ]);
    assert.equal(response.result.structuredContent.failuresTruncated, true);
    assert.deepEqual(response.result.structuredContent.source, {
      project: "fixture",
      root: fs.realpathSync(fixture.project),
      path: "reports/junit.xml",
    });
    assert.match(response.result.content[0].text, /no tests were executed/u);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("pytest summary rejects malformed, unsafe, and oversized reports explicitly", async () => {
  const fixture = createFixture();
  try {
    fs.writeFileSync(path.join(fixture.project, "reports", "malformed.xml"), "<testsuite><testcase>");
    fs.writeFileSync(path.join(fixture.project, "reports", "not-junit.xml"), "<hello />");
    fs.writeFileSync(
      path.join(fixture.project, "reports", "unsafe.xml"),
      '<!DOCTYPE x [<!ENTITY y "boom">]><testsuite />',
    );
    fs.writeFileSync(
      path.join(fixture.project, "reports", "unsafe-utf16.xml"),
      Buffer.from(
        '<?xml version="1.0" encoding="UTF-16"?><!DOCTYPE testsuite [<!ENTITY x "expanded">]><testsuite><testcase name="&x;"/></testsuite>',
        "utf16le",
      ),
    );
    fs.writeFileSync(path.join(fixture.project, "reports", "large.xml"), Buffer.alloc(4 * 1024 * 1024 + 1, 32));
    const responses = await runProtocol(fixture.env, [
      call("pytest_report_summary", { project: "fixture", path: "reports/malformed.xml" }, 1),
      call("pytest_report_summary", { project: "fixture", path: "reports/not-junit.xml" }, 2),
      call("pytest_report_summary", { project: "fixture", path: "reports/unsafe.xml" }, 3),
      call("pytest_report_summary", { project: "fixture", path: "reports/unsafe-utf16.xml" }, 4),
      call("pytest_report_summary", { project: "fixture", path: "reports/large.xml" }, 5),
    ]);
    assert.equal(responses[0].result.isError, true);
    assert.equal(responses[0].result.structuredContent.errorCode, "malformed_xml");
    assert.equal(responses[1].result.structuredContent.errorCode, "not_junit_xml");
    assert.equal(responses[2].result.structuredContent.errorCode, "unsafe_xml");
    assert.equal(responses[3].result.structuredContent.errorCode, "unsafe_xml");
    assert.equal(responses[4].result.structuredContent.errorCode, "report_too_large");
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("memory search and byte-window read preserve provenance, bounds, and source bytes", async () => {
  const fixture = createFixture();
  try {
    const memoryPath = path.join(fixture.memory, "MEMORY.md");
    const before = fs.readFileSync(memoryPath);
    const responses = await runProtocol(fixture.env, [
      call("memory_search", { query: "alpha", maxResults: 10 }, 1),
      call("memory_read", { path: "MEMORY.md", maxBytes: 64 }, 2),
    ]);
    const search = responses[0].result;
    assert.equal(search.isError, false);
    assert.deepEqual(search.structuredContent.source, { root: fs.realpathSync(fixture.memory), path: "." });
    assert.deepEqual(
      search.structuredContent.matches.map((match) => match.path),
      ["MEMORY.md", "rollouts/one.jsonl"],
    );
    assert.equal(JSON.stringify(search).includes("must not escape"), false);
    assert.ok(search.structuredContent.bytesScanned <= 8 * 1024 * 1024);

    const read = responses[1].result;
    assert.equal(read.isError, false);
    assert.equal(read.structuredContent.actualOffset, 0);
    assert.equal(read.structuredContent.nextOffset, 64);
    assert.equal(read.structuredContent.eof, false);
    assert.equal(Buffer.byteLength(read.structuredContent.text, "utf8"), 64);
    assert.deepEqual(fs.readFileSync(memoryPath), before);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("memory reads reject traversal and descendant symlinks without exposing outside content", async () => {
  const fixture = createFixture();
  try {
    const responses = await runProtocol(fixture.env, [
      call("memory_read", { path: "../outside/secret.md" }, 1),
      call("memory_read", { path: "linked-secret.md" }, 2),
    ]);
    assert.equal(responses[0].result.isError, true);
    assert.equal(responses[0].result.structuredContent.errorCode, "path_escape");
    assert.equal(responses[1].result.isError, true);
    assert.equal(responses[1].result.structuredContent.errorCode, "symlink_rejected");
    assert.equal(JSON.stringify(responses).includes("alpha must not escape"), false);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("memory read rejects invalid UTF-8 at EOF and advances across a real split-codepoint boundary", async () => {
  const fixture = createFixture();
  try {
    fs.writeFileSync(path.join(fixture.memory, "invalid.txt"), Buffer.from([0xff]));
    fs.writeFileSync(path.join(fixture.memory, "boundary.txt"), `${"a".repeat(63)}🙂tail`);
    const responses = await runProtocol(fixture.env, [
      call("memory_read", { path: "invalid.txt", maxBytes: 64 }, 1),
      call("memory_read", { path: "boundary.txt", maxBytes: 64 }, 2),
      call("memory_read", { path: "boundary.txt", offset: 63, maxBytes: 64 }, 3),
    ]);

    const invalid = responses[0].result;
    assert.equal(invalid.isError, true);
    assert.equal(invalid.structuredContent.errorCode, "non_text_source");
    assert.equal(Object.hasOwn(invalid.structuredContent, "nextOffset"), false);

    const first = responses[1].result;
    assert.equal(first.isError, false);
    assert.equal(first.structuredContent.text, "a".repeat(63));
    assert.equal(first.structuredContent.nextOffset, 63);
    assert.equal(first.structuredContent.eof, false);

    const continuation = responses[2].result;
    assert.equal(continuation.isError, false);
    assert.equal(continuation.structuredContent.text, "🙂tail");
    assert.ok(continuation.structuredContent.nextOffset > first.structuredContent.nextOffset);
    assert.equal(continuation.structuredContent.eof, true);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("CodeGraph query applies native kind plus explicit directory scope and reports freshness", async () => {
  const fixture = createFixture();
  try {
    const responses = await runProtocol(fixture.env, [
      call(
        "codegraph_scoped_query",
        { project: "fixture", search: "alpha", pathPrefix: "src", kind: "function", limit: 10 },
        1,
      ),
      call("codegraph_scoped_query", { project: "fixture", search: "alpha", limit: 10 }, 2),
    ]);
    const scoped = responses[0].result;
    assert.equal(scoped.isError, false);
    assert.deepEqual(scoped.structuredContent.results.map((item) => item.filePath), ["src/alpha.ts"]);
    assert.equal(scoped.structuredContent.pathPrefix, "src");
    assert.equal(scoped.structuredContent.kind, "function");
    assert.equal(scoped.structuredContent.complete, true);
    assert.equal(scoped.structuredContent.freshness.version, "1.6.0-test");
    assert.equal(scoped.structuredContent.freshness.pendingChanges.modified, 1);
    assert.match(scoped.content[0].text, /freshness warnings/u);

    const unscoped = responses[1].result;
    assert.equal(unscoped.isError, false);
    assert.deepEqual(
      unscoped.structuredContent.results.map((item) => item.filePath),
      ["src/alpha.ts", "src/Beta.ts", "tests/alpha.test.ts"],
    );
    assert.equal(unscoped.structuredContent.pathPrefix, null);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("CodeGraph scoped discovery excludes irrelevant worker paths while retaining freshness bounds", async () => {
  const fixture = createFixture();
  try {
    const [response] = await runProtocol(fixture.env, [
      call("codegraph_scoped_query", { project: "fixture", search: "source256", pathPrefix: "src", limit: 10 }),
    ]);
    assert.equal(response.result.isError, false);
    assert.deepEqual(response.result.structuredContent.results.map((item) => item.filePath), ["src/source256.ts"]);
    assert.equal(response.result.structuredContent.candidateCount, 2);
    assert.equal(response.result.structuredContent.complete, true);
    assert.equal(response.result.structuredContent.freshness.pendingChanges.modified, 1);
    assert.equal(response.result.structuredContent.freshness.reindexRecommended, false);
    assert.equal(JSON.stringify(response).includes("workers/worker.ts"), false);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("public history accepts native nested rollouts larger than 64 MiB with bounded reads", async () => {
  const fixture = createFixture();
  try {
    const relative = "2026/09/16/rollout-2026-09-16T02-45-11-native-thread.jsonl";
    const target = path.join(fixture.history, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, JSON.stringify({ type: "response_item", payload: {
      type: "message", role: "user", content: [{ type: "input_text", text: "Native public turn" }],
    } }) + "\n");
    fs.truncateSync(target, 122 * 1024 * 1024);
    const [response] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "native-thread", maxMessages: 1 })]);
    assert.equal(response.result.isError, false);
    const result = response.result.structuredContent;
    assert.equal(result.source.path, relative);
    assert.equal(result.messages[0].text, "Native public turn");
    assert.ok(result.progress.bytesRead <= 512 * 1024 + 32 * 1024);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history latest reads a native-shaped sparse tail without walking its prefix", async () => {
  const fixture = createFixture();
  try {
    const relative = "2026/09/16/rollout-2026-09-16T02-45-11-native-latest.jsonl";
    const target = path.join(fixture.history, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, "");
    const tailStart = 70 * 1024 * 1024;
    fs.truncateSync(target, tailStart);
    const descriptor = fs.openSync(target, "r+");
    try { fs.writeSync(descriptor, Buffer.from("\n"), 0, 1, tailStart - 1); } finally { fs.closeSync(descriptor); }
    fs.appendFileSync(target, `${JSON.stringify({
      type: "event_msg",
      thread_id: "native-latest",
      payload: { type: "user_message", id: "native-latest-tail", message: "Newest native tail" },
    })}\n`);
    const [response] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "native-latest", mode: "latest", maxMessages: 1 }),
    ]);
    const result = response.result.structuredContent;
    assert.equal(response.result.isError, false);
    assert.deepEqual(result.messages.map((message) => message.id), ["native-latest-tail"]);
    assert.ok(result.snapshotBytes > 64 * 1024 * 1024);
    assert.ok(result.progress.bytesRead <= 48 * 1024);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history fails closed for unclassified assistant content and preserves repeated turns", async () => {
  const fixture = createFixture();
  try {
    const user = { type: "response_item", payload: { type: "message", role: "user", content: [{ type: "input_text", text: "Again" }] } };
    fs.writeFileSync(path.join(fixture.history, "thread-001.jsonl"), [user, user,
      { type: "response_item", payload: { type: "message", role: "assistant", content: [{ type: "reasoning_text", text: "NONPUBLIC_MARKER" }] } },
      { type: "response_item", payload: { type: "message", role: "assistant", phase: "final", channel: "analysis", content: [{ type: "output_text", text: "NONPUBLIC_MARKER" }] } },
      { type: "response_item", payload: { type: "message", role: "assistant", phase: "final", content: [{ type: "reasoning_text", text: "NONPUBLIC_MARKER" }] } },
    ].map(JSON.stringify).join("\n") + "\n");
    const [response] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001" })]);
    const result = response.result.structuredContent;
    assert.equal(JSON.stringify(response).includes("NONPUBLIC_MARKER"), false);
    assert.deepEqual(result.messages.map(m => m.text), ["Again", "Again"]);
    assert.notEqual(result.messages[0].id, result.messages[1].id);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history resumes from EOF and preserves a partially appended record", async () => {
  const fixture = createFixture();
  try {
    const target = path.join(fixture.history, "thread-001.jsonl");
    fs.writeFileSync(target, "");
    const [first] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001" })]);
    assert.equal(first.result.structuredContent.eof, true);
    assert.equal(typeof first.result.structuredContent.nextCursor, "string");
    const line = JSON.stringify({ type: "event_msg", payload: { type: "user_message", message: "Completed append" } });
    fs.appendFileSync(target, line.slice(0, 30));
    const [partial] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001", cursor: first.result.structuredContent.nextCursor })]);
    assert.equal(partial.result.isError, false);
    assert.equal(partial.result.structuredContent.progress.nextOffset, 0);
    assert.equal(partial.result.structuredContent.complete, false);
    assert.equal(partial.result.structuredContent.waitingForAppend, true);
    assert.deepEqual(partial.result.structuredContent.omissions, []);
    fs.appendFileSync(target, line.slice(30) + "\n");
    const [done] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001", cursor: partial.result.structuredContent.nextCursor })]);
    assert.equal(done.result.isError, false);
    assert.deepEqual(done.result.structuredContent.messages.map(m => m.text), ["Completed append"]);
    assert.equal(done.result.structuredContent.complete, true);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history latest pages newest records chronologically against its initial snapshot", async () => {
  const fixture = createFixture();
  try {
    const target = path.join(fixture.history, "thread-001.jsonl");
    const records = Array.from({ length: 6 }, (_, index) => JSON.stringify({
      type: "event_msg",
      thread_id: "thread-001",
      payload: { type: "user_message", id: `latest-${index + 1}`, message: `Latest ${index + 1}` },
    }));
    fs.writeFileSync(target, `${records.join("\n")}\n`);
    const snapshotBytes = fs.statSync(target).size;

    const [first] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", mode: "latest", maxMessages: 2 }),
    ]);
    const one = first.result.structuredContent;
    assert.equal(first.result.isError, false);
    assert.equal(one.mode, "latest");
    assert.equal(one.snapshotBytes, snapshotBytes);
    assert.equal(one.hasMore, true);
    assert.equal(one.complete, false);
    assert.deepEqual(one.messages.map((message) => message.id), ["latest-5", "latest-6"]);
    assert.ok(one.messages.every((message) => message.source.line === null));
    assert.ok(one.messages.every((message) => Number.isSafeInteger(message.source.offset)));
    assert.equal(one.eof, undefined);
    assert.equal(one.waitingForAppend, undefined);

    fs.appendFileSync(target, `${JSON.stringify({
      type: "event_msg",
      thread_id: "thread-001",
      payload: { type: "user_message", id: "latest-7", message: "Must not leak into snapshot" },
    })}\n`);
    const [second] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", cursor: one.nextCursor, maxMessages: 2 }),
    ]);
    const two = second.result.structuredContent;
    assert.equal(second.result.isError, false);
    assert.equal(two.snapshotBytes, snapshotBytes);
    assert.deepEqual(two.messages.map((message) => message.id), ["latest-3", "latest-4"]);

    const [third, conflict] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", cursor: two.nextCursor, maxMessages: 2 }, 1),
      call("public_history_read", { threadId: "thread-001", cursor: two.nextCursor, mode: "forward" }, 2),
    ]);
    const three = third.result.structuredContent;
    assert.equal(third.result.isError, false);
    assert.deepEqual(three.messages.map((message) => message.id), ["latest-1", "latest-2"]);
    assert.equal(three.hasMore, false);
    assert.equal(three.complete, true);
    assert.equal(three.nextCursor, null);
    assert.equal(conflict.result.isError, true);
    assert.equal(conflict.result.structuredContent.errorCode, "invalid_cursor");
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history latest exhausts a snapshot containing only an incomplete record", async () => {
  const fixture = createFixture();
  try {
    fs.writeFileSync(path.join(fixture.history, "thread-001.jsonl"), '{"type":"event_msg"');
    const [response] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", mode: "latest" }),
    ]);
    assert.equal(response.result.isError, false);
    const result = response.result.structuredContent;
    assert.deepEqual(result.messages, []);
    assert.equal(result.complete, true);
    assert.equal(result.hasMore, false);
    assert.equal(result.nextCursor, null);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history latest retains a valid boundary record when its scan budget ends", async () => {
  const fixture = createFixture();
  try {
    const target = path.join(fixture.history, "thread-001.jsonl");
    fs.writeFileSync(target, Array.from({ length: 15 }, (_, index) => JSON.stringify({
      type: "event_msg",
      thread_id: "thread-001",
      payload: { type: "user_message", id: `m${index}`, message: "x".repeat(60_000) },
    })).join("\n") + "\n");
    const observed = [];
    let cursor;
    do {
      const [response] = await runProtocol(fixture.env, [
        call("public_history_read", { threadId: "thread-001", mode: "latest", maxMessages: 20, ...(cursor ? { cursor } : {}) }),
      ]);
      assert.equal(response.result.isError, false);
      const result = response.result.structuredContent;
      observed.push(...result.messages.map((message) => message.id).reverse());
      cursor = result.nextCursor;
    } while (cursor);
    assert.deepEqual(observed, Array.from({ length: 15 }, (_, index) => `m${14 - index}`));
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history forward pages use incremental reads instead of eager scan amplification", async () => {
  const fixture = createFixture();
  try {
    const target = path.join(fixture.history, "thread-001.jsonl");
    const records = [];
    for (let index = 0; index < 10; index++) {
      records.push(JSON.stringify({ type: "system", thread_id: "thread-001", payload: { padding: "x".repeat(48 * 1024) } }));
      records.push(JSON.stringify({
        type: "event_msg",
        thread_id: "thread-001",
        payload: { type: "user_message", id: `amplification-${index}`, message: `Page ${index}` },
      }));
    }
    fs.writeFileSync(target, `${records.join("\n")}\n`);
    let cursor;
    let bytesRead = 0;
    const ids = [];
    for (let page = 0; page < 10; page++) {
      const [response] = await runProtocol(fixture.env, [
        call("public_history_read", { threadId: "thread-001", maxMessages: 1, ...(cursor ? { cursor } : {}) }),
      ]);
      assert.equal(response.result.isError, false);
      const result = response.result.structuredContent;
      bytesRead += result.progress.bytesRead;
      ids.push(...result.messages.map((message) => message.id));
      cursor = result.nextCursor;
    }
    assert.deepEqual(ids, Array.from({ length: 10 }, (_, index) => `amplification-${index}`));
    assert.ok(bytesRead <= 1024 * 1024, `read ${bytesRead} bytes for ten pages`);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history discovery and read project only bounded public messages", async () => {
  const fixture = createFixture();
  try {
    const responses = await runProtocol(fixture.env, [
      listTools(1),
      call("public_history_read", { threadId: "thread-001", maxMessages: 2 }, 2),
    ]);
    assert.ok(responses[0].result.tools.some((tool) => tool.name === "public_history_read"));
    const result = responses[1].result;
    assert.equal(result.isError, false);
    assert.equal(result.structuredContent.threadId, "thread-001");
    assert.deepEqual(result.structuredContent.messages.map((message) => [message.role, message.phase]), [
      ["user", "user"],
      ["assistant", "commentary"],
    ]);
    assert.equal(result.structuredContent.messages.length, 2);
    assert.equal(result.structuredContent.messages[1].redacted, true);
    assert.equal(result.structuredContent.messages[1].text.includes("sk-proj-testnotreal123456"), false);
    assert.equal(result.structuredContent.messages[1].text.includes("REDACTED_CREDENTIAL"), true);
    assert.equal(result.structuredContent.complete, false);
    assert.equal(typeof result.structuredContent.nextCursor, "string");
    assert.equal(JSON.stringify(result).includes("Synthetic private reasoning"), false);
    assert.equal(JSON.stringify(result).includes("Synthetic tool payload"), false);
    assert.equal(JSON.stringify(result).includes("duplicate commentary"), false);
    assert.equal(result.structuredContent.source.root, fs.realpathSync(fixture.history));
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("public history serialized output budget preserves continuation for escaped text", async () => {
  const fixture = createFixture();
  try {
    fs.writeFileSync(path.join(fixture.history, "thread-001.jsonl"), Array.from({ length: 20 }, (_, i) =>
      JSON.stringify({ type: "event_msg", payload: { type: "user_message", id: `large-${i}`, message: '"'.repeat(4096) } }),
    ).join("\n") + "\n");
    const ids = [];
    let cursor;
    for (let page = 0; page < 5; page++) {
      const [response] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001", maxMessages: 20, ...(cursor ? { cursor } : {}) })]);
      assert.equal(response.result.isError, false);
      assert.ok(Buffer.byteLength(JSON.stringify(response.result)) <= 120 * 1024);
      const result = response.result.structuredContent;
      ids.push(...result.messages.map(m => m.id));
      if (result.complete) break;
      cursor = result.nextCursor;
    }
    assert.deepEqual(ids, Array.from({ length: 20 }, (_, i) => `large-${i}`));
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history rejects ambiguous native sources, mismatched metadata and replaced inodes", async () => {
  const fixture = createFixture();
  try {
    const target = path.join(fixture.history, "thread-001.jsonl");
    const native = path.join(fixture.history, "2026/09/16/rollout-2026-09-16T02-45-11-thread-001.jsonl");
    fs.mkdirSync(path.dirname(native), { recursive: true });
    fs.copyFileSync(target, native);
    const [ambiguous] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001" })]);
    assert.equal(ambiguous.result.structuredContent.errorCode, "history_ambiguous");
    fs.unlinkSync(native);
    fs.writeFileSync(target, JSON.stringify({ type: "session_meta", payload: { id: "another-thread" } }) + "\n");
    const [mismatch] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001" })]);
    assert.equal(mismatch.result.structuredContent.errorCode, "history_thread_mismatch");
    fs.writeFileSync(target, "");
    const [initial] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001" })]);
    fs.renameSync(target, `${target}.old`);
    fs.writeFileSync(target, "");
    const [replaced] = await runProtocol(fixture.env, [call("public_history_read", { threadId: "thread-001", cursor: initial.result.structuredContent.nextCursor })]);
    assert.equal(replaced.result.structuredContent.errorCode, "history_source_changed");
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history pagination carries duplicate suppression across record representations", async () => {
  const fixture = createFixture();
  try {
    const [first] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", maxMessages: 2 }),
    ]);
    const cursor = first.result.structuredContent.nextCursor;
    const [second] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", cursor, maxMessages: 20 }),
    ]);
    assert.equal(second.result.isError, false);
    assert.deepEqual(second.result.structuredContent.messages.map((message) => message.id), ["msg-3"]);
    assert.ok(
      second.result.structuredContent.omissions.some(
        (omission) => omission.reason === "duplicate_representation",
      ),
    );
    assert.ok(
      second.result.structuredContent.omissions.some(
        (omission) => omission.reason === "non_public_record",
      ),
    );
    assert.equal(JSON.stringify(second).includes("duplicate commentary"), false);
    assert.equal(JSON.stringify(second).includes("Synthetic hidden system content"), false);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("public history cursor resumes appends and rejects truncation or replacement", async () => {
  const fixture = createFixture();
  try {
    const [first] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", maxMessages: 1 }),
    ]);
    const cursor = first.result.structuredContent.nextCursor;
    assert.equal(first.result.isError, false);
    assert.equal(typeof cursor, "string");

    fs.appendFileSync(
      path.join(fixture.history, "thread-001.jsonl"),
      `${JSON.stringify({ type: "event_msg", thread_id: "thread-001", payload: { type: "user_message", id: "msg-append", message: "Synthetic appended message" } })}\n`,
    );
    const [appended] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", cursor, maxMessages: 20 }),
    ]);
    assert.equal(appended.result.isError, false);
    assert.ok(appended.result.structuredContent.messages.some((message) => message.id === "msg-append"));

    fs.writeFileSync(
      path.join(fixture.history, "thread-001.jsonl"),
      `${JSON.stringify({ type: "event_msg", thread_id: "thread-001", payload: { type: "user_message", id: "msg-replaced", message: "Synthetic replacement" } })}\n`,
    );
    const [replaced] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-001", cursor, maxMessages: 20 }),
    ]);
    assert.equal(replaced.result.isError, true);
    assert.equal(replaced.result.structuredContent.errorCode, "history_source_changed");
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("public history omits malformed and oversized records with bounded receipts", async () => {
  const fixture = createFixture();
  try {
    const badPath = path.join(fixture.history, "thread-bad.jsonl");
    fs.writeFileSync(
      badPath,
      [
        "{malformed synthetic record",
        "x".repeat(128 * 1024 + 1),
        JSON.stringify({ type: "response_item", thread_id: "thread-bad", id: "msg-good", payload: { type: "message", role: "user", content: [{ type: "input_text", text: "Synthetic valid record" }] } }),
      ].join("\n") + "\n",
    );
    const [response] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-bad", maxMessages: 20 }),
    ]);
    assert.equal(response.result.isError, false);
    assert.deepEqual(response.result.structuredContent.messages.map((message) => message.id), ["msg-good"]);
    assert.deepEqual(
      response.result.structuredContent.omissions.map((omission) => omission.reason),
      ["malformed_record", "record_too_large"],
    );
    assert.equal(JSON.stringify(response).includes("x".repeat(256)), false);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("public history advances through an oversized unterminated record without buffering it", async () => {
  const fixture = createFixture();
  try {
    const badPath = path.join(fixture.history, "thread-discard.jsonl");
    fs.writeFileSync(
      badPath,
      `${"x".repeat(512 * 1024 + 64)}\n${JSON.stringify({
        type: "response_item",
        thread_id: "thread-discard",
        id: "msg-after-discard",
        payload: { type: "message", role: "user", content: [{ type: "input_text", text: "Synthetic after oversized record" }] },
      })}\n`,
    );
    const [first] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-discard", maxMessages: 20 }),
    ]);
    assert.equal(first.result.isError, false);
    assert.deepEqual(first.result.structuredContent.messages, []);
    assert.equal(first.result.structuredContent.progress.discarding, true);
    assert.equal(typeof first.result.structuredContent.nextCursor, "string");

    const [second] = await runProtocol(fixture.env, [
      call("public_history_read", {
        threadId: "thread-discard",
        cursor: first.result.structuredContent.nextCursor,
        maxMessages: 20,
      }),
    ]);
    assert.equal(second.result.isError, false);
    assert.deepEqual(second.result.structuredContent.messages.map((message) => message.id), ["msg-after-discard"]);
    assert.equal(second.result.structuredContent.progress.discarding, false);
    assert.equal(second.result.structuredContent.eof, true);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("public history advances a sparse oversized record across a 512 KiB page", async () => {
  const fixture = createFixture();
  try {
    const target = path.join(fixture.history, "thread-sparse-discard.jsonl");
    fs.writeFileSync(target, "");
    fs.truncateSync(target, 512 * 1024 + 257);
    const descriptor = fs.openSync(target, "r+");
    try { fs.writeSync(descriptor, Buffer.from("\n"), 0, 1, 512 * 1024 + 256); } finally { fs.closeSync(descriptor); }
    fs.appendFileSync(target, `${JSON.stringify({
      type: "event_msg",
      thread_id: "thread-sparse-discard",
      payload: { type: "user_message", id: "sparse-after", message: "After sparse oversized record" },
    })}\n`);
    const [first] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-sparse-discard", maxMessages: 1 }),
    ]);
    const one = first.result.structuredContent;
    assert.equal(first.result.isError, false);
    assert.equal(one.progress.discarding, true);
    assert.ok(one.progress.nextOffset >= 512 * 1024);
    const [second] = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "thread-sparse-discard", cursor: one.nextCursor, maxMessages: 1 }),
    ]);
    assert.equal(second.result.isError, false);
    assert.deepEqual(second.result.structuredContent.messages.map((message) => message.id), ["sparse-after"]);
  } finally { fs.rmSync(fixture.root, { recursive: true, force: true }); }
});

test("public history rejects traversal and descendant symlinks", async () => {
  const fixture = createFixture();
  try {
    fs.symlinkSync(path.join(fixture.outside, "secret.md"), path.join(fixture.history, "thread-link.jsonl"));
    const responses = await runProtocol(fixture.env, [
      call("public_history_read", { threadId: "../outside" }, 1),
      call("public_history_read", { threadId: "thread-link" }, 2),
    ]);
    assert.equal(responses[0].result.isError, true);
    assert.equal(responses[0].result.structuredContent.errorCode, "invalid_arguments");
    assert.equal(responses[1].result.isError, true);
    assert.equal(responses[1].result.structuredContent.errorCode, "symlink_rejected");
    assert.equal(JSON.stringify(responses).includes("alpha must not escape"), false);
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("CodeGraph rejects unsupported file and escaping scopes instead of ignoring them", async () => {
  const fixture = createFixture();
  try {
    const responses = await runProtocol(fixture.env, [
      call("codegraph_scoped_query", { project: "fixture", search: "alpha", pathPrefix: "src/alpha.ts" }, 1),
      call("codegraph_scoped_query", { project: "fixture", search: "alpha", pathPrefix: "../outside" }, 2),
    ]);
    assert.equal(responses[0].result.isError, true);
    assert.equal(responses[0].result.structuredContent.errorCode, "unsupported_scope");
    assert.equal(responses[1].result.isError, true);
    assert.equal(responses[1].result.structuredContent.errorCode, "path_escape");
  } finally {
    fs.rmSync(fixture.root, { recursive: true, force: true });
  }
});

test("missing optional configuration omits capabilities and malformed configuration fails listing", async () => {
  const minimalEnv = { ...process.env };
  delete minimalEnv.WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON;
  delete minimalEnv.WEBCODEX_WEB_WORKFLOW_MEMORY_ROOT;
  delete minimalEnv.WEBCODEX_WEB_WORKFLOW_CODEGRAPH_RUNTIME;
  delete minimalEnv.WEBCODEX_WEB_WORKFLOW_CODEGRAPH_ENTRY;
  delete minimalEnv.WEBCODEX_WEB_WORKFLOW_HISTORY_ROOT;
  const minimal = await runProtocol(minimalEnv, [listTools()]);
  assert.deepEqual(minimal[0].result.tools, []);

  const invalid = await runProtocol(
    { ...minimalEnv, WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON: "not-json" },
    [listTools()],
  );
  assert.equal(invalid[0].error.code, -32603);
  assert.match(invalid[0].error.message, /WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON/u);
});
