#!/usr/bin/env node

// Optional WebCodex Native Tool Plugin for bounded Web workflow context.
// stdout is reserved for one-line JSON-RPC protocol messages.

import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import { TextDecoder } from "node:util";
import { fileURLToPath } from "node:url";

const PROTOCOL_VERSION = "webcodex-plugin-v1";
const RESULT_TEXT_BYTES = 60 * 1024;
const REPORT_MAX_BYTES = 4 * 1024 * 1024;
const MEMORY_READ_MAX_BYTES = 32 * 1024;
const MEMORY_SEARCH_FILE_BYTES = 1024 * 1024;
const MEMORY_SEARCH_TOTAL_BYTES = 8 * 1024 * 1024;
const MEMORY_SEARCH_MAX_FILES = 512;
const MEMORY_SEARCH_MAX_DIRECTORIES = 1024;
const CODEGRAPH_CANDIDATE_LIMIT = 1000;
const CHILD_OUTPUT_BYTES = 1024 * 1024;
const CHILD_TIMEOUT_MS = 100_000;
const HISTORY_SCAN_BYTES = 512 * 1024;
const HISTORY_READ_TARGET_BYTES = 32 * 1024;
const HISTORY_RECORD_MAX_BYTES = 128 * 1024;
const HISTORY_DISCOVERY_MAX_ENTRIES = 8192;
const HISTORY_MESSAGE_MAX_BYTES = 4 * 1024;
const HISTORY_MAX_OMISSIONS = 32;
const HISTORY_MAX_MESSAGES = 20;
const HISTORY_MESSAGES_MAX_BYTES = 48 * 1024;
const HISTORY_SEEN_IDS_MAX = 20;
const HISTORY_CURSOR_MAX_LENGTH = 8 * 1024;
const here = path.dirname(fileURLToPath(import.meta.url));
const pytestHelper = path.join(here, "pytest_report.py");
const utf8Decoder = new TextDecoder("utf-8", { fatal: true });

const readOnlyAnnotations = {
  readOnlyHint: true,
  destructiveHint: false,
  idempotentHint: true,
  openWorldHint: false,
};

const stringProperty = (description, maxLength = 4096) => ({
  type: "string",
  description,
  minLength: 1,
  maxLength,
});

const integerProperty = (description, minimum, maximum) => ({
  type: "integer",
  description: `${description} Runtime accepts integers from ${minimum} through ${maximum}, inclusive.`,
});

const pytestTool = {
  name: "pytest_report_summary",
  title: "Summarize a pytest JUnit report",
  description:
    "Parse an existing pytest JUnit XML file under one operator-configured project root. This never runs pytest or claims test execution. It returns source provenance, pass/fail/error/skip counts, bounded failing test identities, and explicit truncation or malformed-report errors.",
  inputSchema: {
    type: "object",
    properties: {
      project: stringProperty("Operator-configured project id containing the report.", 64),
      path: stringProperty("Project-relative path to an existing JUnit XML report."),
      maxFailures: integerProperty("Maximum failing/error test identities to return; defaults to 20.", 1, 25),
    },
    required: ["project", "path"],
    additionalProperties: false,
  },
  annotations: readOnlyAnnotations,
};

const memorySearchTool = {
  name: "memory_search",
  title: "Search shared Codex memory",
  description:
    "Search UTF-8 text beneath the operator-configured read-only memory root. The search does not follow symlinks, write files, copy context, or merge project state. Results identify their exact source root and relative file; scan and result limits are explicit.",
  inputSchema: {
    type: "object",
    properties: {
      query: stringProperty("Case-insensitive literal text to search for.", 256),
      path: stringProperty("Optional memory-root-relative directory scope; defaults to the full configured root."),
      maxResults: integerProperty("Maximum matching lines to return; defaults to 20.", 1, 50),
    },
    required: ["query"],
    additionalProperties: false,
  },
  annotations: readOnlyAnnotations,
};

const memoryReadTool = {
  name: "memory_read",
  title: "Read shared Codex memory",
  description:
    "Read one bounded UTF-8 byte window from a file beneath the operator-configured read-only memory root. Paths cannot traverse the root and descendant symlinks are rejected. The result includes source provenance and a continuation byte offset; no writes or copies are performed.",
  inputSchema: {
    type: "object",
    properties: {
      path: stringProperty("Memory-root-relative file path."),
      offset: integerProperty("Zero-based source byte offset; defaults to 0.", 0, Number.MAX_SAFE_INTEGER),
      maxBytes: integerProperty(
        `Maximum source bytes to read; defaults to 16384 and cannot exceed ${MEMORY_READ_MAX_BYTES}.`,
        64,
        MEMORY_READ_MAX_BYTES,
      ),
    },
    required: ["path"],
    additionalProperties: false,
  },
  annotations: readOnlyAnnotations,
};

const codegraphTool = {
  name: "codegraph_scoped_query",
  title: "Search CodeGraph within a code scope",
  description:
    "Search indexed symbols in an operator-configured project and optionally restrict results to one live project-relative directory. CodeGraph 1.6 query has native search, kind, JSON, and limit options but no path filter, so this adapter fetches a bounded candidate set and applies the directory scope itself. It reports when the candidate or result ceiling prevents an exhaustive answer and includes index freshness evidence. File, glob, language, and arbitrary expression scopes are unsupported and are rejected by the schema or as unsupported_scope, never ignored.",
  inputSchema: {
    type: "object",
    properties: {
      project: stringProperty("Operator-configured project id to query.", 64),
      search: stringProperty("Symbol search text passed to CodeGraph.", 4096),
      pathPrefix: stringProperty("Optional project-relative directory scope."),
      kind: stringProperty(
        "Optional CodeGraph node kind filter, such as function, class, or method. Runtime accepts an ASCII identifier beginning with a letter or underscore, followed by letters, digits, underscores, or hyphens.",
        128,
      ),
      limit: integerProperty("Maximum returned symbols; defaults to 20.", 1, 50),
    },
    required: ["project", "search"],
    additionalProperties: false,
  },
  annotations: readOnlyAnnotations,
};

const publicHistoryTool = {
  name: "public_history_read",
  title: "Read bounded public thread history",
  description:
    "Read bounded public user messages and assistant final/commentary messages for one explicitly selected local thread. Filename-only bounded discovery supports native dated rollouts; no unrelated history bodies are scanned. Analysis, system/developer/configuration and tool payloads are excluded and credential-like text is redacted. Forward is the default and keeps its source-bound cursor at EOF for appends. Latest returns newest messages from a fixed snapshot; its cursor continues only toward older snapshot history.",
  inputSchema: {
    type: "object",
    properties: {
      threadId: stringProperty("Explicit thread identity; the reader opens the configured root's matching thread file.", 128),
      cursor: stringProperty("Opaque cursor returned by a prior read for this same thread and source.", HISTORY_CURSOR_MAX_LENGTH),
      mode: stringProperty("Optional traversal mode: forward (the default) or latest. A continuation cursor fixes its mode.", 16),
      maxMessages: integerProperty(`Maximum public messages to return; defaults to 10 and cannot exceed ${HISTORY_MAX_MESSAGES}.`, 1, HISTORY_MAX_MESSAGES),
    },
    required: ["threadId"],
    additionalProperties: false,
  },
  annotations: readOnlyAnnotations,
};

class ApplicationError extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
  }
}

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function rpcError(id, code, message) {
  send({ jsonrpc: "2.0", id, error: { code, message } });
}

function truncateUtf8(value, maxBytes) {
  const source = String(value ?? "").replaceAll("\u0000", "");
  const bytes = Buffer.from(source, "utf8");
  if (bytes.length <= maxBytes) return source;
  let end = maxBytes;
  while (end > 0) {
    try {
      return `${utf8Decoder.decode(bytes.subarray(0, end))}\n[truncated at ${maxBytes} UTF-8 bytes]`;
    } catch {
      end -= 1;
    }
  }
  return `[truncated at ${maxBytes} UTF-8 bytes]`;
}

function toolResult(id, text, structuredContent, isError = false) {
  const result = {
    content: [{ type: "text", text: truncateUtf8(text, RESULT_TEXT_BYTES) }],
    structuredContent,
    isError,
  };
  if (Buffer.byteLength(JSON.stringify(result), "utf8") > RESULT_TEXT_BYTES * 2) {
    send({
      jsonrpc: "2.0",
      id,
      result: {
        content: [{ type: "text", text: "Web workflow result exceeded its output bound." }],
        structuredContent: { errorCode: "result_too_large" },
        isError: true,
      },
    });
    return;
  }
  send({ jsonrpc: "2.0", id, result });
}

function applicationFailure(id, error) {
  const code = error instanceof ApplicationError ? error.code : "internal_error";
  const message =
    error instanceof ApplicationError
      ? error.message
      : "The Web workflow Plugin could not complete this read-only request.";
  toolResult(id, message, { errorCode: code }, true);
}

function assertObject(value, label = "arguments") {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new ApplicationError("invalid_arguments", `${label} must be an object.`);
  }
  return value;
}

function rejectUnknownKeys(value, allowed) {
  for (const key of Object.keys(value)) {
    if (!allowed.has(key)) {
      throw new ApplicationError("invalid_arguments", `Unsupported argument: ${key}.`);
    }
  }
}

function requiredString(value, field, maxLength) {
  if (typeof value !== "string" || value.length === 0 || value.length > maxLength || value.includes("\u0000")) {
    throw new ApplicationError("invalid_arguments", `${field} must be a non-empty string of at most ${maxLength} characters.`);
  }
  return value;
}

function integerInRange(value, fallback, minimum, maximum, field) {
  if (value === undefined) return fallback;
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    throw new ApplicationError("invalid_arguments", `${field} must be an integer from ${minimum} through ${maximum}.`);
  }
  return value;
}

function safeConfigurationMessage(variable, problem) {
  return `${variable} ${problem}; fix the operator configuration and re-check the Plugin.`;
}

function configuredDirectory(raw, variable) {
  if (!path.isAbsolute(raw)) {
    throw new ApplicationError("invalid_configuration", safeConfigurationMessage(variable, "must be an absolute directory"));
  }
  let resolved;
  try {
    resolved = fs.realpathSync(raw);
    if (!fs.statSync(resolved).isDirectory()) throw new Error("not a directory");
  } catch {
    throw new ApplicationError(
      "invalid_configuration",
      safeConfigurationMessage(variable, "must resolve to a directory"),
    );
  }
  return resolved;
}

function configuredFile(raw, variable, executable = false) {
  if (!path.isAbsolute(raw)) {
    throw new ApplicationError("invalid_configuration", safeConfigurationMessage(variable, "must be an absolute file"));
  }
  let resolved;
  try {
    resolved = fs.realpathSync(raw);
    if (!fs.statSync(resolved).isFile()) throw new Error("not a file");
    if (executable) fs.accessSync(resolved, fs.constants.X_OK);
  } catch {
    const expectation = executable ? "must resolve to an executable file" : "must resolve to a regular file";
    throw new ApplicationError("invalid_configuration", safeConfigurationMessage(variable, expectation));
  }
  return resolved;
}

function loadConfiguration() {
  const projectsRaw = process.env.WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON;
  const memoryRaw = process.env.WEBCODEX_WEB_WORKFLOW_MEMORY_ROOT;
  const codegraphRuntimeRaw = process.env.WEBCODEX_WEB_WORKFLOW_CODEGRAPH_RUNTIME;
  const codegraphEntryRaw = process.env.WEBCODEX_WEB_WORKFLOW_CODEGRAPH_ENTRY;
  const historyRootRaw = process.env.WEBCODEX_WEB_WORKFLOW_HISTORY_ROOT;
  const projects = new Map();

  if (projectsRaw !== undefined) {
    let parsed;
    try {
      parsed = JSON.parse(projectsRaw);
    } catch {
      throw new ApplicationError(
        "invalid_configuration",
        safeConfigurationMessage("WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON", "must contain one JSON object"),
      );
    }
    if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed) || Object.keys(parsed).length === 0) {
      throw new ApplicationError(
        "invalid_configuration",
        safeConfigurationMessage("WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON", "must contain a non-empty JSON object"),
      );
    }
    for (const [id, root] of Object.entries(parsed)) {
      if (!/^[A-Za-z0-9._-]{1,64}$/u.test(id) || typeof root !== "string") {
        throw new ApplicationError(
          "invalid_configuration",
          safeConfigurationMessage("WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON", "has an invalid project id or root"),
        );
      }
      projects.set(id, configuredDirectory(root, `WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON[${id}]`));
    }
  }

  if ((codegraphRuntimeRaw === undefined) !== (codegraphEntryRaw === undefined)) {
    throw new ApplicationError(
      "invalid_configuration",
      "WEBCODEX_WEB_WORKFLOW_CODEGRAPH_RUNTIME and WEBCODEX_WEB_WORKFLOW_CODEGRAPH_ENTRY must be configured together.",
    );
  }
  if (codegraphRuntimeRaw !== undefined && projects.size === 0) {
    throw new ApplicationError(
      "invalid_configuration",
      "Configured CodeGraph access requires WEBCODEX_WEB_WORKFLOW_PROJECTS_JSON.",
    );
  }

  return {
    projects,
    memoryRoot: memoryRaw === undefined ? undefined : configuredDirectory(memoryRaw, "WEBCODEX_WEB_WORKFLOW_MEMORY_ROOT"),
    python: process.env.WEBCODEX_WEB_WORKFLOW_PYTHON || "python3",
    codegraph:
      codegraphRuntimeRaw === undefined
        ? undefined
        : {
            runtime: configuredFile(codegraphRuntimeRaw, "WEBCODEX_WEB_WORKFLOW_CODEGRAPH_RUNTIME", true),
            entry: configuredFile(codegraphEntryRaw, "WEBCODEX_WEB_WORKFLOW_CODEGRAPH_ENTRY"),
          },
    historyRoot:
      historyRootRaw === undefined
        ? undefined
        : configuredDirectory(historyRootRaw, "WEBCODEX_WEB_WORKFLOW_HISTORY_ROOT"),
  };
}

let configuration;
let configurationError;
try {
  configuration = loadConfiguration();
} catch (error) {
  configurationError = error;
}

function requireConfiguration() {
  if (configurationError !== undefined) throw configurationError;
  return configuration;
}

function availableTools() {
  const config = requireConfiguration();
  const tools = [];
  if (config.projects.size > 0) tools.push(pytestTool);
  if (config.memoryRoot !== undefined) tools.push(memorySearchTool, memoryReadTool);
  if (config.codegraph !== undefined) tools.push(codegraphTool);
  if (config.historyRoot !== undefined) tools.push(publicHistoryTool);
  return tools;
}

function projectRoot(config, project) {
  const id = requiredString(project, "project", 64);
  const root = config.projects.get(id);
  if (root === undefined) {
    throw new ApplicationError("unknown_project", `Unknown configured project id: ${id}.`);
  }
  return { id, root };
}

function normalizeRelative(value, field, allowDot = false) {
  const raw = requiredString(value, field, 4096);
  if (raw.includes("\\") || path.posix.isAbsolute(raw)) {
    throw new ApplicationError("invalid_path", `${field} must be a forward-slash relative path.`);
  }
  const normalized = path.posix.normalize(raw);
  if (
    normalized === ".." ||
    normalized.startsWith("../") ||
    (!allowDot && normalized === ".")
  ) {
    throw new ApplicationError("path_escape", `${field} cannot traverse or name the configured root itself.`);
  }
  return normalized;
}

function confinedPath(root, relative, expected, errorCode = "invalid_path") {
  const segments = relative === "." ? [] : relative.split("/");
  let current = root;
  for (let index = 0; index < segments.length; index += 1) {
    current = path.join(current, segments[index]);
    let stat;
    try {
      stat = fs.lstatSync(current);
    } catch {
      throw new ApplicationError(errorCode, "The requested source path does not exist.");
    }
    if (stat.isSymbolicLink()) {
      throw new ApplicationError("symlink_rejected", "Descendant symlinks are not readable through this Plugin.");
    }
    if (index < segments.length - 1 && !stat.isDirectory()) {
      throw new ApplicationError(errorCode, "A requested source path component is not a directory.");
    }
  }
  const resolved = fs.realpathSync(current);
  const relation = path.relative(root, resolved);
  if (relation === ".." || relation.startsWith(`..${path.sep}`) || path.isAbsolute(relation)) {
    throw new ApplicationError("path_escape", "The requested source path escapes its configured root.");
  }
  const stat = fs.statSync(resolved);
  if (expected === "file" && !stat.isFile()) {
    throw new ApplicationError(errorCode, "The requested source path is not a regular file.");
  }
  if (expected === "directory" && !stat.isDirectory()) {
    throw new ApplicationError(errorCode, "Only a live project-relative directory scope is supported.");
  }
  return { resolved, stat };
}

function readFilePrefix(root, relative, maximum) {
  const { resolved, stat } = confinedPath(root, relative, "file");
  const flags = fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0);
  const fd = fs.openSync(resolved, flags);
  try {
    const current = fs.fstatSync(fd);
    if (!current.isFile()) throw new ApplicationError("invalid_path", "The requested source path is not a regular file.");
    const toRead = Math.min(current.size, maximum);
    const buffer = Buffer.alloc(toRead);
    let used = 0;
    while (used < toRead) {
      const count = fs.readSync(fd, buffer, used, toRead - used, used);
      if (count === 0) break;
      used += count;
    }
    return { buffer: buffer.subarray(0, used), size: current.size, truncated: current.size > maximum, stat };
  } finally {
    fs.closeSync(fd);
  }
}

function readFileWindow(root, relative, offset, maximum) {
  const { resolved } = confinedPath(root, relative, "file");
  const flags = fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0);
  const fd = fs.openSync(resolved, flags);
  try {
    const stat = fs.fstatSync(fd);
    if (!stat.isFile()) throw new ApplicationError("invalid_path", "The requested source path is not a regular file.");
    if (offset > stat.size) {
      throw new ApplicationError("invalid_offset", "offset is beyond the end of the source file.");
    }
    const toRead = Math.min(maximum, stat.size - offset);
    const buffer = Buffer.alloc(toRead);
    let used = 0;
    while (used < toRead) {
      const count = fs.readSync(fd, buffer, used, toRead - used, offset + used);
      if (count === 0) break;
      used += count;
    }
    return { buffer: buffer.subarray(0, used), size: stat.size };
  } finally {
    fs.closeSync(fd);
  }
}

function expectedUtf8SequenceLength(leadingByte) {
  if (leadingByte >= 0xc2 && leadingByte <= 0xdf) return 2;
  if (leadingByte >= 0xe0 && leadingByte <= 0xef) return 3;
  if (leadingByte >= 0xf0 && leadingByte <= 0xf4) return 4;
  return undefined;
}

function isContinuationByte(value) {
  return (value & 0xc0) === 0x80;
}

function isPotentialUtf8Prefix(buffer, start, end, expectedLength) {
  const available = end - start;
  if (available >= expectedLength) return false;
  for (let index = start + 1; index < end; index += 1) {
    if (!isContinuationByte(buffer[index])) return false;
  }
  if (available <= 1) return true;
  const lead = buffer[start];
  const second = buffer[start + 1];
  if (lead === 0xe0 && second < 0xa0) return false;
  if (lead === 0xed && second > 0x9f) return false;
  if (lead === 0xf0 && second < 0x90) return false;
  if (lead === 0xf4 && second > 0x8f) return false;
  return true;
}

function incompleteUtf8SuffixStart(buffer, start) {
  for (let candidate = buffer.length - 1; candidate >= Math.max(start, buffer.length - 3); candidate -= 1) {
    const expectedLength = expectedUtf8SequenceLength(buffer[candidate]);
    if (
      expectedLength !== undefined &&
      isPotentialUtf8Prefix(buffer, candidate, buffer.length, expectedLength)
    ) {
      return candidate;
    }
  }
  return undefined;
}

function decodeUtf8Window(buffer, requestedOffset, sourceSize) {
  let start = 0;
  if (requestedOffset > 0) {
    while (start < Math.min(buffer.length, 3) && isContinuationByte(buffer[start])) start += 1;
  }
  const decode = (end) => {
    const text = utf8Decoder.decode(buffer.subarray(start, end));
    if (text.includes("\u0000")) {
      throw new ApplicationError("non_text_source", "The source contains NUL bytes and is not treated as text.");
    }
    return { text, actualOffset: requestedOffset + start, nextOffset: requestedOffset + end };
  };
  try {
    return decode(buffer.length);
  } catch (error) {
    if (error instanceof ApplicationError) throw error;
  }

  const hasUnreadSource = requestedOffset + buffer.length < sourceSize;
  if (hasUnreadSource) {
    const suffixStart = incompleteUtf8SuffixStart(buffer, start);
    if (suffixStart !== undefined) {
      try {
        return decode(suffixStart);
      } catch (error) {
        if (error instanceof ApplicationError) throw error;
      }
    }
  }
  throw new ApplicationError("non_text_source", "The requested source window is not valid UTF-8 text.");
}

function decodeUtf8Prefix(buffer, truncated) {
  const attempts = truncated ? 3 : 0;
  for (let removed = 0; removed <= attempts && buffer.length >= removed; removed += 1) {
    try {
      const text = utf8Decoder.decode(buffer.subarray(0, buffer.length - removed));
      if (text.includes("\u0000")) throw new Error("binary");
      return text;
    } catch {
      // A bounded prefix may end within one UTF-8 code point. Retry only at
      // that trailing boundary; invalid bytes elsewhere remain non-text.
    }
  }
  throw new ApplicationError("non_text_source", "The source is not valid UTF-8 text.");
}

function runChild(executable, args, input) {
  return new Promise((resolve, reject) => {
    const child = spawn(executable, args, {
      stdio: [input === undefined ? "ignore" : "pipe", "pipe", "pipe"],
      windowsHide: true,
      shell: false,
      env: { ...process.env, NO_COLOR: "1" },
    });
    const stdout = [];
    const stderr = [];
    let stdoutBytes = 0;
    let stderrBytes = 0;
    let settled = false;
    const finishError = (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      child.kill("SIGKILL");
      reject(error);
    };
    const timer = setTimeout(() => {
      finishError(new ApplicationError("provider_timeout", "The local parser/provider timed out."));
    }, CHILD_TIMEOUT_MS);
    child.stdout.on("data", (chunk) => {
      stdoutBytes += chunk.length;
      if (stdoutBytes > CHILD_OUTPUT_BYTES) {
        finishError(new ApplicationError("provider_output_too_large", "The local parser/provider output exceeded its byte bound."));
        return;
      }
      stdout.push(chunk);
    });
    child.stderr.on("data", (chunk) => {
      stderrBytes += chunk.length;
      if (stderrBytes > CHILD_OUTPUT_BYTES) {
        finishError(new ApplicationError("provider_output_too_large", "The local parser/provider diagnostic exceeded its byte bound."));
        return;
      }
      stderr.push(chunk);
    });
    child.once("error", () => {
      finishError(new ApplicationError("provider_unavailable", "The configured local parser/provider could not be started."));
    });
    child.once("close", (code) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      if (code !== 0) {
        reject(new ApplicationError("provider_failed", "The local parser/provider exited without a valid result."));
        return;
      }
      resolve({
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      });
    });
    if (input !== undefined) child.stdin.end(input);
  });
}

async function summarizePytest(args, config) {
  const values = assertObject(args);
  rejectUnknownKeys(values, new Set(["project", "path", "maxFailures"]));
  const project = projectRoot(config, values.project);
  const relative = normalizeRelative(values.path, "path");
  const maxFailures = integerInRange(values.maxFailures, 20, 1, 25, "maxFailures");
  const report = readFilePrefix(project.root, relative, REPORT_MAX_BYTES + 1);
  if (report.size > REPORT_MAX_BYTES || report.buffer.length > REPORT_MAX_BYTES) {
    throw new ApplicationError("report_too_large", `The report exceeds the ${REPORT_MAX_BYTES}-byte input bound.`);
  }
  const { stdout } = await runChild(
    config.python,
    [pytestHelper, "--max-failures", String(maxFailures)],
    report.buffer,
  );
  let parsed;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    throw new ApplicationError("parser_output_invalid", "The pytest report parser returned malformed output.");
  }
  if (parsed?.ok !== true) {
    throw new ApplicationError(parsed?.errorCode || "malformed_xml", parsed?.message || "The JUnit XML report is malformed.");
  }
  const structured = {
    parserStatus: "parsed",
    testsExecuted: false,
    source: { project: project.id, root: project.root, path: relative },
    bytesRead: report.buffer.length,
    counts: parsed.counts,
    failures: parsed.failures,
    failuresTruncated: parsed.failuresTruncated,
  };
  return {
    text:
      `Parsed existing JUnit report; no tests were executed. ` +
      `${parsed.counts.passed} passed, ${parsed.counts.failed} failed, ` +
      `${parsed.counts.errors} errors, ${parsed.counts.skipped} skipped (${parsed.counts.total} total).` +
      (parsed.failuresTruncated ? " Failing identities were truncated by maxFailures." : ""),
    structured,
  };
}

const searchableExtensions = new Set([".json", ".jsonl", ".md", ".toml", ".txt", ".yaml", ".yml"]);

function collectMemoryFiles(root, scope) {
  const start = confinedPath(root, scope, "directory").resolved;
  const queue = [{ absolute: start, relative: scope === "." ? "" : scope }];
  const files = [];
  let directories = 0;
  let truncated = false;
  while (queue.length > 0 && files.length < MEMORY_SEARCH_MAX_FILES && directories < MEMORY_SEARCH_MAX_DIRECTORIES) {
    const directory = queue.shift();
    directories += 1;
    let entries;
    try {
      entries = fs.readdirSync(directory.absolute, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name));
    } catch {
      throw new ApplicationError("memory_read_failed", "A memory directory could not be read.");
    }
    for (const entry of entries) {
      if (entry.isSymbolicLink()) continue;
      const relative = directory.relative ? `${directory.relative}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        if (entry.name !== ".git" && entry.name !== ".serena") {
          queue.push({ absolute: path.join(directory.absolute, entry.name), relative });
        }
      } else if (entry.isFile() && searchableExtensions.has(path.extname(entry.name).toLowerCase())) {
        files.push(relative);
        if (files.length >= MEMORY_SEARCH_MAX_FILES) break;
      }
    }
  }
  if (queue.length > 0 || files.length >= MEMORY_SEARCH_MAX_FILES || directories >= MEMORY_SEARCH_MAX_DIRECTORIES) {
    truncated = true;
  }
  return { files, truncated };
}

function summarizeLine(line) {
  return truncateUtf8(line.replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/gu, " ").trim(), 512);
}

function searchMemory(args, config) {
  const values = assertObject(args);
  rejectUnknownKeys(values, new Set(["query", "path", "maxResults"]));
  const query = requiredString(values.query, "query", 256);
  const scope = values.path === undefined ? "." : normalizeRelative(values.path, "path", true);
  const maxResults = integerInRange(values.maxResults, 20, 1, 50, "maxResults");
  const collected = collectMemoryFiles(config.memoryRoot, scope);
  const matches = [];
  const matchedFiles = new Set();
  let bytesScanned = 0;
  let filesScanned = 0;
  let skippedNonText = 0;
  let sourceWindowsTruncated = false;
  let scanLimitReached = false;
  const needle = query.toLocaleLowerCase("en-US");

  for (const relative of collected.files) {
    const remaining = MEMORY_SEARCH_TOTAL_BYTES - bytesScanned;
    if (remaining <= 0 || matches.length >= maxResults) {
      scanLimitReached = remaining <= 0;
      break;
    }
    const maximum = Math.min(MEMORY_SEARCH_FILE_BYTES, remaining);
    const source = readFilePrefix(config.memoryRoot, relative, maximum);
    bytesScanned += source.buffer.length;
    filesScanned += 1;
    sourceWindowsTruncated ||= source.truncated;
    let text;
    try {
      text = decodeUtf8Prefix(source.buffer, source.truncated);
    } catch {
      skippedNonText += 1;
      continue;
    }
    const lines = text.split(/\r?\n/u);
    for (let index = 0; index < lines.length; index += 1) {
      if (lines[index].toLocaleLowerCase("en-US").includes(needle)) {
        matches.push({ path: relative, line: index + 1, text: summarizeLine(lines[index]) });
        matchedFiles.add(relative);
        if (matches.length >= maxResults) break;
      }
    }
  }

  const truncated =
    collected.truncated ||
    sourceWindowsTruncated ||
    scanLimitReached ||
    matches.length >= maxResults ||
    filesScanned < collected.files.length;
  const structured = {
    source: { root: config.memoryRoot, path: scope },
    query,
    matches,
    matchedFiles: matchedFiles.size,
    filesScanned,
    bytesScanned,
    skippedNonText,
    truncated,
    limits: {
      maxResults,
      maxFiles: MEMORY_SEARCH_MAX_FILES,
      maxFileBytes: MEMORY_SEARCH_FILE_BYTES,
      maxScanBytes: MEMORY_SEARCH_TOTAL_BYTES,
    },
  };
  return {
    text: `Found ${matches.length} matching lines in ${matchedFiles.size} memory files; scanned ${filesScanned} files and ${bytesScanned} bytes.${truncated ? " Results are bounded and may be incomplete." : ""}`,
    structured,
  };
}

function readMemory(args, config) {
  const values = assertObject(args);
  rejectUnknownKeys(values, new Set(["path", "offset", "maxBytes"]));
  const relative = normalizeRelative(values.path, "path");
  const offset = integerInRange(values.offset, 0, 0, Number.MAX_SAFE_INTEGER, "offset");
  const maxBytes = integerInRange(values.maxBytes, 16 * 1024, 64, MEMORY_READ_MAX_BYTES, "maxBytes");
  const source = readFileWindow(config.memoryRoot, relative, offset, maxBytes);
  const decoded = decodeUtf8Window(source.buffer, offset, source.size);
  const eof = decoded.nextOffset >= source.size;
  const structured = {
    source: { root: config.memoryRoot, path: relative },
    requestedOffset: offset,
    actualOffset: decoded.actualOffset,
    nextOffset: decoded.nextOffset,
    sourceBytes: source.size,
    eof,
    text: decoded.text,
  };
  return {
    text: `Read ${decoded.nextOffset - decoded.actualOffset} UTF-8 source bytes from ${relative} at byte ${decoded.actualOffset}.${eof ? " End of file." : ` Continue at byte ${decoded.nextOffset}.`}`,
    structured,
  };
}

function historyThreadPath(threadId, root, cursor) {
  const id = requiredString(threadId, "threadId", 128);
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/u.test(id)) {
    throw new ApplicationError(
      "invalid_arguments",
      "threadId must be an ASCII identifier suitable for one configured history file.",
    );
  }
  const matches = (relative) => relative === `${id}.jsonl` ||
    new RegExp(`^(?:\\d{4}/\\d{2}/\\d{2}/)?rollout-\\d{4}-\\d{2}-\\d{2}T\\d{2}-\\d{2}-\\d{2}-${id.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")}\\.jsonl$`, "u").test(relative);
  if (cursor !== undefined) {
    if (cursor.threadId !== id || !matches(cursor.path)) {
      throw new ApplicationError("invalid_cursor", "cursor belongs to a different thread history.");
    }
    return { id, relative: cursor.path };
  }
  // Only filenames are inspected. Never read unrelated history to locate a thread.
  const found = [];
  let entries = 0;
  function visit(relative, depth) {
    const directory = fs.opendirSync(path.join(root, relative));
    try {
      let entry;
      while ((entry = directory.readSync()) !== null) {
        if (++entries > HISTORY_DISCOVERY_MAX_ENTRIES) {
          throw new ApplicationError("history_discovery_limit", "History filename discovery exceeded its bound; configure a narrower history root.");
        }
        const candidate = relative ? `${relative}/${entry.name}` : entry.name;
        if (matches(candidate)) found.push(candidate);
        if (entry.isDirectory() && depth < 3 && (depth === 0 ? /^\d{4}$/u : /^\d{2}$/u).test(entry.name)) {
          visit(candidate, depth + 1);
        }
      }
    } finally { directory.closeSync(); }
  }
  visit("", 0);
  if (found.length !== 1) {
    throw new ApplicationError(found.length ? "history_ambiguous" : "history_not_found", "Expected exactly one matching thread history under the configured root.");
  }
  return { id, relative: found[0] };
}

function historyStatIdentity(stat) {
  return {
    device: String(stat.dev),
    inode: String(stat.ino),
    size: stat.size,
    modified: String(Math.trunc(stat.mtimeMs * 1000)),
  };
}

function hashFileRange(source, start, length) {
  const hash = createHash("sha256");
  const buffer = Buffer.alloc(Math.min(4096, Math.max(length, 1)));
  let remaining = length;
  let offset = 0;
  while (remaining > 0) {
    const count = fs.readSync(source.fd, buffer, 0, Math.min(buffer.length, remaining), start + offset);
    if (count === 0) throw historySourceChanged();
    source.bytesRead += count;
    hash.update(buffer.subarray(0, count));
    remaining -= count;
    offset += count;
  }
  return hash.digest("hex");
}

function readHistorySource(root, relative) {
  const { resolved } = confinedPath(root, relative, "file", "history_not_found");
  const flags = fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW ?? 0);
  const fd = fs.openSync(resolved, flags);
  try {
    const stat = fs.fstatSync(fd);
    if (!stat.isFile()) throw new ApplicationError("history_not_found", "The selected thread history is not a regular file.");
    return {
      resolved,
      size: stat.size,
      stat,
      fd,
      bytesRead: 0,
    };
  } catch (error) {
    fs.closeSync(fd);
    throw error;
  }
}

function readHistoryRange(source, offset, length) {
  if (length <= 0 || offset >= source.size) return Buffer.alloc(0);
  const wanted = Math.min(length, source.size - offset);
  const buffer = Buffer.alloc(wanted);
  let used = 0;
  while (used < wanted) {
    const count = fs.readSync(source.fd, buffer, used, wanted - used, offset + used);
    if (count === 0) throw historySourceChanged();
    used += count;
  }
  source.bytesRead += used;
  return buffer;
}

function encodeHistoryCursor(value) {
  return Buffer.from(JSON.stringify(value), "utf8").toString("base64url");
}

function decodeHistoryCursor(value) {
  if (value === undefined) return undefined;
  const raw = requiredString(value, "cursor", HISTORY_CURSOR_MAX_LENGTH);
  let parsed;
  try {
    parsed = JSON.parse(Buffer.from(raw, "base64url").toString("utf8"));
  } catch {
    throw new ApplicationError("invalid_cursor", "cursor is not a valid history continuation cursor.");
  }
  const validIdentity = (identity) => identity !== null &&
    typeof identity === "object" &&
    typeof identity.device === "string" &&
    typeof identity.inode === "string" &&
    Number.isSafeInteger(identity.size) &&
    identity.size >= 0 &&
    typeof identity.modified === "string" &&
    typeof identity.headHash === "string" &&
    typeof identity.tailHash === "string";
  if (
    parsed === null ||
    typeof parsed !== "object" ||
    typeof parsed.threadId !== "string" ||
    typeof parsed.path !== "string" ||
    (parsed.seen !== undefined &&
      (!Array.isArray(parsed.seen) ||
        parsed.seen.length > HISTORY_SEEN_IDS_MAX ||
        parsed.seen.some((id) => typeof id !== "string" || id.length === 0 || id.length > 256))) ||
    !validIdentity(parsed.identity)
  ) {
    throw new ApplicationError("invalid_cursor", "cursor has an invalid history continuation shape.");
  }
  if (parsed.version === 2) {
    if (!Number.isSafeInteger(parsed.offset) || parsed.offset < 0 ||
        !Number.isSafeInteger(parsed.line) || parsed.line < 1 ||
        (parsed.discarding !== undefined && typeof parsed.discarding !== "boolean")) {
      throw new ApplicationError("invalid_cursor", "cursor has an invalid history continuation shape.");
    }
    return parsed;
  }
  if (parsed.version === 3 && parsed.mode === "latest") {
    if (!Number.isSafeInteger(parsed.snapshotBytes) || parsed.snapshotBytes < 0 ||
        parsed.snapshotBytes !== parsed.identity.size ||
        !Number.isSafeInteger(parsed.boundary) || parsed.boundary < 0 || parsed.boundary > parsed.snapshotBytes ||
        (parsed.discarding !== undefined && typeof parsed.discarding !== "boolean")) {
      throw new ApplicationError("invalid_cursor", "cursor has an invalid latest-history continuation shape.");
    }
    return parsed;
  }
  throw new ApplicationError("invalid_cursor", "cursor has an invalid history continuation shape.");
}

function historySourceChanged() {
  return new ApplicationError(
    "history_source_changed",
    "The selected thread history was truncated or replaced; restart from the beginning.",
  );
}

function validateHistoryCursor(cursor, threadId, relative, source) {
  if (cursor.threadId !== threadId || cursor.path !== relative) {
    throw new ApplicationError("invalid_cursor", "cursor belongs to a different thread history.");
  }
  const identity = cursor.identity;
  const current = historyStatIdentity(source.stat);
  if (
    identity.device !== current.device ||
    identity.inode !== current.inode ||
    (cursor.version === 2 && cursor.offset > identity.size) ||
    (cursor.version === 3 && cursor.boundary > identity.size) ||
    source.size < identity.size ||
    source.size < cursor.offset ||
    (source.size === identity.size && identity.modified !== current.modified)
  ) {
    throw historySourceChanged();
  }
  const length = Math.min(identity.size, 4096);
  if (identity.headHash !== hashFileRange(source, 0, length) ||
      identity.tailHash !== hashFileRange(source, identity.size - length, length)) {
    throw historySourceChanged();
  }
}

function recordThreadId(record) {
  return record?.type === "session_meta" ? record.payload?.id : record?.thread_id;
}

function redactPublicText(value) {
  let text = String(value ?? "").replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/gu, " ");
  let redacted = false;
  const replace = (pattern, replacement) => {
    const next = text.replace(pattern, replacement);
    if (next !== text) redacted = true;
    text = next;
  };
  replace(/\b(?:sk|rk|pk)-[A-Za-z0-9_-]{12,}\b/gu, "[REDACTED_CREDENTIAL]");
  replace(/\b(?:gh[pousr]_[A-Za-z0-9_]{12,}|xox[baprs]-[A-Za-z0-9-]{12,})\b/gu, "[REDACTED_CREDENTIAL]");
  replace(/\b(?:AKIA|ASIA)[A-Z0-9]{16}\b/gu, "[REDACTED_CREDENTIAL]");
  replace(/\bBearer\s+[A-Za-z0-9._~+/=-]{12,}/giu, "Bearer [REDACTED_CREDENTIAL]");
  replace(
    /\b((?:api[_-]?key|access[_-]?token|authorization|password|secret|token)\s*[:=]\s*)[^\s,;]+/giu,
    "$1[REDACTED_CREDENTIAL]",
  );
  const bounded = truncateUtf8(text, HISTORY_MESSAGE_MAX_BYTES);
  if (bounded !== text) redacted = true;
  return { text: bounded, redacted };
}

function publicHistoryMessage(record) {
  if (record === null || typeof record !== "object" || Array.isArray(record)) return undefined;
  const payload = record.payload;
  if (payload === null || typeof payload !== "object") return undefined;
  let role, phase, rawText;
  const channels = [payload.phase, payload.channel, record.channel].filter(value => value != null);
  if (channels.some(value => value !== "final" && value !== "commentary")) return undefined;
  if (record.type === "event_msg" && ["user_message", "agent_message"].includes(payload.type)) {
    role = payload.type === "user_message" ? "user" : "assistant";
    phase = role === "user" ? "user" : (channels[0] ?? "final");
    rawText = payload.message;
  } else if (record.type === "response_item" && payload.type === "message" && ["user", "assistant"].includes(payload.role)) {
    role = payload.role;
    if (role === "assistant" && channels.length === 0) return undefined;
    phase = role === "user" ? "user" : channels[0];
    if (!Array.isArray(payload.content)) return undefined;
    const textType = role === "user" ? "input_text" : "output_text";
    // Images and unrecognized content blocks are never recursively projected.
    rawText = payload.content.filter(item => item?.type === textType && typeof item.text === "string").map(item => item.text).join("\n");
  } else return undefined;
  if (typeof rawText !== "string") return undefined;
  rawText = rawText.trim();
  if (rawText.length === 0) return undefined;
  const safe = redactPublicText(rawText);
  const payloadId = [payload.id, record.id].find(candidate => typeof candidate === "string" && candidate.length > 0 && candidate.length <= 256);
  return {
    id: payloadId,
    role,
    phase,
    text: safe.text,
    redacted: safe.redacted,
  };
}

function addHistoryOmission(omissions, line, offset, reason) {
  if (omissions.length < HISTORY_MAX_OMISSIONS) omissions.push({ line, offset, reason });
}

function historyCursor(threadId, relative, source, offset, line, discarding, seen) {
  return encodeHistoryCursor({
    version: 2,
    threadId,
    path: relative,
    offset,
    line,
    discarding,
    seen: Array.from(seen).slice(-HISTORY_SEEN_IDS_MAX),
    identity: {
      ...historyStatIdentity(source.stat),
      headHash: hashFileRange(source, 0, Math.min(source.size, 4096)),
      tailHash: hashFileRange(
        source,
        Math.max(0, source.stat.size - 4096),
        Math.min(source.stat.size, 4096),
      ),
    },
  });
}

function historyIdentity(source, snapshotBytes = source.size) {
  return {
    ...historyStatIdentity(source.stat),
    size: snapshotBytes,
    headHash: hashFileRange(source, 0, Math.min(snapshotBytes, 4096)),
    tailHash: hashFileRange(source, Math.max(0, snapshotBytes - 4096), Math.min(snapshotBytes, 4096)),
  };
}

function projectHistoryRecord(recordBytes, sourceLine, sourceOffset, threadId, relative, seen, omissions, latest) {
  let normalized = recordBytes;
  if (normalized.length > 0 && normalized[normalized.length - 1] === 0x0d) normalized = normalized.subarray(0, normalized.length - 1);
  if (normalized.length > HISTORY_RECORD_MAX_BYTES) {
    addHistoryOmission(omissions, sourceLine, sourceOffset, "record_too_large");
    return undefined;
  }
  if (normalized.length === 0) return undefined;
  let record;
  try {
    record = JSON.parse(utf8Decoder.decode(normalized));
  } catch {
    addHistoryOmission(omissions, sourceLine, sourceOffset, "malformed_record");
    return undefined;
  }
  const recordId = recordThreadId(record);
  if (record?.type === "session_meta" && recordId !== threadId) {
    throw new ApplicationError("history_thread_mismatch", "The source session identity does not match the requested thread.");
  }
  if (recordId !== undefined && recordId !== threadId) {
    addHistoryOmission(omissions, sourceLine, sourceOffset, "different_thread");
    return undefined;
  }
  const message = publicHistoryMessage(record);
  if (message === undefined) {
    addHistoryOmission(omissions, sourceLine, sourceOffset, "non_public_record");
    return undefined;
  }
  if (message.id !== undefined && seen.has(message.id)) {
    addHistoryOmission(omissions, sourceLine, sourceOffset, "duplicate_representation");
    return undefined;
  }
  return {
    explicitId: message.id,
    projected: {
      ...message,
      id: message.id ?? `${relative}:${sourceOffset}`,
      source: { path: relative, line: latest ? null : sourceLine, offset: sourceOffset },
    },
  };
}

function rememberHistoryMessage(explicitId, seen) {
  if (explicitId !== undefined) {
    seen.add(explicitId);
    if (seen.size > HISTORY_SEEN_IDS_MAX) seen.delete(seen.values().next().value);
  }
}

function readForwardHistory(source, state) {
  const { threadId, relative, maxMessages, offset, line, discarding: initialDiscarding, seen } = state;
  const messages = [];
  const omissions = [];
  let messagesBytes = 0;
  let current = offset;
  let lineStart = offset;
  let nextLine = line;
  let discarding = initialDiscarding;
  let parts = [];
  let recordBytes = 0;
  let recordsScanned = 0;
  let scanBytesRead = 0;
  let stopped = false;
  while (current < source.size && scanBytesRead < HISTORY_SCAN_BYTES && !stopped) {
    const chunk = readHistoryRange(source, current, Math.min(HISTORY_READ_TARGET_BYTES, HISTORY_SCAN_BYTES - scanBytesRead));
    scanBytesRead += chunk.length;
    let position = 0;
    while (position < chunk.length) {
      const newline = chunk.indexOf(0x0a, position);
      const end = newline < 0 ? chunk.length : newline;
      const piece = chunk.subarray(position, end);
      const consumed = (newline < 0 ? chunk.length : newline + 1) - position;
      if (!discarding) {
        recordBytes += piece.length;
        if (recordBytes > HISTORY_RECORD_MAX_BYTES) {
          addHistoryOmission(omissions, nextLine, lineStart, "record_too_large");
          parts = [];
          discarding = true;
        } else {
          parts.push(piece);
        }
      }
      current += consumed;
      position += consumed;
      if (newline < 0) continue;
      const sourceLine = nextLine;
      nextLine += 1;
      recordsScanned += 1;
      if (!discarding) {
        const admission = projectHistoryRecord(Buffer.concat(parts, recordBytes), sourceLine, lineStart, threadId, relative, seen, omissions, false);
        if (admission !== undefined) {
          const projectedBytes = Buffer.byteLength(JSON.stringify(admission.projected), "utf8");
          if (messagesBytes + projectedBytes > HISTORY_MESSAGES_MAX_BYTES) {
            current = lineStart;
            nextLine = sourceLine;
            recordsScanned -= 1;
            stopped = true;
            break;
          }
          rememberHistoryMessage(admission.explicitId, seen);
          messages.push(admission.projected);
          messagesBytes += projectedBytes;
        }
      }
      discarding = false;
      parts = [];
      recordBytes = 0;
      lineStart = current;
      if (messages.length >= maxMessages) {
        stopped = true;
        break;
      }
    }
  }
  const atEof = current >= source.size;
  const partial = !discarding && lineStart < current;
  const waitingForAppend = atEof && (partial || discarding);
  const nextOffset = partial ? lineStart : current;
  const complete = nextOffset >= source.size && !waitingForAppend;
  return {
    messages,
    omissions,
    recordsScanned,
    bytesScanned: current - offset,
    nextOffset,
    nextLine,
    discarding,
    complete,
    waitingForAppend,
  };
}

function previousNewline(source, end, scan) {
  let current = end;
  while (current > 0 && scan.bytes < HISTORY_SCAN_BYTES) {
    const size = Math.min(HISTORY_READ_TARGET_BYTES, current, HISTORY_SCAN_BYTES - scan.bytes);
    const start = current - size;
    const chunk = readHistoryRange(source, start, size);
    scan.bytes += chunk.length;
    const newline = chunk.lastIndexOf(0x0a);
    if (newline >= 0) return { found: true, offset: start + newline };
    current = start;
  }
  return { found: false, offset: current };
}

function readLatestRecord(source, start, end, scan) {
  const parts = [];
  let current = start;
  while (current < end) {
    const size = Math.min(HISTORY_READ_TARGET_BYTES, end - current, HISTORY_SCAN_BYTES - scan.bytes);
    if (size <= 0) return undefined;
    const chunk = readHistoryRange(source, current, size);
    scan.bytes += chunk.length;
    parts.push(chunk);
    current += chunk.length;
  }
  return Buffer.concat(parts);
}

function latestHistoryCursor(threadId, relative, identity, boundary, seen, discarding) {
  return encodeHistoryCursor({
    version: 3,
    mode: "latest",
    threadId,
    path: relative,
    snapshotBytes: identity.size,
    boundary,
    discarding,
    seen: Array.from(seen).slice(-HISTORY_SEEN_IDS_MAX),
    identity,
  });
}

function readLatestHistory(source, state) {
  const { threadId, relative, maxMessages, identity, seen } = state;
  let boundary = state.boundary;
  let discarding = state.discarding;
  const messages = [];
  const omissions = [];
  let messagesBytes = 0;
  let recordsScanned = 0;
  const scan = { bytes: 0 };
  while (boundary > 0 && scan.bytes < HISTORY_SCAN_BYTES && messages.length < maxMessages) {
    if (discarding) {
      const skipped = previousNewline(source, boundary, scan);
      if (!skipped.found) {
        boundary = skipped.offset;
        break;
      }
      boundary = skipped.offset + 1;
      discarding = false;
      continue;
    }
    const terminal = readHistoryRange(source, boundary - 1, 1);
    scan.bytes += terminal.length;
    if (terminal[0] !== 0x0a) {
      const trailing = previousNewline(source, boundary, scan);
      if (!trailing.found) {
        // A scan-budget boundary is not evidence that this trailing record is
        // oversized. Keep a possibly valid record at its original boundary so
        // a fresh call can reread it with a full budget.
        if (trailing.offset === 0 || boundary - trailing.offset > HISTORY_RECORD_MAX_BYTES) {
          boundary = trailing.offset;
          if (boundary > 0) discarding = true;
        }
        break;
      }
      boundary = trailing.offset + 1;
      continue;
    }
    const previous = previousNewline(source, boundary - 1, scan);
    if (!previous.found) {
      // Do not conflate the per-call scan budget with the record bound: a
      // short valid record can be cut by the remaining call budget after
      // newer records consumed it. Only advance a discard continuation once
      // the scanned suffix itself proves this record exceeds the record cap.
      if (previous.offset > 0) {
        if (boundary - 1 - previous.offset > HISTORY_RECORD_MAX_BYTES) {
          boundary = previous.offset;
          discarding = true;
        }
        break;
      }
    }
    const start = previous.found ? previous.offset + 1 : 0;
    const recordLength = boundary - 1 - start;
    if (recordLength > HISTORY_RECORD_MAX_BYTES) {
      addHistoryOmission(omissions, null, start, "record_too_large");
      recordsScanned += 1;
      boundary = start;
      continue;
    }
    const record = readLatestRecord(source, start, boundary - 1, scan);
    if (record === undefined) break;
    const admission = projectHistoryRecord(record, null, start, threadId, relative, seen, omissions, true);
    recordsScanned += 1;
    if (admission !== undefined) {
      const projectedBytes = Buffer.byteLength(JSON.stringify(admission.projected), "utf8");
      if (messagesBytes + projectedBytes > HISTORY_MESSAGES_MAX_BYTES) break;
      rememberHistoryMessage(admission.explicitId, seen);
      messages.push(admission.projected);
      messagesBytes += projectedBytes;
    }
    boundary = start;
  }
  const complete = boundary === 0;
  return { messages: messages.reverse(), omissions, recordsScanned, bytesScanned: scan.bytes, boundary, discarding, complete };
}

function readPublicHistory(args, config) {
  const values = assertObject(args);
  rejectUnknownKeys(values, new Set(["threadId", "cursor", "mode", "maxMessages"]));
  const cursor = decodeHistoryCursor(values.cursor);
  const requestedMode = values.mode === undefined ? undefined : requiredString(values.mode, "mode", 16);
  if (requestedMode !== undefined && requestedMode !== "forward" && requestedMode !== "latest") {
    throw new ApplicationError("invalid_arguments", "mode must be forward or latest.");
  }
  const cursorMode = cursor?.version === 3 ? "latest" : "forward";
  if (cursor !== undefined && requestedMode !== undefined && requestedMode !== cursorMode) {
    throw new ApplicationError("invalid_cursor", "cursor mode conflicts with the requested history traversal mode.");
  }
  const mode = requestedMode ?? cursorMode;
  const { id: threadId, relative } = historyThreadPath(values.threadId, config.historyRoot, cursor);
  const maxMessages = integerInRange(values.maxMessages, 10, 1, HISTORY_MAX_MESSAGES, "maxMessages");
  const source = readHistorySource(config.historyRoot, relative);
  try {
    if (cursor !== undefined) validateHistoryCursor(cursor, threadId, relative, source);
    if (mode === "latest") {
      const identity = cursor?.identity ?? historyIdentity(source);
      const seen = new Set(cursor?.seen ?? []);
      const result = readLatestHistory(source, {
        threadId,
        relative,
        maxMessages,
        identity,
        boundary: cursor?.boundary ?? identity.size,
        discarding: cursor?.discarding === true,
        seen,
      });
      const nextCursor = result.complete ? null : latestHistoryCursor(threadId, relative, identity, result.boundary, seen, result.discarding);
      const structured = {
        source: { root: config.historyRoot, path: relative }, threadId, mode, snapshotBytes: identity.size,
        messages: result.messages, returnedMessages: result.messages.length, hasMore: !result.complete, complete: result.complete, nextCursor,
        progress: { bytesScanned: result.bytesScanned, bytesRead: source.bytesRead, recordsScanned: result.recordsScanned, nextBoundary: result.boundary, discarding: result.discarding },
        omissions: result.omissions, omissionsTruncated: result.omissions.length >= HISTORY_MAX_OMISSIONS,
        limits: { maxMessages, maxScanBytes: HISTORY_SCAN_BYTES, maxRecordBytes: HISTORY_RECORD_MAX_BYTES, maxMessageBytes: HISTORY_MESSAGE_MAX_BYTES, maxMessagesJsonBytes: HISTORY_MESSAGES_MAX_BYTES, maxReadBytes: HISTORY_SCAN_BYTES + 16 * 1024, maxDiscoveryEntries: HISTORY_DISCOVERY_MAX_ENTRIES },
      };
      return { text: `Read ${result.messages.length} newest public messages for thread ${threadId}; scanned ${result.recordsScanned} records and ${result.bytesScanned} bytes.${result.complete ? " Snapshot exhausted." : " Continue with nextCursor for older snapshot history."}${result.omissions.length > 0 ? " See omission receipts for skipped records." : ""}`, structured };
    }
    const seen = new Set(cursor?.seen ?? []);
    const result = readForwardHistory(source, { threadId, relative, maxMessages, offset: cursor?.offset ?? 0, line: cursor?.line ?? 1, discarding: cursor?.discarding === true, seen });
    const nextCursor = historyCursor(threadId, relative, source, result.nextOffset, result.nextLine, result.discarding, seen);
    const structured = {
      source: { root: config.historyRoot, path: relative }, threadId, mode, messages: result.messages, returnedMessages: result.messages.length,
      complete: result.complete, eof: result.nextOffset >= source.size, waitingForAppend: result.waitingForAppend, nextCursor,
      progress: { bytesScanned: result.bytesScanned, bytesRead: source.bytesRead, recordsScanned: result.recordsScanned, nextOffset: result.nextOffset, nextLine: result.nextLine, discarding: result.discarding },
      omissions: result.omissions, omissionsTruncated: result.omissions.length >= HISTORY_MAX_OMISSIONS,
      limits: { maxMessages, maxScanBytes: HISTORY_SCAN_BYTES, maxRecordBytes: HISTORY_RECORD_MAX_BYTES, maxMessageBytes: HISTORY_MESSAGE_MAX_BYTES, maxMessagesJsonBytes: HISTORY_MESSAGES_MAX_BYTES, maxReadBytes: HISTORY_SCAN_BYTES + 16 * 1024, maxDiscoveryEntries: HISTORY_DISCOVERY_MAX_ENTRIES },
    };
    return { text: `Read ${result.messages.length} public messages for thread ${threadId}; scanned ${result.recordsScanned} records and ${result.bytesScanned} bytes.${result.waitingForAppend ? " Incomplete record: retain nextCursor and wait for source growth." : result.complete ? " Current EOF: retain nextCursor for future appends." : " Continue with nextCursor for the bounded remainder."}${result.omissions.length > 0 ? " See omission receipts for skipped records." : ""}`, structured };
  } finally { fs.closeSync(source.fd); }
}

async function runCodeGraph(config, command) {
  const { stdout } = await runChild(config.codegraph.runtime, [
    "--liftoff-only",
    "--disable-warning=ExperimentalWarning",
    config.codegraph.entry,
    ...command,
  ]);
  try {
    return JSON.parse(stdout);
  } catch {
    throw new ApplicationError("codegraph_output_invalid", "CodeGraph returned malformed JSON output.");
  }
}

function boundedProviderString(value, maximum) {
  if (value === undefined || value === null) return undefined;
  return truncateUtf8(String(value), maximum);
}

function projectRelativeCodePath(value) {
  if (typeof value !== "string" || value.includes("\u0000")) {
    throw new ApplicationError("codegraph_output_invalid", "CodeGraph returned an invalid file path.");
  }
  const normalized = path.posix.normalize(value.replaceAll("\\", "/"));
  if (normalized === ".." || normalized.startsWith("../") || path.posix.isAbsolute(normalized)) {
    throw new ApplicationError("codegraph_output_invalid", "CodeGraph returned a file path outside the project.");
  }
  return normalized;
}

function projectFreshness(status) {
  if (status === null || typeof status !== "object" || Array.isArray(status)) {
    throw new ApplicationError("codegraph_output_invalid", "CodeGraph status output is invalid.");
  }
  const count = (value, field) => {
    if (!Number.isSafeInteger(value) || value < 0) {
      throw new ApplicationError("codegraph_output_invalid", `CodeGraph status returned an invalid ${field} count.`);
    }
    return value;
  };
  let worktreeMismatch = null;
  if (status.worktreeMismatch !== null && status.worktreeMismatch !== undefined) {
    try {
      worktreeMismatch = boundedProviderString(JSON.stringify(status.worktreeMismatch), 2048);
    } catch {
      throw new ApplicationError("codegraph_output_invalid", "CodeGraph status returned an invalid worktree mismatch.");
    }
  }
  return {
    initialized: status.initialized === true,
    version: boundedProviderString(status.version, 64) ?? "",
    lastIndexed: boundedProviderString(status.lastIndexed, 128) ?? "",
    pendingChanges:
      status.pendingChanges !== null && typeof status.pendingChanges === "object"
        ? {
            added: count(status.pendingChanges.added ?? 0, "added"),
            modified: count(status.pendingChanges.modified ?? 0, "modified"),
            removed: count(status.pendingChanges.removed ?? 0, "removed"),
          }
        : { added: 0, modified: 0, removed: 0 },
    worktreeMismatch,
    reindexRecommended: status.index?.reindexRecommended === true,
    state: boundedProviderString(status.index?.state, 64) ?? "",
  };
}

function projectNode(item) {
  if (item === null || typeof item !== "object" || item.node === null || typeof item.node !== "object") {
    throw new ApplicationError("codegraph_output_invalid", "CodeGraph query returned an invalid result item.");
  }
  const node = item.node;
  return {
    name: boundedProviderString(node.name, 512) ?? "",
    qualifiedName: boundedProviderString(node.qualifiedName, 1024) ?? "",
    kind: boundedProviderString(node.kind, 128) ?? "",
    filePath: projectRelativeCodePath(node.filePath),
    language: boundedProviderString(node.language, 128) ?? "",
    startLine: Number.isSafeInteger(node.startLine) ? node.startLine : null,
    endLine: Number.isSafeInteger(node.endLine) ? node.endLine : null,
    score: typeof item.score === "number" && Number.isFinite(item.score) ? item.score : null,
  };
}

async function queryCodeGraph(args, config) {
  const values = assertObject(args);
  rejectUnknownKeys(values, new Set(["project", "search", "pathPrefix", "kind", "limit"]));
  const project = projectRoot(config, values.project);
  const search = requiredString(values.search, "search", 4096);
  const kind = values.kind === undefined ? undefined : requiredString(values.kind, "kind", 128);
  if (kind !== undefined && !/^[A-Za-z_][A-Za-z0-9_-]*$/u.test(kind)) {
    throw new ApplicationError("invalid_arguments", "kind is not a supported CodeGraph node-kind value.");
  }
  const limit = integerInRange(values.limit, 20, 1, 50, "limit");
  let scope;
  if (values.pathPrefix !== undefined) {
    scope = normalizeRelative(values.pathPrefix, "pathPrefix", true);
    confinedPath(project.root, scope, "directory", "unsupported_scope");
  }
  const freshness = projectFreshness(await runCodeGraph(config, ["status", project.root, "--json"]));
  if (!freshness.initialized) {
    throw new ApplicationError("codegraph_uninitialized", "CodeGraph is not initialized for the selected project.");
  }
  const candidateLimit = scope === undefined ? limit : CODEGRAPH_CANDIDATE_LIMIT;
  const command = ["query", "-p", project.root, "--limit", String(candidateLimit), "--json"];
  if (kind !== undefined) command.push("--kind", kind);
  command.push("--", search);
  const raw = await runCodeGraph(config, command);
  if (!Array.isArray(raw)) {
    throw new ApplicationError("codegraph_output_invalid", "CodeGraph query output is not a result array.");
  }
  const candidates = raw.map(projectNode);
  const filtered =
    scope === undefined || scope === "."
      ? candidates
      : candidates.filter((node) => node.filePath === scope || node.filePath.startsWith(`${scope}/`));
  const results = filtered.slice(0, limit);
  const providerTruncated = raw.length >= candidateLimit;
  const resultTruncated = filtered.length > limit;
  const complete = !providerTruncated && !resultTruncated;
  const pending = freshness.pendingChanges.added + freshness.pendingChanges.modified + freshness.pendingChanges.removed;
  const structured = {
    source: { provider: "CodeGraph", project: project.id, root: project.root },
    search,
    kind: kind ?? null,
    pathPrefix: scope ?? null,
    results,
    returned: results.length,
    candidateCount: candidates.length,
    providerTruncated,
    resultTruncated,
    complete,
    freshness,
  };
  return {
    text:
      `CodeGraph returned ${results.length} scoped symbols from ${candidates.length} bounded candidates.` +
      (complete ? "" : " The result is explicitly incomplete because a candidate or result limit was reached.") +
      (pending > 0 || freshness.worktreeMismatch !== null || freshness.reindexRecommended
        ? " The index has freshness warnings; verify decision-bearing claims against current source."
        : " The index reports no pending changes; still verify decision-bearing claims against current source."),
    structured,
  };
}

async function callTool(id, name, args) {
  try {
    const config = requireConfiguration();
    if (name === pytestTool.name && config.projects.size > 0) {
      const result = await summarizePytest(args, config);
      toolResult(id, result.text, result.structured);
      return;
    }
    if (name === memorySearchTool.name && config.memoryRoot !== undefined) {
      const result = searchMemory(args, config);
      toolResult(id, result.text, result.structured);
      return;
    }
    if (name === memoryReadTool.name && config.memoryRoot !== undefined) {
      const result = readMemory(args, config);
      toolResult(id, result.text, result.structured);
      return;
    }
    if (name === codegraphTool.name && config.codegraph !== undefined) {
      const result = await queryCodeGraph(args, config);
      toolResult(id, result.text, result.structured);
      return;
    }
    if (name === publicHistoryTool.name && config.historyRoot !== undefined) {
      const result = readPublicHistory(args, config);
      toolResult(id, result.text, result.structured);
      return;
    }
    throw new ApplicationError("unknown_tool", "Unknown or unconfigured Web workflow tool.");
  } catch (error) {
    applicationFailure(id, error);
  }
}

async function handleLine(line) {
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    console.error("received malformed JSON");
    return;
  }
  const { id, method, params = {} } = request;
  if (request.jsonrpc !== "2.0" || id === undefined) {
    rpcError(id ?? null, -32600, "invalid request");
    return;
  }
  if (method === "initialize") {
    if (params.protocolVersion !== PROTOCOL_VERSION) {
      rpcError(id, -32602, "unsupported protocol version");
      return;
    }
    send({ jsonrpc: "2.0", id, result: { protocolVersion: PROTOCOL_VERSION } });
    return;
  }
  if (method === "tools/list") {
    try {
      send({ jsonrpc: "2.0", id, result: { tools: availableTools() } });
    } catch (error) {
      rpcError(id, -32603, error instanceof Error ? error.message : "invalid Plugin configuration");
    }
    return;
  }
  if (method === "tools/call") {
    await callTool(id, params.name, params.arguments ?? {});
    return;
  }
  rpcError(id, -32601, "method not found");
}

const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity, terminal: false });
let queue = Promise.resolve();
input.on("line", (line) => {
  queue = queue.then(() => handleLine(line)).catch((error) => {
    console.error(`web workflow adapter failure: ${error?.message ?? error}`);
  });
});
