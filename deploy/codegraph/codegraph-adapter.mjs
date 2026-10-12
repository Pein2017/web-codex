#!/usr/bin/env node

// WebCodex Native Tool Plugin adapter for CodeGraph CLI.
// stdout is reserved for one-line JSON-RPC protocol messages; diagnostics go to stderr.

import { execFile } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const PROTOCOL_VERSION = "webcodex-plugin-v1";
const CODEGRAPH_RUNTIME =
  "/data/CoordExp/codex-tools/web-codex/dependencies/codegraph/node_modules/@colbymchenry/codegraph-linux-x64/node";
const CODEGRAPH_ENTRY =
  "/data/CoordExp/codex-tools/web-codex/dependencies/codegraph/node_modules/@colbymchenry/codegraph-linux-x64/lib/dist/bin/codegraph.js";
const ALLOWED_ROOT = "/data/CoordExp";
const EXTERNAL_SOURCE_ROOTS = new Set([
  "/data/ms-swift/swift",
  "/root/miniconda3/envs/ms/lib/python3.12/site-packages/transformers",
  "/root/miniconda3/envs/ms/lib/python3.12/site-packages/vllm",
]);
const CALL_TIMEOUT_MS = 110_000;
const CHILD_BUFFER_BYTES = 256 * 1024;
const RESULT_TEXT_BYTES = 60 * 1024;

const stringProperty = (description, maxLength = 4096) => ({
  type: "string",
  description,
  minLength: 1,
  maxLength,
});

const integerProperty = (description) => ({
  type: "integer",
  description,
});

const projectProperty = stringProperty(
  "Absolute Git project root under /data/CoordExp, or an explicitly allowed external source root: /data/ms-swift/swift, /root/miniconda3/envs/ms/lib/python3.12/site-packages/transformers, /root/miniconda3/envs/ms/lib/python3.12/site-packages/vllm.",
  4096,
);

const readOnlyAnnotations = {
  readOnlyHint: true,
  destructiveHint: false,
  openWorldHint: false,
};

const tools = [
  {
    name: "codegraph_explore",
    title: "Explore code relationships",
    description:
      "Find relevant symbols, source, and call paths for a focused question. Use projectPath for the exact checkout or worktree. Verify decision-bearing claims against source because the graph can be stale or incomplete.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        query: stringProperty("Focused code relationship question.", 8192),
        maxFiles: integerProperty("Maximum source files to include; 1 through 20."),
      },
      required: ["projectPath", "query"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_node",
    title: "Inspect a symbol or indexed file",
    description:
      "Show one symbol with callers/callees, or inspect an indexed project-relative file. Provide name, file, or both; file disambiguates a symbol when both are supplied.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        name: stringProperty("Symbol name to inspect.", 2048),
        file: stringProperty("Project-relative file path.", 4096),
        offset: integerProperty("Optional 1-based start line in file mode."),
        limit: integerProperty("Optional maximum lines in file mode; 1 through 1000."),
        symbolsOnly: {
          type: "boolean",
          description: "In file mode, return only the symbol map and dependents.",
        },
      },
      required: ["projectPath"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_context",
    title: "Build task context",
    description:
      "Build bounded code context for a task from indexed symbols and relationships.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        task: stringProperty("Focused engineering or investigation task.", 8192),
        maxNodes: integerProperty("Maximum symbols to include; 1 through 100."),
        includeCode: {
          type: "boolean",
          description: "Include source blocks; defaults to true.",
        },
      },
      required: ["projectPath", "task"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_query",
    title: "Search indexed symbols",
    description: "Search symbols in the CodeGraph index with an optional kind filter.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        search: stringProperty("Symbol search text.", 4096),
        limit: integerProperty("Maximum results; 1 through 100."),
        kind: stringProperty("Optional node kind such as function, class, or method.", 128),
      },
      required: ["projectPath", "search"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_callers",
    title: "Find symbol callers",
    description: "Find indexed functions or methods that call a symbol.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        symbol: stringProperty("Exact or resolvable symbol name.", 2048),
        limit: integerProperty("Maximum results; 1 through 100."),
      },
      required: ["projectPath", "symbol"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_callees",
    title: "Find symbol callees",
    description: "Find indexed functions or methods called by a symbol.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        symbol: stringProperty("Exact or resolvable symbol name.", 2048),
        limit: integerProperty("Maximum results; 1 through 100."),
      },
      required: ["projectPath", "symbol"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_impact",
    title: "Analyze symbol impact",
    description: "Traverse indexed relationships to estimate code affected by changing a symbol.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        symbol: stringProperty("Exact or resolvable symbol name.", 2048),
        depth: integerProperty("Traversal depth; 1 through 8."),
      },
      required: ["projectPath", "symbol"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_affected",
    title: "Find affected tests",
    description: "Find indexed test files affected by a bounded list of changed project-relative files.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        files: {
          type: "array",
          description: "Changed project-relative file paths.",
          minItems: 1,
          maxItems: 64,
          items: stringProperty("Project-relative file path.", 4096),
        },
        depth: integerProperty("Dependency traversal depth; 1 through 10."),
        filter: stringProperty("Optional glob selecting test files.", 1024),
      },
      required: ["projectPath", "files"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_files",
    title: "List indexed files",
    description: "Show a bounded view of the indexed project file structure.",
    inputSchema: {
      type: "object",
      properties: {
        projectPath: projectProperty,
        filter: stringProperty("Optional project-relative directory filter.", 4096),
        pattern: stringProperty("Optional file glob.", 1024),
        maxDepth: integerProperty("Maximum tree depth; 1 through 20."),
      },
      required: ["projectPath"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_status",
    title: "Inspect CodeGraph index status",
    description: "Return JSON status, freshness, size, and pending-change evidence for a project index.",
    inputSchema: {
      type: "object",
      properties: { projectPath: projectProperty },
      required: ["projectPath"],
      additionalProperties: false,
    },
    annotations: readOnlyAnnotations,
  },
  {
    name: "codegraph_sync",
    title: "Synchronize CodeGraph index",
    description:
      "Update an existing project's .codegraph index from current files. This changes index state but not tracked source files; it does not initialize a missing index.",
    inputSchema: {
      type: "object",
      properties: { projectPath: projectProperty },
      required: ["projectPath"],
      additionalProperties: false,
    },
    annotations: {
      readOnlyHint: false,
      destructiveHint: false,
      idempotentHint: true,
      openWorldHint: false,
    },
  },
];

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function rpcError(id, code, message) {
  send({ jsonrpc: "2.0", id, error: { code, message } });
}

function toolResult(id, text, isError = false) {
  send({
    jsonrpc: "2.0",
    id,
    result: { content: [{ type: "text", text }], isError },
  });
}

function boundedText(value) {
  const source = String(value ?? "").replaceAll("\u0000", "");
  const bytes = Buffer.from(source);
  if (bytes.length <= RESULT_TEXT_BYTES) return source;
  const kept = bytes.subarray(0, RESULT_TEXT_BYTES - 160).toString("utf8");
  return `${kept}\n\n[WebCodex CodeGraph adapter truncated output at ${RESULT_TEXT_BYTES} bytes; narrow the query or limits.]`;
}

function integerInRange(value, fallback, min, max, field) {
  if (value === undefined) return fallback;
  if (!Number.isInteger(value) || value < min || value > max) {
    throw new Error(`${field} must be an integer from ${min} through ${max}`);
  }
  return value;
}

function projectRoot(requested) {
  if (typeof requested !== "string" || requested.length === 0) {
    throw new Error("projectPath must be a non-empty string");
  }
  const resolved = fs.realpathSync(requested);
  const externalSource = EXTERNAL_SOURCE_ROOTS.has(resolved);
  const relative = path.relative(ALLOWED_ROOT, resolved);
  if (!externalSource && (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative))) {
    throw new Error("projectPath must resolve under /data/CoordExp or equal an explicitly allowed external source root");
  }
  if (!fs.statSync(resolved).isDirectory()) {
    throw new Error("projectPath must resolve to a directory");
  }
  if (!externalSource && !fs.existsSync(path.join(resolved, ".git"))) {
    throw new Error("projectPath must be a Git checkout or worktree root");
  }
  return resolved;
}

function relativeFile(value, field) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\u0000")) {
    throw new Error(`${field} must be a non-empty path`);
  }
  const normalized = path.posix.normalize(value.replaceAll("\\", "/"));
  if (normalized === ".." || normalized.startsWith("../") || path.posix.isAbsolute(normalized)) {
    throw new Error(`${field} must be project-relative and cannot escape the project`);
  }
  return normalized;
}

function commandFor(name, args) {
  const project = projectRoot(args.projectPath);
  switch (name) {
    case "codegraph_explore": {
      const maxFiles = integerInRange(args.maxFiles, 8, 1, 20, "maxFiles");
      return ["explore", "-p", project, "--max-files", String(maxFiles), args.query];
    }
    case "codegraph_node": {
      if (!args.name && !args.file) throw new Error("codegraph_node requires name, file, or both");
      const command = ["node", "-p", project];
      if (args.file) command.push("--file", relativeFile(args.file, "file"));
      if (args.offset !== undefined) {
        command.push("--offset", String(integerInRange(args.offset, 1, 1, 10_000_000, "offset")));
      }
      if (args.limit !== undefined) {
        command.push("--limit", String(integerInRange(args.limit, 200, 1, 1000, "limit")));
      }
      if (args.symbolsOnly === true) command.push("--symbols-only");
      if (args.name) command.push(args.name);
      return command;
    }
    case "codegraph_context": {
      const maxNodes = integerInRange(args.maxNodes, 20, 1, 100, "maxNodes");
      const command = ["context", "-p", project, "--max-nodes", String(maxNodes)];
      if (args.includeCode === false) command.push("--no-code");
      command.push(args.task);
      return command;
    }
    case "codegraph_query": {
      const limit = integerInRange(args.limit, 10, 1, 100, "limit");
      const command = ["query", "-p", project, "--limit", String(limit)];
      if (args.kind) command.push("--kind", args.kind);
      command.push(args.search);
      return command;
    }
    case "codegraph_callers":
    case "codegraph_callees": {
      const subcommand = name.slice("codegraph_".length);
      const limit = integerInRange(args.limit, 20, 1, 100, "limit");
      return [subcommand, "-p", project, "--limit", String(limit), args.symbol];
    }
    case "codegraph_impact": {
      const depth = integerInRange(args.depth, 2, 1, 8, "depth");
      return ["impact", "-p", project, "--depth", String(depth), args.symbol];
    }
    case "codegraph_affected": {
      const depth = integerInRange(args.depth, 5, 1, 10, "depth");
      const command = ["affected", "-p", project, "--depth", String(depth)];
      if (args.filter) command.push("--filter", args.filter);
      command.push(...args.files.map((file) => relativeFile(file, "files item")));
      return command;
    }
    case "codegraph_files": {
      const command = ["files", "-p", project, "--format", "tree"];
      if (args.filter) command.push("--filter", relativeFile(args.filter, "filter"));
      if (args.pattern) command.push("--pattern", args.pattern);
      if (args.maxDepth !== undefined) {
        command.push("--max-depth", String(integerInRange(args.maxDepth, 6, 1, 20, "maxDepth")));
      }
      return command;
    }
    case "codegraph_status":
      return ["status", project, "--json"];
    case "codegraph_sync":
      return ["sync", project];
    default:
      throw new Error("unknown CodeGraph tool");
  }
}

async function callTool(id, name, args) {
  try {
    const cwd = projectRoot(args?.projectPath);
    const command = commandFor(name, args ?? {});
    const { stdout, stderr } = await execFileAsync(CODEGRAPH_RUNTIME, [
      "--liftoff-only",
      "--disable-warning=ExperimentalWarning",
      CODEGRAPH_ENTRY,
      ...command,
    ], {
      cwd,
      encoding: "utf8",
      timeout: CALL_TIMEOUT_MS,
      maxBuffer: CHILD_BUFFER_BYTES,
      killSignal: "SIGKILL",
      env: { ...process.env, NO_COLOR: "1" },
    });
    const output = stdout.trim() || stderr.trim() || "CodeGraph completed without output.";
    toolResult(id, boundedText(output), false);
  } catch (error) {
    const detail = error?.stderr || error?.stdout || error?.message || "CodeGraph call failed";
    toolResult(id, boundedText(`CodeGraph call failed: ${detail}`), true);
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
    send({ jsonrpc: "2.0", id, result: { tools } });
    return;
  }
  if (method === "tools/call") {
    await callTool(id, params.name, params.arguments);
    return;
  }
  rpcError(id, -32601, "method not found");
}

const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
let queue = Promise.resolve();
input.on("line", (line) => {
  queue = queue.then(() => handleLine(line)).catch((error) => {
    console.error(`adapter failure: ${error?.message ?? error}`);
  });
});
