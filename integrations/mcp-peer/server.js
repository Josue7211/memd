#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
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

function listPeerBundles(bundleRoot) {
  const projectRoot = path.dirname(bundleRoot);
  const projectsRoot = path.dirname(projectRoot);
  const peers = [];

  for (const entry of fs.readdirSync(projectsRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const candidateProject = path.join(projectsRoot, entry.name);
    const candidateBundle = path.join(candidateProject, ".memd");
    if (!fs.existsSync(path.join(candidateBundle, "config.json"))) continue;

    const runtime = readRuntime(candidateBundle);
    const heartbeat = readHeartbeat(candidateBundle);
    const resume = readResume(candidateBundle);

    peers.push({
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

  return peers;
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
    baseUrl: runtime.base_url ?? "http://127.0.0.1:8787",
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

const bundleRoot = resolveBundleRoot();
const identity = currentIdentity(bundleRoot);

const server = new Server(
  { name: "memd-peer-mcp", version: "0.1.0" },
  {
    capabilities: { tools: {} },
    instructions:
      "Use memd peer coordination tools for coworking. Prefer explicit claims and assignments over implicit overlap. Treat memd as the coordination source of truth.",
  }
);

const tools = [
  {
    name: "list_peers",
    description: "List other live or recent memd peer sessions discovered from sibling bundles.",
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
    description: "Read pending peer messages for the current session.",
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
    description: "Acknowledge a peer message addressed to the current session.",
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
    description: "List active peer claims for the current project/namespace.",
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
];

server.setRequestHandler(ListToolsRequestSchema, async () => ({ tools }));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const args = request.params.arguments ?? {};
  const peers = listPeerBundles(bundleRoot);
  const peerBySession = new Map(peers.map((peer) => [peer.session, peer]));

  switch (request.params.name) {
    case "list_peers": {
      const includeCurrent = Boolean(args.include_current);
      const activeOnly = args.active_only !== false;
      const rows = peers
        .filter((peer) => includeCurrent || peer.bundle_root !== bundleRoot)
        .filter((peer) => !activeOnly || peer.presence === "active")
        .map(
          (peer) =>
            `- ${peer.effective_agent ?? peer.session ?? "unknown"} | presence=${peer.presence} | workspace=${peer.workspace ?? "none"} | focus="${compact(peer.focus ?? "none", 80)}"`
        );
      return textResult([
        `bundle=${bundleRoot}`,
        `current=${identity.effectiveAgent ?? "unknown"}`,
        `peers=${rows.length}`,
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

    case "recover_stale_session": {
      if (!identity.session) throw new Error("Bundle session is required for stale-session recovery.");
      const stale = peerBySession.get(args.stale_session);
      if (!stale) throw new Error(`Unknown stale session: ${args.stale_session}`);
      if (stale.presence === "active") {
        throw new Error(`Session ${args.stale_session} is still active.`);
      }
      const targetSession = args.target_session ?? identity.session;
      const target = peerBySession.get(targetSession);
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

    case "send_message": {
      if (!identity.session) throw new Error("Bundle session is required for peer messaging.");
      const target = peerBySession.get(args.target_session);
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
      const target = peerBySession.get(args.target_session);
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
      const target = peerBySession.get(args.target_session);
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
      const target = peerBySession.get(args.target_session);
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
      const target = peerBySession.get(args.target_session);
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
