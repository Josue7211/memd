#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPROVED_REQUEST="${MEMD_APPROVED_COMMUNICATIONS_REQUEST:-$ROOT/.memd/state/approved-communications-request.json}"
SUPERMEMORY_REQUEST="${MEMD_SUPERMEMORY_REPLAY_REQUEST:-$ROOT/.memd/state/supermemory-replay-request.json}"

node - "$APPROVED_REQUEST" "$SUPERMEMORY_REQUEST" <<'NODE'
const fs = require('fs');

const approvedPath = process.argv[2];
const supermemoryPath = process.argv[3];
let blocked = false;

function readJson(path) {
  try {
    return JSON.parse(fs.readFileSync(path, 'utf8'));
  } catch {
    return null;
  }
}

function print(key, value) {
  if (value === undefined || value === null || value === '') return;
  const text = Array.isArray(value) ? value.join(',') : String(value);
  process.stdout.write(`${key}=${text.replace(/\s+/g, ' ').trim()}\n`);
}

const approved = readJson(approvedPath);
if (approved && approved.status && approved.status !== 'pass' && approved.status !== 'ok') {
  blocked = true;
  const approval = approved.approval || {};
  print('UNBLOCK_APPROVED_COMMUNICATIONS_STATUS', approved.status);
  print('UNBLOCK_APPROVED_COMMUNICATIONS_MISSING', approved.missing || []);
  print('UNBLOCK_APPROVED_COMMUNICATIONS_REQUEST', approval.requestPath || approvedPath);
  print('UNBLOCK_APPROVED_COMMUNICATIONS_TEMPLATE', approval.approvedFileTemplate);
  print(
    'UNBLOCK_APPROVED_COMMUNICATIONS_ACTION',
    `Provide approved metadata via APPROVED_COMMUNICATIONS_FILE=${approval.approvedFileTemplate || '<approved-json>'} scripts/live-state-capture-approved-communications.mjs, or explicitly approve zero metadata with APPROVED_COMMUNICATIONS_EMPTY_APPROVED=1 scripts/live-state-capture-approved-communications.mjs`,
  );
  print(
    'UNBLOCK_APPROVED_COMMUNICATIONS_PRIVACY',
    'metadata/redacted snippets only; every item approved=true; no raw bodies, transcripts, HTML, blobs, or media',
  );
}

const supermemory = readJson(supermemoryPath);
if (supermemory && supermemory.status && supermemory.status !== 'pass' && supermemory.status !== 'ok') {
  blocked = true;
  const routes = supermemory.approved_routes || {};
  const fixture = supermemory.same_fixture_contract || {};
  const bitwarden = supermemory.bitwarden || {};
  print('UNBLOCK_SUPERMEMORY_STATUS', supermemory.status);
  print('UNBLOCK_SUPERMEMORY_MISSING', supermemory.missing_requirements || []);
  print('UNBLOCK_SUPERMEMORY_REQUEST', supermemoryPath);
  print('UNBLOCK_SUPERMEMORY_MEMD_REPORT', fixture.memd_report);
  print('UNBLOCK_SUPERMEMORY_REPLAY_PATH', fixture.replay_path);
  print('UNBLOCK_SUPERMEMORY_REQUIRED_LIMIT', fixture.required_limit);
  print('UNBLOCK_SUPERMEMORY_BITWARDEN_STATUS', bitwarden.status);
  print(
    'UNBLOCK_SUPERMEMORY_ACTION',
    `Unlock/use approved credential route, then run ${routes.run_replay || 'TRY_REPLAY=1 scripts/verify/25-5-supermemory-head-to-head.sh'}, or provide artifact with ${routes.provide_artifact || 'SUPERMEMORY_REPLAYS=/path/to/supermemory-replays scripts/verify/25-5-supermemory-head-to-head.sh'}`,
  );
  print('UNBLOCK_SUPERMEMORY_PRIVACY', 'credential must stay process-local; do not store SUPERMEMORY_API_KEY in memd artifacts');
}

print('UNBLOCK_STATUS', blocked ? 'blocked' : 'clear');
NODE
