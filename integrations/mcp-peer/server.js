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

    default:
      throw new Error(`Unknown tool: ${request.params.name}`);
  }
});

const transport = new StdioServerTransport();
await server.connect(transport);
