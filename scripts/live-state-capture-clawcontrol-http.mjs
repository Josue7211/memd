#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const apiBase = (process.env.CLAWCONTROL_API_BASE || 'http://127.0.0.1:3000').replace(/\/+$/, '');
const memdBin = process.env.MEMD_BIN || 'memd';
const memdOutput = process.env.MEMD_OUTPUT || new URL('../.memd', import.meta.url).pathname;
const timeoutMs = Number(process.env.TIMEOUT_MS || '1500');
const freshnessSecs = Math.max(60, Number(process.env.FRESHNESS_SECS || '86400'));
const dryRun = process.env.DRY_RUN === '1' || process.env.DRY_RUN === 'true';
const probeOnly = process.env.PROBE_ONLY === '1' || process.env.PROBE_ONLY === 'true';

const endpoints = [
  { module: 'calendar', scope: 'primary', path: '/api/calendar' },
  { module: 'todos', scope: 'default', path: '/api/todos' },
  { module: 'reminders', scope: 'default', path: '/api/reminders' },
  { module: 'messages', scope: 'approved', path: '/api/messages?limit=10', sensitive: true },
  { module: 'email', scope: 'approved', path: '/api/email?folder=INBOX&limit=10', sensitive: true },
];

function asRecord(value) {
  return value && typeof value === 'object' && !Array.isArray(value) ? value : {};
}

function asArray(value) {
  return Array.isArray(value) ? value.filter((item) => item && typeof item === 'object') : [];
}

function textValue(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function boolValue(value) {
  return value === true;
}

function compactLine(value, max = 140) {
  const single = String(value || '').replace(/\s+/g, ' ').trim();
  return single.length > max ? `${single.slice(0, max - 3).trim()}...` : single;
}

function summarizeDate(value) {
  const raw = textValue(value);
  if (!raw) return '';
  const date = new Date(raw);
  return Number.isNaN(date.getTime()) ? raw : date.toISOString();
}

function summarizeCalendar(data) {
  const record = asRecord(data);
  const events = asArray(record.events)
    .map((event) => ({
      title: compactLine(textValue(event.title) || 'Untitled event', 80),
      start: summarizeDate(event.start),
      end: summarizeDate(event.end),
      calendar: compactLine(textValue(event.calendar), 48),
      allDay: boolValue(event.allDay),
    }))
    .sort((left, right) => left.start.localeCompare(right.start))
    .slice(0, 8);
  if (events.length === 0) return 'calendar: loaded; upcoming_events=0';
  return [
    `calendar: loaded; source=${textValue(record.source) || 'unknown'}; upcoming_events=${events.length}`,
    ...events.map(
      (event) =>
        `- ${event.title} | ${event.start || 'unknown start'}${event.end ? ` to ${event.end}` : ''}${event.calendar ? ` | ${event.calendar}` : ''}${event.allDay ? ' | all-day' : ''}`,
    ),
  ].join('\n');
}

function summarizeTodos(data) {
  const todos = asArray(asRecord(data).todos);
  const open = todos.filter((todo) => !boolValue(todo.done) && !boolValue(todo.completed)).slice(0, 8);
  if (open.length === 0) return `todos: loaded; open=0 total=${todos.length}`;
  return [
    `todos: loaded; open=${open.length} total=${todos.length}`,
    ...open.map((todo) => {
      const title = compactLine(textValue(todo.text) || textValue(todo.title) || 'Untitled todo', 90);
      const due = textValue(todo.due_date) || textValue(todo.dueDate);
      return `- ${title}${due ? ` | due ${due}` : ''}`;
    }),
  ].join('\n');
}

function summarizeReminders(data) {
  const reminders = asArray(asRecord(data).reminders);
  const open = reminders.filter((reminder) => !boolValue(reminder.completed)).slice(0, 8);
  if (open.length === 0) return `reminders: loaded; open=0 total=${reminders.length}`;
  return [
    `reminders: loaded; open=${open.length} total=${reminders.length}`,
    ...open.map((reminder) => {
      const title = compactLine(textValue(reminder.title) || textValue(reminder.text) || 'Untitled reminder', 90);
      const due = textValue(reminder.due_date) || textValue(reminder.dueDate);
      return `- ${title}${due ? ` | due ${due}` : ''}`;
    }),
  ].join('\n');
}

function summarizeMessages(data) {
  const conversations = asArray(asRecord(data).conversations).slice(0, 8);
  if (conversations.length === 0) return 'messages: loaded; conversations=0';
  return [
    `messages: loaded; conversations=${conversations.length}`,
    ...conversations.map((conversation) => {
      const name = compactLine(
        textValue(conversation.displayName) ||
          textValue(conversation.name) ||
          textValue(conversation.chatIdentifier) ||
          'Unknown conversation',
        70,
      );
      const unread = Number(conversation.unreadCount ?? conversation.unread_count ?? 0);
      return `- ${name}${unread > 0 ? ` | unread ${unread}` : ''}`;
    }),
  ].join('\n');
}

function summarizeEmail(data) {
  const emails = asArray(asRecord(data).emails).slice(0, 8);
  if (emails.length === 0) return 'email: loaded; inbox_items=0';
  return [
    `email: loaded; inbox_items=${emails.length}`,
    ...emails.map((email) => {
      const from = compactLine(textValue(email.from) || textValue(email.sender) || 'unknown sender', 56);
      const subject = compactLine(textValue(email.subject) || 'No subject', 90);
      const unread = boolValue(email.read) ? '' : ' | unread';
      return `- ${from}: ${subject}${unread}`;
    }),
  ].join('\n');
}

function summarize(module, data) {
  switch (module) {
    case 'calendar':
      return summarizeCalendar(data);
    case 'todos':
      return summarizeTodos(data);
    case 'reminders':
      return summarizeReminders(data);
    case 'messages':
      return summarizeMessages(data);
    case 'email':
      return summarizeEmail(data);
    default:
      return `${module}: loaded`;
  }
}

function recordFor({ module, scope, sensitive }, data, summary) {
  return {
    sourceApp: 'clawcontrol',
    module,
    scope,
    visibility: 'private',
    privacy: sensitive ? 'metadata' : 'approved',
    approved: !sensitive,
    agentsecretsApproved: false,
    freshnessSecs,
    labels: ['live-app-state', module, ...(sensitive ? ['metadata'] : [])],
    summary,
    payload: sensitive ? { summary } : data,
  };
}

async function fetchJson(path) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(`${apiBase}${path}`, { signal: controller.signal });
    if (!response.ok) return { ok: false, status: response.status, error: `HTTP ${response.status}` };
    return { ok: true, status: response.status, data: await response.json() };
  } catch (error) {
    return {
      ok: false,
      status: 0,
      error: error && error.name === 'AbortError' ? 'timeout' : 'unreachable',
    };
  } finally {
    clearTimeout(timer);
  }
}

function visiblePageRecord() {
  const raw = process.env.VISIBLE_PAGE_JSON || readVisiblePageFile();
  if (!raw) return undefined;
  const payload = JSON.parse(raw);
  const summary =
    process.env.VISIBLE_PAGE_SUMMARY ||
    `visible page/module, route=${compactLine(payload.route || 'unknown', 80)} title=${compactLine(payload.title || 'unknown', 80)}`;
  return {
    sourceApp: 'clawcontrol',
    module: 'visible_page',
    scope: 'current',
    visibility: 'private',
    privacy: 'metadata',
    approved: false,
    agentsecretsApproved: false,
    freshnessSecs,
    labels: ['live-app-state', 'visible_page'],
    summary,
    payload,
  };
}

function readVisiblePageFile() {
  if (!process.env.VISIBLE_PAGE_FILE) return '';
  return readFileSync(process.env.VISIBLE_PAGE_FILE, 'utf8');
}

const records = [];
const visible = visiblePageRecord();
if (visible) records.push(visible);

const probe = {
  apiBase,
  timeoutMs,
  visiblePage: visible ? 'present' : 'missing',
  endpoints: [],
};

for (const endpoint of endpoints) {
  const result = await fetchJson(endpoint.path);
  probe.endpoints.push({
    module: endpoint.module,
    path: endpoint.path,
    ok: result.ok,
    status: result.status,
    error: result.ok ? undefined : result.error,
  });
  if (!result.ok) continue;
  records.push(recordFor(endpoint, result.data, summarize(endpoint.module, result.data)));
}

if (probeOnly) {
  const produced = records.map((record) => record.module);
  const required = ['visible_page', ...endpoints.map((endpoint) => endpoint.module)];
  const missing = required.filter((module) => !produced.includes(module));
  console.log(
    JSON.stringify(
      {
        ...probe,
        produced,
        missing,
        recordCount: records.length,
      },
      null,
      2,
    ),
  );
  process.exit(missing.length === 0 ? 0 : 2);
}

if (records.length === 0) {
  console.error(`live-state-capture-clawcontrol-http: no reachable live endpoints at ${apiBase}`);
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
  { input: batch, encoding: 'utf8', stdio: ['pipe', 'inherit', 'inherit'] },
);
process.exit(result.status ?? 1);
