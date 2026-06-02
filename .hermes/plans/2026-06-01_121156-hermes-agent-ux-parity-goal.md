# Goal: memd 100% Hermes Agent UX parity

## Goal

Make  setup/settings/status/config surfaces feel like Hermes Agent end-to-end, not a rough imitation.

Target: when a user runs Updated existing memd bundle at /home/josue/Documents/projects/memd/.memd, # memd config

- bundle: `/home/josue/Documents/projects/memd/.memd`
- project root: `/home/josue/Documents/projects/memd`
- ready: `true`
- runtime: `present`
- project: `memd`
- namespace: `main`
- agent: `codex`
- base url: `http://100.104.154.24:8787`
- route: `auto`
- intent: `current_task`
- voice mode: `caveman-ultra`
- auto commit: `disabled`
- workspace: `none`
- visibility: `none`
- hive system: `codex`
- hive role: `agent`
- hive group goal: `repair and operate the shared memd control-plane runtime`
- authority: `participant`
- authority mode: `shared`
- authority degraded: `no`
- shared base url: `http://100.104.154.24:8787`
- fallback base url: `none`
- localhost fallback policy: `deny`, {
  "active_agent": "codex",
  "agents": true,
  "authority": "shared",
  "authority_warning": [],
  "bundle": "/home/josue/Documents/projects/memd/.memd",
  "capability_surface": {
    "bridgeable": 0,
    "discovered": 1280,
    "harness_native": 1227,
    "universal": 47
  },
  "config": true,
  "cowork_surface": null,
  "defaults": {
    "agent": "codex",
    "authority": "participant",
    "authority_policy": {
      "localhost_fallback_policy": "deny",
      "shared_primary": true
    },
    "authority_state": {
      "activated_at": "2026-05-25T15:15:00Z",
      "activated_by": "repair",
      "blocked_capabilities": [],
      "degraded": false,
      "expires_at": null,
      "fallback_base_url": null,
      "mode": "shared",
      "reason": "shared authority available",
      "shared_base_url": "http://100.104.154.24:8787",
      "warning_acknowledged_at": null
    },
    "auto_commit": {
      "enabled": false
    },
    "auto_short_term_capture": true,
    "base_url": "http://100.104.154.24:8787",
    "capabilities": [
      "coordination",
      "memory"
    ],
    "heartbeat_model": "gpt-5.4-mini",
    "hive_group_goal": "repair and operate the shared memd control-plane runtime",
    "hive_groups": [
      "project:memd",
      "control-plane"
    ],
    "hive_project_anchor": "/run/media/josue/T7/projects/memd",
    "hive_project_enabled": false,
    "hive_project_joined_at": "2026-05-25T15:15:00Z",
    "hive_role": "agent",
    "hive_system": "codex",
    "intent": "current_task",
    "namespace": "main",
    "project": "memd",
    "route": "auto",
    "session": "session-008f3488",
    "tab_id": null,
    "visibility": null,
    "voice_mode": "caveman-ultra",
    "workspace": null
  },
  "degraded": false,
  "env": true,
  "env_ps1": true,
  "evolution": null,
  "exists": true,
  "fallback_base_url": null,
  "harness_bridge": {
    "generated_at": "2026-05-30T21:11:11.992507988Z",
    "harnesses": [
      {
        "harness": "codex",
        "missing_surfaces": [],
        "notes": [
          "Codex is native when the config, hook, and skill surfaces are all present."
        ],
        "portability_class": "harness-native",
        "required_surfaces": [
          "config",
          "hook",
          "skill"
        ],
        "wired": true
      },
      {
        "harness": "claude",
        "missing_surfaces": [],
        "notes": [
          "Claude is native when the settings and session hook surfaces exist."
        ],
        "portability_class": "harness-native",
        "required_surfaces": [
          "settings",
          "hook"
        ],
        "wired": true
      },
      {
        "harness": "claw",
        "missing_surfaces": [],
        "notes": [
          "Claw is memd-ready when the binary is installed, config exists, and memd skills are visible through shared skill roots."
        ],
        "portability_class": "harness-native",
        "required_surfaces": [
          "binary",
          "config",
          "skill"
        ],
        "wired": true
      },
      {
        "harness": "openclaw",
        "missing_surfaces": [],
        "notes": [
          "OpenClaw is native when AGENTS.md and BOOTSTRAP.md bridge surfaces exist."
        ],
        "portability_class": "harness-native",
        "required_surfaces": [
          "agents",
          "bootstrap"
        ],
        "wired": true
      },
      {
        "harness": "opencode",
        "missing_surfaces": [],
        "notes": [
          "OpenCode is native when config, plugin, and command surfaces all route through memd."
        ],
        "portability_class": "harness-native",
        "required_surfaces": [
          "config",
          "plugin",
          "command"
        ],
        "wired": true
      }
    ],
    "missing_harnesses": [],
    "portability_class": "portable",
    "portable": true,
    "ready": true
  },
  "heartbeat": {
    "agent": "codex",
    "authority": "participant",
    "authority_degraded": false,
    "authority_mode": "shared",
    "base_branch": null,
    "base_url": "http://100.104.154.24:8787",
    "base_url_healthy": null,
    "blocked_by": [],
    "branch": "main",
    "capabilities": [
      "coordination",
      "memory"
    ],
    "confidence": null,
    "cowork_with": [],
    "display_name": null,
    "effective_agent": "codex@session-008f3488",
    "focus": "id=e8a3b626-3033-4a48-bafa-b484f7ca73cf | stage=canonical | scope=project | kind=decision | status=active | project=memd | ns=main | vis=private | agent=codex@session-008f3488 | lane=architecture | tags=jarvis,plannin...",
    "handoff_state": null,
    "handoff_target": null,
    "heartbeat_model": "gpt-5.4-mini",
    "hive_group_goal": "repair and operate the shared memd control-plane runtime",
    "hive_groups": [
      "control-plane",
      "project:memd"
    ],
    "hive_role": "agent",
    "hive_system": "codex",
    "host": null,
    "lane_id": "/home/josue/Documents/projects/memd",
    "last_seen": "2026-06-01T16:11:56.744150176Z",
    "namespace": "main",
    "needs_help": false,
    "needs_review": false,
    "next_action": "lane=architecture",
    "next_recovery": "status: id=d29994da-7262-4c53-976e-876d06a39d0c | stage=canonical | scope=project | kind=status | status=active | project=memd | ns=main | vis=private | agent=codex@session-008f3488 | lane=decisions | tags=checkpoint,current-...",
    "offered_to": [],
    "pid": 100058,
    "pressure": "2026-05-09 runtime/RAG/Mac Bridge/authority-search code is complete and committed: 709e14d, 10f18ee, 0ac72d7. V20 evidence ops remains active; 1.0.0 still blocked on real users/devices/harness pairs/external auditor/third-party replay evidence.",
    "project": "memd",
    "repo_root": "/home/josue/Documents/projects/memd",
    "risk": null,
    "role": "agent",
    "scope_claims": [
      "project"
    ],
    "session": "session-008f3488",
    "status": "live",
    "tab_id": null,
    "task_id": null,
    "topic_claim": "lane=architecture",
    "touches": [
      "project"
    ],
    "visibility": null,
    "worker_name": "Memd Codex 008f3488",
    "working": "lane=architecture",
    "workspace": null,
    "worktree_root": "/home/josue/Documents/projects/memd"
  },
  "hooks": true,
  "lane_fault": null,
  "lane_receipts": null,
  "lane_surface": null,
  "localhost_read_only_allowed": false,
  "maintenance_surface": null,
  "memory_quality_degraded": false,
  "missing": [],
  "rag": {
    "configured": false,
    "enabled": false,
    "healthy": null,
    "source": "backend.rag",
    "url": null
  },
  "resume_preview": null,
  "runtimes": {
    "all_wired": true,
    "claude": {
      "command": true,
      "hook": true,
      "settings": true,
      "skill": true,
      "wired": true
    },
    "claude_family": {
      "count": 1,
      "harnesses": [
        {
          "harness": "claw",
          "hook": false,
          "root": "/home/josue/.claw",
          "settings": true,
          "wired": false
        }
      ]
    },
    "claw": {
      "binary": true,
      "config": true,
      "skill": true,
      "wired": true
    },
    "codex": {
      "config": true,
      "hook": true,
      "skill": true,
      "wired": true
    },
    "openclaw": {
      "agents": true,
      "bootstrap": true,
      "wired": true
    },
    "opencode": {
      "command": true,
      "config": true,
      "plugin": true,
      "wired": true
    }
  },
  "server": {
    "atlas": {
      "dormant": false,
      "edge_item_ratio": 12.3359375,
      "edges_active": 1579,
      "edges_dormant": 9,
      "edges_total": 1588,
      "region_count": 16
    },
    "items": 2478,
    "pressure": {
      "candidates": 649,
      "expired": 2089,
      "inbox": 307,
      "stale": 239
    },
    "rag": {
      "enabled": false,
      "last_sync_status": "disabled",
      "reachable": false,
      "timeout_ms": 300
    },
    "status": "ok"
  },
  "session_overlay": {
    "bundle_session": "session-t7-memd",
    "live_session": "session-008f3488",
    "rebased_from": "session-t7-memd"
  },
  "setup_ready": true,
  "shared_base_url": "http://100.104.154.24:8787",
  "shared_primary": true,
  "truth_summary": null,
  "worker_name_env_ready": true
}, or related first-run commands, the terminal UX should match Hermes Agents command grammar, visual rhythm, language, and beginner flow closely enough that the user recognizes it as the same product family.