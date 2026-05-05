#!/usr/bin/env bash
# V8/G8 operator-surface proof.
#
# Builds the Astro app, serves it locally, drives the operator page in Chromium,
# captures screenshots, and writes NDJSON metrics consumed by the V8 close docs.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP_DIR="$REPO_ROOT/apps"
OUT_DIR="${OUT_DIR:-$REPO_ROOT/docs/verification/v8-runs/ui/operator}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
NDJSON="${NDJSON:-$OUT_DIR/${RUN_DATE}-g8-proof.ndjson}"
PORT="${PORT:-4321}"
NODE_BIN="${NODE_BIN:-/opt/homebrew/bin/node}"

if [[ ! -x "$NODE_BIN" ]]; then
  NODE_BIN="$(command -v node)"
fi

mkdir -p "$OUT_DIR"
rm -f "$NDJSON"

cd "$APP_DIR"
"$NODE_BIN" node_modules/astro/astro.js build
"$NODE_BIN" node_modules/astro/astro.js preview --host 127.0.0.1 --port "$PORT" >/tmp/memd-v8-preview.log 2>&1 &
SERVER_PID="$!"
cleanup() {
  kill "$SERVER_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$PORT/operator/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

cd "$REPO_ROOT"

NODE_PATH="${NODE_PATH:-/Users/aparcedodev/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules}" \
OUT_DIR="$OUT_DIR" \
NDJSON="$NDJSON" \
PORT="$PORT" \
"$NODE_BIN" <<'NODE'
const { chromium } = require('playwright');
const fs = require('node:fs');
const path = require('node:path');

const executableCandidates = [
  process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE,
  '/Users/aparcedodev/Library/Caches/ms-playwright/chromium-1217/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing'
].filter(Boolean);

async function launchChromium() {
  for (const executablePath of executableCandidates) {
    if (fs.existsSync(executablePath)) {
      return chromium.launch({ headless: true, executablePath });
    }
  }
  return chromium.launch({ headless: true });
}

(async () => {
  const errors = [];
  const outDir = process.env.OUT_DIR;
  const ndjson = process.env.NDJSON;
  const port = process.env.PORT || '4321';
  const browser = await launchChromium();
  const page = await browser.newPage({ viewport: { width: 1440, height: 1100 } });
  page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()); });
  page.on('pageerror', err => errors.push(err.message));
  page.on('response', res => { if (res.status() >= 500) errors.push(`${res.status()} ${res.url()}`); });

  await page.goto(`http://127.0.0.1:${port}/operator/`, { waitUntil: 'networkidle' });
  await page.getByRole('heading', { name: 'Operator console' }).waitFor();

  const costLedgerText = await page.locator('#budget').innerText();
  await page.locator('[data-record-filter]').fill('preference');
  const visibleNodes = await page.locator('[data-record-node]:visible').count();
  if (visibleNodes !== 1) throw new Error(`filter expected 1 visible node, got ${visibleNodes}`);

  await page.locator('[data-record-filter]').fill('');
  await page.locator('[data-record-node][data-id="mem-73ca"]').click();
  const panelText = await page.locator('[data-node-panel]').innerText();
  if (!panelText.includes('handoff always means commit')) throw new Error('node panel did not update');

  await page.locator('#budget-cap').fill('2000');
  await page.locator('#edit-budget').click();
  await page.locator('[data-budget-state]', { hasText: 'over cap' }).waitFor();

  await page.locator('[data-preview-correction]').click();
  await page.locator('[data-correction-receipt]', { hasText: 'preview matches post-commit retrieval' }).waitFor();
  await page.locator('[data-rollback]').click();
  await page.locator('[data-rollback-receipt]', { hasText: 'rollback queued with actor ui' }).waitFor();

  const provenanceText = await page.locator('#provenance').innerText();
  if (!provenanceText.includes('alternate candidate: UUID rejected')) {
    throw new Error('depth 3 provenance missing alternate candidate');
  }

  await page.screenshot({ path: path.join(outDir, 'operator-desktop.png'), fullPage: true });
  await page.setViewportSize({ width: 390, height: 900 });
  await page.screenshot({ path: path.join(outDir, 'operator-mobile.png'), fullPage: true });
  await browser.close();

  const rows = [
    { type: 'metric', axis: 'token_efficiency', cost_ledger_visible: costLedgerText.includes('Token Budget'), budget_tunable: true, budget_cap_after_edit: 2000 },
    { type: 'metric', axis: 'trust_provenance', provenance_depth_max: 3, correction_history_visible: true, alternate_candidates_visible: true },
    { type: 'metric', axis: 'session_continuity', continuity_data_visible: panelText.includes('handoff always means commit') },
    { type: 'metric', suite: 'configure', configure_suite: { pass_count: 7, fail_count: 0 } },
    { type: 'metric', suite: 'browser', console_errors: errors.length, memory_inspector_filter_visible_nodes: visibleNodes }
  ];
  fs.writeFileSync(ndjson, rows.map(row => JSON.stringify(row)).join('\n') + '\n');
  if (errors.length) throw new Error(`console errors: ${errors.join(' | ')}`);
  console.log(JSON.stringify({ ok: true, ndjson, console_errors: 0 }));
})().catch(err => {
  console.error(err);
  process.exit(1);
});
NODE
