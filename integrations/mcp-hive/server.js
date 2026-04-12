#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { Bash, OverlayFs, ReadWriteFs } from "just-bash";
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";

function compact(text, max = 120) {
  const clean = String(text ?? "").replace(/\s+/g, " ").trim();
  if (clean.length <= max) return clean;
  return `${clean.slice(0, max - 1)}…`;
}

function readJson(filePath, fallback = null) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch {
    return fallback;
  }
}

function resolveBundleRoot() {
  const explicit = process.env.MEMD_BUNDLE_ROOT?.trim();
  if (explicit) return path.resolve(explicit);
  return path.resolve(process.cwd(), ".memd");
}

function resolveMemdBin() {
  return (process.env.MEMD_BIN || "memd").trim();
}

function readRuntime(bundleRoot) {
  return readJson(path.join(bundleRoot, "config.json"), {});
}

function readHeartbeat(bundleRoot) {
  return readJson(path.join(bundleRoot, "state", "heartbeat.json"), {});
}

function readResume(bundleRoot) {
  return readJson(path.join(bundleRoot, "state", "last-resume.json"), {});
}

function bundleContext(bundleRoot) {
  const runtime = readRuntime(bundleRoot);
  const heartbeat = readHeartbeat(bundleRoot);
  const resume = readResume(bundleRoot);
  return { runtime, heartbeat, resume };
}

function detectPresence(heartbeat) {
  if (!heartbeat?.last_seen) return "unknown";
  const ageMs = Date.now() - new Date(heartbeat.last_seen).getTime();
  if (ageMs <= 120_000) return "active";
  if (ageMs <= 900_000) return "stale";
  return "dead";
}

function listHiveBundles(bundleRoot) {
  const projectRoot = path.dirname(bundleRoot);
  const projectsRoot = path.dirname(projectRoot);
  const hives = [];

  for (const entry of fs.readdirSync(projectsRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const candidateProject = path.join(projectsRoot, entry.name);
    const candidateBundle = path.join(candidateProject, ".memd");
    if (!fs.existsSync(path.join(candidateBundle, "config.json"))) continue;

    const runtime = readRuntime(candidateBundle);
    const heartbeat = readHeartbeat(candidateBundle);
    const resume = readResume(candidateBundle);

    hives.push({
      project_dir: candidateProject,
      bundle_root: candidateBundle,
      project: runtime.project ?? null,
      namespace: runtime.namespace ?? null,
      agent: runtime.agent ?? null,
      session: runtime.session ?? null,
      effective_agent:
        runtime.agent && runtime.session
          ? `${runtime.agent}@${runtime.session}`
          : runtime.agent ?? null,
      base_url: runtime.base_url ?? null,
      workspace: heartbeat.workspace ?? runtime.workspace ?? null,
      visibility: heartbeat.visibility ?? runtime.visibility ?? null,
      focus: heartbeat.focus ?? resume.focus ?? null,
      pressure: heartbeat.pressure ?? resume.pressure ?? null,
      next_recovery: heartbeat.next_recovery ?? resume.next_recovery ?? null,
      presence: detectPresence(heartbeat),
      host: heartbeat.host ?? null,
      pid: heartbeat.pid ?? null,
    });
  }

  return hives;
}

function matchesIdentityScope(hive, identity) {
  return (
    hive.project === (identity.project ?? null) &&
    hive.namespace === (identity.namespace ?? null) &&
    hive.workspace === (identity.workspace ?? null)
  );
}

function recommendBoundaries(tasks = []) {
  const lines = [];
  for (const task of tasks) {
    const branchPrefix =
      task.coordination_mode === "shared_review"
        ? "review"
        : task.coordination_mode === "help_only"
          ? "help"
          : "feat";
    const branchSuffix = String(task.task_id || "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "");
    const scopeHint =
      Array.isArray(task.claim_scopes) && task.claim_scopes.length > 0
        ? task.claim_scopes.join(", ")
        : "define a narrower scope";
    lines.push(
      `${task.task_id} [${task.coordination_mode}] -> ${branchPrefix}/${branchSuffix || "task"} | scopes ${scopeHint}`
    );
  }
  return lines;
}

function policyConflicts(tasks = [], claims = []) {
  const lines = [];
  for (const task of tasks) {
    if (task.coordination_mode !== "exclusive_write") continue;
    for (const scope of task.claim_scopes ?? []) {
      const claim = claims.find((entry) => entry.scope === scope);
      const claimOwner = claim?.session ?? null;
      const taskOwner = task?.session ?? null;
      if (claimOwner && claimOwner !== taskOwner) {
        lines.push(
          `task ${task.task_id} requires exclusive_write but scope ${scope} is held by ${claim.effective_agent || claim.session || "none"}`
        );
      }
    }
  }
  return lines;
}

async function memdGet(baseUrl, route, query = null) {
  const url = new URL(route, baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`);
  if (query) {
    for (const [key, value] of Object.entries(query)) {
      if (value === undefined || value === null || value === "") continue;
      url.searchParams.set(key, String(value));
    }
  }
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`memd GET ${route} failed: ${response.status} ${await response.text()}`);
  }
  return response.json();
}

async function memdPost(baseUrl, route, body) {
  const url = new URL(route, baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`);
  const response = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error(`memd POST ${route} failed: ${response.status} ${await response.text()}`);
  }
  return response.json();
}

function currentIdentity(bundleRoot) {
  const { runtime, heartbeat, resume } = bundleContext(bundleRoot);
  const session = runtime.session ?? null;
  const agent = runtime.agent ?? null;
  const effectiveAgent = agent && session ? `${agent}@${session}` : agent;
  return {
    bundleRoot,
    baseUrl: runtime.base_url ?? "http://100.104.154.24:8787",
    project: runtime.project ?? null,
    namespace: runtime.namespace ?? null,
    workspace: heartbeat.workspace ?? runtime.workspace ?? null,
    visibility: heartbeat.visibility ?? runtime.visibility ?? null,
    session,
    agent,
    effectiveAgent,
    host: heartbeat.host ?? os.hostname(),
    pid: heartbeat.pid ?? process.pid,
    focus: heartbeat.focus ?? resume.focus ?? null,
    pressure: heartbeat.pressure ?? resume.pressure ?? null,
  };
}

function textResult(lines) {
  return {
    content: [{ type: "text", text: Array.isArray(lines) ? lines.join("\n") : String(lines) }],
  };
}

function normalizeToolRoot(bundleRoot, root = "project") {
  switch (root) {
    case "bundle":
      return bundleRoot;
    case "integration":
      return path.resolve(bundleRoot, "..", "integrations", "mcp-hive");
    case "project":
    default:
      return path.resolve(bundleRoot, "..");
  }
}

function toVirtualCwd(mountPoint, cwd) {
  if (!cwd) return mountPoint;
  const normalized = String(cwd).replace(/\\/g, "/");
  const rooted = normalized.startsWith("/") ? normalized : `${mountPoint}/${normalized}`;
  const resolved = path.posix.normalize(rooted);
  if (resolved !== mountPoint && !resolved.startsWith(`${mountPoint}/`)) {
    throw new Error(`cwd must stay within ${mountPoint}`);
  }
  return resolved;
}

async function execVirtualBash(bundleRoot, options = {}) {
  const root = normalizeToolRoot(bundleRoot, options.root);
  const workspaceFs = options.allow_write
    ? new ReadWriteFs({ root })
    : new OverlayFs({ root, readOnly: false });
  const mountPoint = workspaceFs.getMountPoint();
  const bash = new Bash({
    fs: workspaceFs,
    cwd: toVirtualCwd(mountPoint, options.cwd),
  });
  const timeoutMs = Math.max(100, Math.min(Number(options.timeout_ms) || 5000, 30000));
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const result = await bash.exec(String(options.script ?? ""), {
      env:
        options.env && typeof options.env === "object" && !Array.isArray(options.env)
          ? Object.fromEntries(
              Object.entries(options.env).map(([key, value]) => [String(key), String(value)])
            )
          : undefined,
      stdin: options.stdin == null ? undefined : String(options.stdin),
      signal: controller.signal,
    });

    return textResult([
      `root=${root}`,
      `mode=${options.allow_write ? "read-write" : "overlay"}`,
      `cwd=${result.env?.PWD ?? toVirtualCwd(mountPoint, options.cwd)}`,
      `exit_code=${result.exitCode}`,
      "stdout<<EOF",
      result.stdout || "",
      "EOF",
      "stderr<<EOF",
      result.stderr || "",
      "EOF",
    ]);
  } catch (error) {
    if (controller.signal.aborted) {
      throw new Error(`bash_exec timed out after ${timeoutMs}ms`);
    }
    throw error;
  } finally {
    clearTimeout(timer);
  }
}

const bundleRoot = resolveBundleRoot();
const identity = currentIdentity(bundleRoot);

const server = new Server(
  { name: "memd-hive-mcp", version: "0.1.0" },
  {
    capabilities: { tools: {} },
    instructions:
      "Use memd hive coordination tools for coworking. Prefer explicit claims and assignments over implicit overlap. Treat memd as the coordination source of truth.",
  }
);

const tools = [
  {
    name: "list_hives",
    description: "List other live or recent memd hive sessions discovered from sibling bundles.",
    inputSchema: {
      type: "object",
      properties: {
        include_current: { type: "boolean", default: false },
        active_only: { type: "boolean", default: true },
      },
    },
  },
  {
    name: "check_inbox",
    description: "Read pending hive messages for the current session.",
    inputSchema: {
      type: "object",
      properties: {
        include_acknowledged: { type: "boolean", default: false },
      },
    },
  },
  {
    name: "coordination_inbox",
    description: "Read the compact coordination inbox for the current session, including messages and shared-task pressure.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "coordination_dashboard",
    description: "Render a compact dashboard-style view of current coordination pressure and recent history.",
    inputSchema: {
      type: "object",
      properties: {
        view: {
          type: "string",
          description: "all, inbox, requests, recovery, policy, suggestions, or history",
          default: "all",
        },
      },
    },
  },
  {
    name: "coordination_changes",
    description: "Read a compact reusable coordination delta feed for hooks or richer operator surfaces.",
    inputSchema: {
      type: "object",
      properties: {
        view: {
          type: "string",
          description: "all, inbox, requests, recovery, policy, suggestions, or history",
          default: "all",
        },
      },
    },
  },
  {
    name: "coordination_suggestions",
    description: "Read policy-aware bounded coordination suggestions derived from current pressure and policy signals.",
    inputSchema: {
      type: "object",
      properties: {
        view: {
          type: "string",
          description: "all, policy, suggestions, or history",
          default: "suggestions",
        },
      },
    },
  },
  {
    name: "coordination_action",
    description: "Execute a bounded coordination action through one operator-facing entrypoint.",
    inputSchema: {
      type: "object",
      properties: {
        action: {
          type: "string",
          description: "ack_message, assign_scope, recover_session, request_help, or request_review",
        },
        message_id: { type: "string" },
        target_session: { type: "string" },
        stale_session: { type: "string" },
        scope: { type: "string" },
        content: { type: "string" },
      },
      required: ["action"],
    },
  },
  {
    name: "recover_stale_session",
    description: "Recover claims and shared tasks from a stale or dead session into the current session or another target session.",
    inputSchema: {
      type: "object",
      properties: {
        stale_session: { type: "string" },
        target_session: { type: "string" },
      },
      required: ["stale_session"],
    },
  },
  {
    name: "recommend_boundaries",
    description: "Recommend cleaner branch and scope boundaries for active shared tasks.",
    inputSchema: {
      type: "object",
      properties: {},
    },
  },
  {
    name: "send_message",
    description: "Send a direct coordination message to another session.",
    inputSchema: {
      type: "object",
      properties: {
        target_session: { type: "string" },
        kind: { type: "string", default: "handoff" },
        content: { type: "string" },
      },
      required: ["target_session", "content"],
    },
  },
  {
    name: "ack_message",
    description: "Acknowledge a hive message addressed to the current session.",
    inputSchema: {
      type: "object",
      properties: {
        id: { type: "string" },
      },
      required: ["id"],
    },
  },
  {
    name: "list_claims",
    description: "List active hive claims for the current project/namespace.",
    inputSchema: {
      type: "object",
      properties: {
        active_only: { type: "boolean", default: true },
      },
    },
  },
  {
    name: "acquire_claim",
    description: "Acquire a scoped claim for the current session.",
    inputSchema: {
      type: "object",
      properties: {
        scope: { type: "string" },
        ttl_seconds: { type: "number", default: 900 },
      },
      required: ["scope"],
    },
  },
  {
    name: "release_claim",
    description: "Release a scoped claim held by the current session.",
    inputSchema: {
      type: "object",
      properties: {
        scope: { type: "string" },
      },
      required: ["scope"],
    },
  },
  {
    name: "transfer_claim",
    description: "Transfer a scoped claim from the current session to another session.",
    inputSchema: {
      type: "object",
      properties: {
        scope: { type: "string" },
        target_session: { type: "string" },
      },
      required: ["scope", "target_session"],
    },
  },
  {
    name: "assign_work",
    description: "Assign a scoped unit of work by transferring the claim and sending an assignment message.",
    inputSchema: {
      type: "object",
      properties: {
        scope: { type: "string" },
        target_session: { type: "string" },
        content: { type: "string" },
      },
      required: ["scope", "target_session"],
    },
  },
  {
    name: "list_tasks",
    description: "List shared tasks for the current project and workspace.",
    inputSchema: {
      type: "object",
      properties: {
        active_only: { type: "boolean", default: true },
      },
    },
  },
  {
    name: "upsert_task",
    description: "Create or update a shared task backed by memd coordination state.",
    inputSchema: {
      type: "object",
      properties: {
        task_id: { type: "string" },
        title: { type: "string" },
        description: { type: "string" },
        status: { type: "string", default: "active" },
        coordination_mode: { type: "string", default: "exclusive_write" },
        claim_scopes: {
          type: "array",
          items: { type: "string" },
          default: [],
        },
      },
      required: ["task_id", "title"],
    },
  },
  {
    name: "assign_task",
    description: "Assign a shared task to another session.",
    inputSchema: {
      type: "object",
      properties: {
        task_id: { type: "string" },
        target_session: { type: "string" },
      },
      required: ["task_id", "target_session"],
    },
  },
  {
    name: "request_task_help",
    description: "Mark a shared task as needing help and notify another session.",
    inputSchema: {
      type: "object",
      properties: {
        task_id: { type: "string" },
        target_session: { type: "string" },
        title: { type: "string" },
        description: { type: "string" },
      },
      required: ["task_id", "target_session"],
    },
  },
  {
    name: "request_task_review",
    description: "Mark a shared task as needing review and notify another session.",
    inputSchema: {
      type: "object",
      properties: {
        task_id: { type: "string" },
        target_session: { type: "string" },
        title: { type: "string" },
        description: { type: "string" },
      },
      required: ["task_id", "target_session"],
    },
  },
  {
    name: "bash_exec",
    description:
      "Run a command in an isolated just-bash shell rooted at the project, bundle, or integration directory. Default is overlay mode, so writes do not touch disk.",
    inputSchema: {
      type: "object",
      properties: {
        script: { type: "string" },
        root: {
          type: "string",
          description: "project, bundle, or integration",
          default: "project",
        },
        cwd: {
          type: "string",
          description: "Optional working directory inside the selected root.",
        },
        stdin: { type: "string" },
        timeout_ms: { type: "number", default: 5000 },
        allow_write: {
          type: "boolean",
          default: false,
          description: "When true, writes go to disk inside the selected root.",
        },
        env: {
          type: "object",
          additionalProperties: { type: "string" },
        },
      },
      required: ["script"],
    },
  },
];

server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools }));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const args = request.params.arguments ?? {};
  const hives = listHiveBundles(bundleRoot);
  const hiveBySession = new Map(hives.map((hive) => [hive.session, hive]));

  switch (request.params.name) {
    case "bash_exec": {
      return execVirtualBash(bundleRoot, args);
    }

    case "list_hives": {
      const includeCurrent = Boolean(args.include_current);
      const activeOnly = args.active_only !== false;
      const rows = hives
        .filter((hive) => includeCurrent || hive.bundle_root !== bundleRoot)
        .filter((hive) => !activeOnly || hive.presence === "active")
        .map(
          (hive) =>
            `- ${hive.effective_agent ?? hive.session ?? "unknown"} | presence=${hive.presence} | workspace=${hive.workspace ?? "none"} | focus="${compact(hive.focus ?? "none", 80)}"`
        );
      return textResult([
        `bundle=${bundleRoot}`,
        `current=${identity.effectiveAgent ?? "unknown"}`,
        `hives=${rows.length}`,
        ...rows,
      ]);
    }

    case "check_inbox": {
      if (!identity.session) throw new Error("Bundle session is required for inbox access.");
      const response = await memdGet(identity.baseUrl, "/coordination/messages/inbox", {
        session: identity.session,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        include_acknowledged: args.include_acknowledged === true,
        limit: 64,
      });
      const rows = (response.messages ?? []).map(
        (message) =>
          `- ${message.id.slice(0, 8)} [${message.kind}] ${message.from_agent ?? message.from_session} -> ${message.to_session} | ${compact(message.content, 100)}`
      );
      return textResult([`messages=${rows.length}`, ...rows]);
    }

    case "coordination_inbox": {
      if (!identity.session) throw new Error("Bundle session is required for coordination inbox.");
      const response = await memdGet(identity.baseUrl, "/coordination/inbox", {
        session: identity.session,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        limit: 64,
      });
      const lines = [
        `messages=${response.messages?.length ?? 0}`,
        `owned=${response.owned_tasks?.length ?? 0}`,
        `help=${response.help_tasks?.length ?? 0}`,
        `review=${response.review_tasks?.length ?? 0}`,
      ];
      for (const message of response.messages ?? []) {
        lines.push(`- msg ${message.id.slice(0, 8)} [${message.kind}] ${compact(message.content, 100)}`);
      }
      for (const task of response.owned_tasks ?? []) {
        lines.push(`- own ${task.task_id} [${task.status}] ${compact(task.title, 96)}`);
      }
      return textResult(lines);
    }

    case "coordination_dashboard": {
      if (!identity.session) throw new Error("Bundle session is required for coordination dashboard.");
      const view = args.view || "all";
      const inbox = await memdGet(identity.baseUrl, "/coordination/inbox", {
        session: identity.session,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        limit: 32,
      });
      const receipts = await memdGet(identity.baseUrl, "/coordination/receipts", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        limit: 12,
      });
      const claims = await memdGet(identity.baseUrl, "/coordination/claims", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        active_only: true,
        limit: 256,
      });
      const tasks = await memdGet(identity.baseUrl, "/coordination/tasks", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        active_only: true,
        limit: 256,
      });
      const hives = listHiveBundles(bundleRoot).filter((hive) => matchesIdentityScope(hive, identity));
      const staleHives = hives.filter(
        (hive) => hive.session && hive.session !== identity.session && (hive.presence === "stale" || hive.presence === "dead")
      );
      const reclaimableClaims = (claims.claims ?? []).filter((claim) =>
        staleHives.some((hive) => hive.session === claim.session)
      );
      const stalledTasks = (tasks.tasks ?? []).filter((task) =>
        staleHives.some((hive) => hive.session === task.session)
      );
      const conflicts = policyConflicts(tasks.tasks ?? [], claims.claims ?? []);
      const recommendations = recommendBoundaries(tasks.tasks ?? []);
      const lines = [
        "## Coordination",
        `messages=${inbox.messages?.length ?? 0} owned=${inbox.owned_tasks?.length ?? 0} help=${inbox.help_tasks?.length ?? 0} review=${inbox.review_tasks?.length ?? 0}`,
        `recovery stale=${staleHives.length} reclaimable=${reclaimableClaims.length} stalled=${stalledTasks.length}`,
        `policy conflicts=${conflicts.length} recommendations=${recommendations.length} receipts=${receipts.receipts?.length ?? 0}`,
      ];
      const showAll = view === "all" || view === "overview";
      if (showAll || view === "inbox") {
        lines.push("", "## Inbox");
        for (const message of inbox.messages ?? []) {
          lines.push(`- ${message.kind}: ${compact(message.content, 96)}`);
        }
        for (const task of inbox.owned_tasks ?? []) {
          lines.push(`- own ${task.task_id}: ${compact(task.title, 96)}`);
        }
      }
      if (showAll || view === "requests") {
        lines.push("", "## Requests");
        for (const task of inbox.help_tasks ?? []) {
          lines.push(`- help ${task.task_id}: owner=${task.effective_agent || task.session || "none"}`);
        }
        for (const task of inbox.review_tasks ?? []) {
          lines.push(`- review ${task.task_id}: owner=${task.effective_agent || task.session || "none"}`);
        }
      }
      if (showAll || view === "recovery") {
        lines.push("", "## Recovery");
        for (const hive of staleHives) {
          lines.push(
            `- stale session=${hive.session || "none"} agent=${hive.effective_agent || hive.agent || "none"} presence=${hive.presence} focus="${compact(hive.focus || "none", 72)}"`
          );
        }
        for (const claim of reclaimableClaims) {
          lines.push(`- reclaimable claim ${claim.scope}: owner=${claim.effective_agent || claim.session || "none"}`);
        }
        for (const task of stalledTasks) {
          lines.push(`- stalled task ${task.task_id}: owner=${task.effective_agent || task.session || "none"}`);
        }
      }
      if (showAll || view === "policy") {
        lines.push("", "## Policy");
        for (const line of conflicts) {
          lines.push(`- policy ${compact(line, 96)}`);
        }
        for (const line of recommendations) {
          lines.push(`- recommend ${compact(line, 96)}`);
        }
      }
      if (showAll || view === "history") {
        lines.push("", "## Recent Receipts");
        for (const receipt of receipts.receipts ?? []) {
          lines.push(`- ${receipt.kind}: ${compact(receipt.summary, 96)}`);
        }
      }
      return textResult(lines);
    }

    case "coordination_changes": {
      const view = args.view || "all";
      const stdout = execFileSync(
        resolveMemdBin(),
        [
          "coordination",
          "--output",
          bundleRoot,
          "--changes-only",
          "--summary",
          "--view",
          view,
        ],
        {
          encoding: "utf8",
          env: process.env,
        }
      );
      return textResult(stdout.trim().split("\n"));
    }
    case "coordination_suggestions": {
      const view = args.view || "suggestions";
      const stdout = execFileSync(
        resolveMemdBin(),
        ["coordination", "--output", bundleRoot, "--summary", "--view", view],
        {
          encoding: "utf8",
          env: process.env,
        }
      );
      const lines = stdout.trim().split("\n");
      const suggestions = [];
      let inSuggestions = false;
      for (const line of lines) {
        if (line.startsWith("## Suggestions")) {
          inSuggestions = true;
          continue;
        }
        if (line.startsWith("## ")) {
          inSuggestions = false;
          continue;
        }
        if (inSuggestions && line.trim().startsWith("- ")) {
          suggestions.push(line.trim());
        }
      }
      if (suggestions.length === 0) {
        suggestions.push("suggestions=0");
      }
      return textResult(suggestions);
    }

    case "coordination_action": {
      switch (args.action) {
        case "ack_message": {
          if (!identity.session) throw new Error("Bundle session is required to acknowledge a message.");
          if (!args.message_id) throw new Error("message_id is required for ack_message.");
          const response = await memdPost(identity.baseUrl, "/coordination/messages/ack", {
            id: args.message_id,
            session: identity.session,
          });
          return textResult([`action=ack_message`, `acknowledged=${response.messages?.length ?? 0}`]);
        }
        case "assign_scope": {
          if (!identity.session) throw new Error("Bundle session is required for assign_scope.");
          if (!args.scope || !args.target_session) {
            throw new Error("scope and target_session are required for assign_scope.");
          }
          const target = hiveBySession.get(args.target_session);
          if (!target?.base_url) throw new Error(`Unknown target session: ${args.target_session}`);
          await memdPost(identity.baseUrl, "/coordination/claims/transfer", {
            scope: args.scope,
            from_session: identity.session,
            to_session: args.target_session,
            to_agent: target.agent ?? null,
            to_effective_agent: target.effective_agent ?? null,
          });
          await memdPost(target.base_url, "/coordination/messages/send", {
            kind: "assignment",
            from_session: identity.session,
            from_agent: identity.effectiveAgent,
            to_session: args.target_session,
            project: identity.project,
            namespace: identity.namespace,
            workspace: identity.workspace,
            content: args.content || `Take ownership of ${args.scope}`,
            scope: args.scope,
          });
          return textResult([
            `action=assign_scope`,
            `scope=${args.scope}`,
            `target_session=${args.target_session}`,
          ]);
        }
        case "recover_session": {
          if (!args.stale_session) throw new Error("stale_session is required for recover_session.");
          const targetSession = args.target_session || identity.session;
          const stale = hiveBySession.get(args.stale_session);
          const target = hiveBySession.get(targetSession);
          if (!stale) throw new Error(`Unknown stale session: ${args.stale_session}`);
          if (!target?.base_url) throw new Error(`Unknown target session: ${targetSession}`);
          const claims = await memdGet(identity.baseUrl, "/coordination/claims", {
            project: identity.project,
            namespace: identity.namespace,
            workspace: identity.workspace,
            active_only: true,
            limit: 256,
          });
          const tasks = await memdGet(identity.baseUrl, "/coordination/tasks", {
            project: identity.project,
            namespace: identity.namespace,
            workspace: identity.workspace,
            active_only: true,
            limit: 256,
          });
          const reclaimableClaims = (claims.claims ?? []).filter((claim) => claim.session === args.stale_session);
          const stalledTasks = (tasks.tasks ?? []).filter((task) => task.session === args.stale_session);
          for (const claim of reclaimableClaims) {
            await memdPost(identity.baseUrl, "/coordination/claims/recover", {
              scope: claim.scope,
              from_session: args.stale_session,
              to_session: targetSession,
              to_agent: target.agent ?? null,
              to_effective_agent: target.effective_agent ?? null,
            });
          }
          for (const task of stalledTasks) {
            await memdPost(identity.baseUrl, "/coordination/tasks/assign", {
              task_id: task.task_id,
              from_session: args.stale_session,
              to_session: targetSession,
              to_agent: target.agent ?? null,
              to_effective_agent: target.effective_agent ?? null,
              note: `Recovered from ${stale.presence} session ${args.stale_session}`,
            });
          }
          return textResult([
            `action=recover_session`,
            `stale_session=${args.stale_session}`,
            `target_session=${targetSession}`,
            `claims=${reclaimableClaims.length}`,
            `tasks=${stalledTasks.length}`,
          ]);
        }
        case "request_help":
        case "request_review": {
          if (!identity.session) throw new Error("Bundle session is required for request actions.");
          if (!args.target_session || !args.content) {
            throw new Error("target_session and content are required for request actions.");
          }
          const kind = args.action === "request_help" ? "help_request" : "review_request";
          const target = hiveBySession.get(args.target_session);
          if (!target?.base_url) throw new Error(`Unknown target session: ${args.target_session}`);
          const response = await memdPost(target.base_url, "/coordination/messages/send", {
            kind,
            from_session: identity.session,
            from_agent: identity.effectiveAgent,
            to_session: args.target_session,
            project: identity.project,
            namespace: identity.namespace,
            workspace: identity.workspace,
            content: args.content,
            scope: args.scope ?? null,
          });
          return textResult([
            `action=${args.action}`,
            `target_session=${args.target_session}`,
            `sent=${response.messages?.length ?? 0}`,
          ]);
        }
        default:
          throw new Error(`Unsupported coordination action: ${args.action}`);
      }
    }

    case "recover_stale_session": {
      if (!identity.session) throw new Error("Bundle session is required for stale-session recovery.");
      const stale = hiveBySession.get(args.stale_session);
      if (!stale) throw new Error(`Unknown stale session: ${args.stale_session}`);
      if (stale.presence === "active") {
        throw new Error(`Session ${args.stale_session} is still active.`);
      }
      const targetSession = args.target_session ?? identity.session;
      const target = hiveBySession.get(targetSession);
      if (!target) throw new Error(`Unknown target session: ${targetSession}`);

      const claims = await memdGet(identity.baseUrl, "/coordination/claims", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        active_only: true,
        limit: 256,
      });
      const tasks = await memdGet(identity.baseUrl, "/coordination/tasks", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        active_only: true,
        limit: 256,
      });

      const reclaimableClaims = (claims.claims ?? []).filter((claim) => claim.session === args.stale_session);
      const stalledTasks = (tasks.tasks ?? []).filter((task) => task.session === args.stale_session);

      for (const claim of reclaimableClaims) {
        await memdPost(identity.baseUrl, "/coordination/claims/recover", {
          scope: claim.scope,
          from_session: args.stale_session,
          to_session: targetSession,
          to_agent: target.agent ?? null,
          to_effective_agent: target.effective_agent ?? null,
        });
      }

      for (const task of stalledTasks) {
        await memdPost(identity.baseUrl, "/coordination/tasks/assign", {
          task_id: task.task_id,
          from_session: args.stale_session,
          to_session: targetSession,
          to_agent: target.agent ?? null,
          to_effective_agent: target.effective_agent ?? null,
          note: `Recovered from ${stale.presence} session ${args.stale_session}`,
        });
      }

      return textResult([
        `recovered_session=${args.stale_session}`,
        `presence=${stale.presence}`,
        `target_session=${targetSession}`,
        `claims=${reclaimableClaims.length}`,
        `tasks=${stalledTasks.length}`,
      ]);
    }

    case "recommend_boundaries": {
      const tasks = await memdGet(identity.baseUrl, "/coordination/tasks", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        active_only: true,
        limit: 128,
      });
      const lines = recommendBoundaries(tasks.tasks ?? []);
      return textResult([`recommendations=${lines.length}`, ...lines]);
    }

    case "send_message": {
      if (!identity.session) throw new Error("Bundle session is required for hive messaging.");
      const target = hiveBySession.get(args.target_session);
      if (!target?.base_url) throw new Error(`Unknown target session: ${args.target_session}`);
      const response = await memdPost(target.base_url, "/coordination/messages/send", {
        kind: args.kind ?? "handoff",
        from_session: identity.session,
        from_agent: identity.effectiveAgent,
        to_session: args.target_session,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        content: args.content,
      });
      return textResult([`sent=${response.messages?.length ?? 0}`]);
    }

    case "ack_message": {
      if (!identity.session) throw new Error("Bundle session is required to acknowledge messages.");
      const response = await memdPost(identity.baseUrl, "/coordination/messages/ack", {
        id: args.id,
        session: identity.session,
      });
      return textResult([`acknowledged=${response.messages?.length ?? 0}`]);
    }

    case "list_claims": {
      const response = await memdGet(identity.baseUrl, "/coordination/claims", {
        project: identity.project,
        namespace: identity.namespace,
        active_only: args.active_only !== false,
        limit: 128,
      });
      const rows = (response.claims ?? []).map(
        (claim) =>
          `- ${claim.scope} | holder=${claim.effective_agent ?? claim.session} | workspace=${claim.workspace ?? "none"} | expires_at=${claim.expires_at}`
      );
      return textResult([`claims=${rows.length}`, ...rows]);
    }

    case "acquire_claim": {
      if (!identity.session) throw new Error("Bundle session is required to acquire claims.");
      const response = await memdPost(identity.baseUrl, "/coordination/claims/acquire", {
        scope: args.scope,
        session: identity.session,
        agent: identity.agent,
        effective_agent: identity.effectiveAgent,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        host: identity.host,
        pid: identity.pid,
        ttl_seconds: args.ttl_seconds ?? 900,
      });
      return textResult([`acquired=${response.claims?.length ?? 0}`, `scope=${args.scope}`]);
    }

    case "release_claim": {
      if (!identity.session) throw new Error("Bundle session is required to release claims.");
      const response = await memdPost(identity.baseUrl, "/coordination/claims/release", {
        scope: args.scope,
        session: identity.session,
      });
      return textResult([`released=${response.claims?.length ?? 0}`, `scope=${args.scope}`]);
    }

    case "transfer_claim": {
      if (!identity.session) throw new Error("Bundle session is required to transfer claims.");
      const target = hiveBySession.get(args.target_session);
      if (!target) throw new Error(`Unknown target session: ${args.target_session}`);
      const response = await memdPost(identity.baseUrl, "/coordination/claims/transfer", {
        scope: args.scope,
        from_session: identity.session,
        to_session: args.target_session,
        to_agent: target.agent,
        to_effective_agent: target.effective_agent,
      });
      return textResult([`transferred=${response.claims?.length ?? 0}`, `scope=${args.scope}`]);
    }

    case "assign_work": {
      if (!identity.session) throw new Error("Bundle session is required to assign work.");
      const target = hiveBySession.get(args.target_session);
      if (!target?.base_url) throw new Error(`Unknown target session: ${args.target_session}`);
      await memdPost(identity.baseUrl, "/coordination/claims/transfer", {
        scope: args.scope,
        from_session: identity.session,
        to_session: args.target_session,
        to_agent: target.agent,
        to_effective_agent: target.effective_agent,
      });
      const response = await memdPost(target.base_url, "/coordination/messages/send", {
        kind: "assignment",
        from_session: identity.session,
        from_agent: identity.effectiveAgent,
        to_session: args.target_session,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        content:
          args.content ??
          `Assigned scope ${args.scope}. Take ownership and continue from there.`,
      });
      return textResult([`assigned=${response.messages?.length ?? 0}`, `scope=${args.scope}`]);
    }

    case "list_tasks": {
      const response = await memdGet(identity.baseUrl, "/coordination/tasks", {
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        active_only: args.active_only !== false,
        limit: 64,
      });
      const rows = (response.tasks ?? []).map(
        (task) =>
          `- ${task.task_id} [${task.status}] owner=${task.effective_agent ?? task.session ?? "none"} help=${task.help_requested ? "yes" : "no"} review=${task.review_requested ? "yes" : "no"} | ${compact(task.title, 96)}`
      );
      return textResult([`tasks=${rows.length}`, ...rows]);
    }

    case "upsert_task": {
      if (!identity.session) throw new Error("Bundle session is required to own shared tasks.");
      const response = await memdPost(identity.baseUrl, "/coordination/tasks/upsert", {
        task_id: args.task_id,
        title: args.title,
        description: args.description ?? null,
        status: args.status ?? "active",
        coordination_mode: args.coordination_mode ?? "exclusive_write",
        session: identity.session,
        agent: identity.agent,
        effective_agent: identity.effectiveAgent,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        claim_scopes: Array.isArray(args.claim_scopes) ? args.claim_scopes : [],
        help_requested: false,
        review_requested: false,
      });
      const task = response.tasks?.[0];
      return textResult([
        `task=${task?.task_id ?? args.task_id}`,
        `status=${task?.status ?? (args.status ?? "active")}`,
        `mode=${task?.coordination_mode ?? (args.coordination_mode ?? "exclusive_write")}`,
      ]);
    }

    case "assign_task": {
      if (!identity.session) throw new Error("Bundle session is required to assign shared tasks.");
      const target = hiveBySession.get(args.target_session);
      if (!target) throw new Error(`Unknown target session: ${args.target_session}`);
      const response = await memdPost(identity.baseUrl, "/coordination/tasks/assign", {
        task_id: args.task_id,
        from_session: identity.session,
        to_session: args.target_session,
        to_agent: target.agent,
        to_effective_agent: target.effective_agent,
        note: null,
      });
      const task = response.tasks?.[0];
      return textResult([
        `task=${task?.task_id ?? args.task_id}`,
        `assigned_to=${task?.effective_agent ?? task?.session ?? args.target_session}`,
        `status=${task?.status ?? "assigned"}`,
      ]);
    }

    case "request_task_help":
    case "request_task_review": {
      if (!identity.session) throw new Error("Bundle session is required for task requests.");
      const target = hiveBySession.get(args.target_session);
      if (!target?.base_url) throw new Error(`Unknown target session: ${args.target_session}`);
      const needsHelp = request.params.name === "request_task_help";
      const taskResponse = await memdPost(identity.baseUrl, "/coordination/tasks/upsert", {
        task_id: args.task_id,
        title: args.title ?? `Shared task ${args.task_id}`,
        description: args.description ?? null,
        status: needsHelp ? "needs_help" : "needs_review",
        session: identity.session,
        agent: identity.agent,
        effective_agent: identity.effectiveAgent,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        claim_scopes: [],
        help_requested: needsHelp,
        review_requested: !needsHelp,
      });
      const message = await memdPost(target.base_url, "/coordination/messages/send", {
        kind: needsHelp ? "help_request" : "review_request",
        from_session: identity.session,
        from_agent: identity.effectiveAgent,
        to_session: args.target_session,
        project: identity.project,
        namespace: identity.namespace,
        workspace: identity.workspace,
        content: needsHelp
          ? `Need help on shared task ${args.task_id}.`
          : `Need review on shared task ${args.task_id}.`,
      });
      return textResult([
        `task=${taskResponse.tasks?.[0]?.task_id ?? args.task_id}`,
        `requested=${needsHelp ? "help" : "review"}`,
        `message=${message.messages?.[0]?.id ?? "sent"}`,
      ]);
    }

    default:
      throw new Error(`Unknown tool: ${request.params.name}`);
  }
});

const transport = new StdioServerTransport();
await server.connect(transport);
