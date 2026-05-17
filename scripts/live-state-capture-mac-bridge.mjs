#!/usr/bin/env node

import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = new URL('..', import.meta.url).pathname.replace(/\/+$/, '');
const envFile = process.env.MAC_BRIDGE_ENV || join(root, 'integrations', 'mac-bridge', '.env');
const bridgeEnv = readEnvFile(envFile);
const bridgeBase = (process.env.MAC_BRIDGE_BASE || `http://127.0.0.1:${bridgeEnv.BRIDGE_PORT || '4100'}`).replace(/\/+$/, '');
const apiKey = (process.env.MAC_BRIDGE_API_KEY || bridgeEnv.BRIDGE_API_KEY || '').trim();
const memdBin = process.env.MEMD_BIN || 'memd';
const memdOutput = process.env.MEMD_OUTPUT || join(root, '.memd');
const timeoutMs = Number(process.env.TIMEOUT_MS || '2500');
const freshnessSecs = Math.max(60, Number(process.env.FRESHNESS_SECS || '3600'));
const fixtureDir = process.env.MAC_BRIDGE_FIXTURE_DIR || '';
const dryRun = process.env.DRY_RUN === '1' || process.env.DRY_RUN === 'true';

function readEnvFile(path) {
  try {
    return Object.fromEntries(
      readFileSync(path, 'utf8')
        .split(/\n/)
        .map((line) => line.trim())
        .filter((line) => line && !line.startsWith('#') && line.includes('='))
        .map((line) => {
          const index = line.indexOf('=');
          const key = line.slice(0, index).trim();
          const value = line.slice(index + 1).trim().replace(/^['"]|['"]$/g, '');
          return [key, value];
        }),
    );
  } catch {
    return {};
  }
}

function compact(value, max = 140) {
  const single = String(value || '').replace(/\s+/g, ' ').trim();
  return single.length > max ? `${single.slice(0, max - 3).trim()}...` : single;
}

function asRecord(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value.filter((item) => item && typeof item === 'object') : [];
}

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function isoDate(value) {
  const raw = text(value);
  if (!raw) return '';
  const date = new Date(raw);
  return Number.isNaN(date.getTime()) ? raw : date.toISOString();
}

async function fetchBridge(path) {
  if (fixtureDir) {
    const name = path.startsWith('/calendar') ? 'calendar.json' : 'reminders.json';
    return JSON.parse(readFileSync(join(fixtureDir, name), 'utf8'));
  }
  if (!apiKey) {
    throw new Error('MAC_BRIDGE_API_KEY/BRIDGE_API_KEY unavailable');
  }
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(`${bridgeBase}${path}`, {
      signal: controller.signal,
      headers: { 'X-API-Key': apiKey },
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return response.json();
  } finally {
    clearTimeout(timer);
  }
}

function calendarEvents(data) {
  return asArray(asRecord(data).events)
    .map((event) => ({
      title: compact(text(event.title) || 'Untitled event', 80),
      start: isoDate(event.start),
      end: isoDate(event.end),
      calendar: compact(text(event.calendar), 48),
      allDay: event.allDay === true,
    }))
    .filter((event) => event.title || event.start)
    .sort((left, right) => left.start.localeCompare(right.start))
    .slice(0, 12);
}

function reminderItems(data) {
  const raw = asArray(data) || [];
  const source = raw.length > 0 ? raw : asArray(asRecord(data).reminders);
  return source
    .map((reminder) => ({
      title: compact(text(reminder.title) || text(reminder.text) || 'Untitled reminder', 90),
      dueDate: text(reminder.dueDate) || text(reminder.due_date) || text(reminder.due) || '',
      list: compact(text(reminder.list) || text(reminder.listName), 48),
      completed: reminder.completed === true || reminder.done === true,
      priority: text(reminder.priority),
    }))
    .filter((reminder) => reminder.title && !reminder.completed)
    .slice(0, 12);
}

function record({ module, scope, summary, payload }) {
  return {
    sourceApp: 'clawcontrol',
    module,
    scope,
    visibility: 'private',
    privacy: 'metadata',
    approved: true,
    agentsecretsApproved: false,
    freshnessSecs,
    labels: ['live-app-state', module, 'metadata', 'producer:mac-bridge'],
    summary,
    payload: {
      producer: 'mac-bridge',
      sourceAppAlias: 'clawcontrol',
      ...payload,
    },
  };
}

const records = [];
const errors = [];

try {
  const calendar = calendarEvents(await fetchBridge('/calendar?pastDays=0&futureDays=30'));
  records.push(
    record({
      module: 'calendar',
      scope: 'primary',
      summary: calendar.length === 0
        ? 'calendar: mac-bridge loaded; upcoming_events=0'
        : [
            `calendar: mac-bridge loaded; upcoming_events=${calendar.length}`,
            ...calendar.slice(0, 8).map((event) =>
              `- ${event.title} | ${event.start || 'unknown start'}${event.end ? ` to ${event.end}` : ''}${event.calendar ? ` | ${event.calendar}` : ''}${event.allDay ? ' | all-day' : ''}`),
          ].join('\n'),
      payload: { events: calendar, range: 'current-and-next' },
    }),
  );
} catch (error) {
  errors.push(`calendar:${error.message}`);
}

try {
  const reminders = reminderItems(await fetchBridge('/reminders?filter=open'));
  records.push(
    record({
      module: 'reminders',
      scope: 'default',
      summary: reminders.length === 0
        ? 'reminders: mac-bridge loaded; open=0'
        : [
            `reminders: mac-bridge loaded; open=${reminders.length}`,
            ...reminders.slice(0, 8).map((reminder) =>
              `- ${reminder.title}${reminder.dueDate ? ` | due ${reminder.dueDate}` : ''}${reminder.list ? ` | ${reminder.list}` : ''}`),
          ].join('\n'),
      payload: { reminders },
    }),
  );
  records.push(
    record({
      module: 'todos',
      scope: 'default',
      summary: reminders.length === 0
        ? 'todos: mac-bridge reminders mirror loaded; open=0'
        : [
            `todos: mac-bridge reminders mirror loaded; open=${reminders.length}`,
            ...reminders.slice(0, 8).map((todo) =>
              `- ${todo.title}${todo.dueDate ? ` | due ${todo.dueDate}` : ''}${todo.priority ? ` | priority ${todo.priority}` : ''}`),
          ].join('\n'),
      payload: {
        todos: reminders.map((todo) => ({
          text: todo.title,
          dueDate: todo.dueDate,
          completed: todo.completed,
          priority: todo.priority,
        })),
        mirroredFrom: 'reminders',
      },
    }),
  );
} catch (error) {
  errors.push(`reminders:${error.message}`);
}

if (records.length === 0) {
  console.error(`live-state-capture-mac-bridge: no usable records${errors.length ? ` errors=${errors.join(',')}` : ''}`);
  process.exit(2);
}

const batch = JSON.stringify({ records }, null, 2);
if (dryRun) {
  console.log(batch);
  process.exit(0);
}

const result = spawnSync(
  memdBin,
  ['live-state', 'ingest-batch', '--output', memdOutput, '--stdin'],
  { input: batch, encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'] },
);
if (result.status !== 0) {
  const stderr = String(result.stderr || '').replace(/\s+/g, ' ').trim();
  if (stderr) {
    console.error(`live-state-capture-mac-bridge: ingest failed: ${compact(stderr, 240)}`);
  }
  process.exit(result.status ?? 1);
}
console.error(
  `live-state-capture-mac-bridge: captured records=${records.length} modules=${records
    .map((record) => record.module)
    .join(',')} privacy=metadata visibility=private`,
);
if (errors.length) {
  console.error(`live-state-capture-mac-bridge: partial capture errors=${errors.join(',')}`);
}
