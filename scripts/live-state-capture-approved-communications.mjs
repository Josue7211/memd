#!/usr/bin/env node

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = new URL('..', import.meta.url).pathname.replace(/\/+$/, '');
const memdBin = process.env.MEMD_BIN || 'memd';
const memdOutput = process.env.MEMD_OUTPUT || join(root, '.memd');
const sourceStatusOutput = process.env.SOURCE_STATUS_OUTPUT || memdOutput;
const freshnessSecs = Math.max(60, Number(process.env.FRESHNESS_SECS || '3600'));
const dryRun = process.env.DRY_RUN === '1' || process.env.DRY_RUN === 'true';
const approvedEmpty =
  process.env.APPROVED_COMMUNICATIONS_EMPTY_APPROVED === '1' ||
  process.env.APPROVED_COMMUNICATIONS_EMPTY_APPROVED === 'true';
const files = [
  { path: process.env.APPROVED_COMMUNICATIONS_FILE, module: '' },
  { path: process.env.APPROVED_MESSAGES_FILE, module: 'messages' },
  { path: process.env.APPROVED_EMAIL_FILE, module: 'email' },
].filter((file) => file.path);

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

function hasOwn(value, key) {
  return Object.prototype.hasOwnProperty.call(asRecord(value), key);
}

function text(value) {
  return typeof value === 'string' ? value.trim() : '';
}

function bool(value) {
  return value === true;
}

function number(value) {
  return Number.isFinite(Number(value)) ? Number(value) : 0;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function sourceStatusFile() {
  return join(sourceStatusOutput, 'state', 'live-app-source-status.json');
}

function approvalRequestFile() {
  return join(sourceStatusOutput, 'state', 'approved-communications-request.json');
}

function approvalRequestDocument({ missing = ['messages', 'email'], lastError = null } = {}) {
  return {
    schema: 'memd.approved-communications-request.v1',
    status: 'needs_user_or_process_approval',
    approval: {
      set: 'APPROVED_COMMUNICATIONS_FILE',
      path: approvalRequestFile(),
      emptyApproval: 'APPROVED_COMMUNICATIONS_EMPTY_APPROVED=1',
      emptyApprovalRule:
        'Use empty approval only when the user/process explicitly approves zero message/email metadata.',
    },
    missing,
    lastError,
    privacyContract: [
      'Only metadata or redacted snippets are allowed.',
      'Every message/email item must set approved=true.',
      'redactedSnippet requires redacted=true or redactionApproved=true.',
      'Attachment/media metadata requires agentsecretsApproved=true.',
      'Raw chat/mail body text, HTML, transcripts, blobs, and raw media are rejected.',
    ],
    template: {
      messages: [
        {
          approved: true,
          contact: 'Approved contact name',
          threadId: 'optional-thread-id',
          unreadCount: 0,
          lastMessageAt: '2026-05-18T00:00:00Z',
          topic: 'Short approved metadata topic',
          redacted: true,
          redactedSnippet: 'Optional user-approved redacted snippet',
        },
      ],
      email: [
        {
          approved: true,
          from: 'approved-sender@example.com',
          subject: 'Approved subject metadata',
          folder: 'INBOX',
          receivedAt: '2026-05-18T00:00:00Z',
          unread: false,
          redacted: true,
          redactedSnippet: 'Optional user-approved redacted snippet',
        },
      ],
    },
  };
}

function writeApprovalRequest(options) {
  if (dryRun) return null;
  const path = approvalRequestFile();
  mkdirSync(join(sourceStatusOutput, 'state'), { recursive: true });
  writeFileSync(path, `${JSON.stringify(approvalRequestDocument(options), null, 2)}\n`);
  return path;
}

function readSourceStatusStore() {
  const path = sourceStatusFile();
  if (!existsSync(path)) {
    return { version: 1, updated_at: null, sources: [] };
  }
  try {
    const store = JSON.parse(readFileSync(path, 'utf8'));
    return {
      version: Math.max(1, Number(store.version || 1)),
      updated_at: store.updated_at || null,
      sources: Array.isArray(store.sources)
        ? store.sources.filter((source) => source && typeof source === 'object')
        : [],
    };
  } catch {
    return { version: 1, updated_at: null, sources: [] };
  }
}

function writeSourceStatus({ status, produced = [], missing = [], lastError = null }) {
  if (dryRun) return;
  const now = new Date().toISOString();
  const store = readSourceStatusStore();
  const approvalRequestPath =
    status === 'missing_approval' || status === 'invalid_approval'
      ? writeApprovalRequest({ missing, lastError })
      : null;
  const endpoints = ['messages', 'email'].map((module) => {
    const ok = produced.includes(module);
    return {
      module,
      path: `approved-communications:${module}`,
      api_base: 'approved-communications',
      ok,
      status: ok ? 200 : 0,
      error: ok ? undefined : lastError || 'approved communications metadata missing',
    };
  });
  const source = {
    source_app: 'approved_communications',
    status,
    checked_at: now,
    api_base: 'approved-communications',
    api_bases: ['approved-communications'],
    auth_configured: files.length > 0 || approvedEmpty,
    visible_page: 'not_applicable',
    produced,
    missing,
    record_count: produced.length,
    endpoints,
    last_error: lastError,
    approval_request_path: approvalRequestPath,
  };
  store.version = 1;
  store.updated_at = now;
  store.sources = store.sources.filter(
    (existing) =>
      !(
        (existing.source_app === source.source_app ||
          existing.source_app === 'clawcontrol' ||
          existing.source_app === 'approved-communications') &&
        (existing.api_base === source.api_base ||
          (Array.isArray(existing.api_bases) &&
            existing.api_bases.includes(source.api_base)))
      ),
  );
  store.sources.push(source);
  const path = sourceStatusFile();
  mkdirSync(join(sourceStatusOutput, 'state'), { recursive: true });
  writeFileSync(path, `${JSON.stringify(store, null, 2)}\n`);
}

const forbiddenPayloadKeys = new Set([
  'base64',
  'blob',
  'body',
  'body_html',
  'body_text',
  'bytes',
  'content',
  'content_html',
  'content_text',
  'data',
  'data_url',
  'html',
  'html_body',
  'plaintext',
  'plain_text',
  'raw',
  'rawbody',
  'raw_body',
  'text',
  'transcript',
]);

function assertNoRawContent(value, path = []) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => assertNoRawContent(item, [...path, String(index)]));
    return;
  }
  if (!value || typeof value !== 'object') {
    if (typeof value === 'string' && looksLikeRawMedia(value)) {
      throw new Error(`raw media blob is not allowed at ${path.join('.') || 'input'}`);
    }
    return;
  }
  for (const [key, child] of Object.entries(value)) {
    const normalized = key.trim().toLowerCase().replace(/[-\s]/g, '_');
    if (isForbiddenPayloadKey(normalized)) {
      throw new Error(`raw communication field is not allowed: ${[...path, key].join('.')}`);
    }
    assertNoRawContent(child, [...path, key]);
  }
}

function isForbiddenPayloadKey(normalized) {
  return (
    forbiddenPayloadKeys.has(normalized) ||
    normalized.includes('body') ||
    normalized === 'html' ||
    normalized.endsWith('_html') ||
    normalized === 'text' ||
    normalized.endsWith('_text')
  );
}

function looksLikeRawMedia(value) {
  const trimmed = value.trim();
  const lower = trimmed.toLowerCase();
  return (
    lower.startsWith('data:image/') ||
    lower.startsWith('data:video/') ||
    lower.startsWith('data:audio/') ||
    (trimmed.length > 256 &&
      /^[A-Za-z0-9+/=\r\n]+$/.test(trimmed) &&
      /[+/=]/.test(trimmed))
  );
}

function hasMediaRef(value) {
  if (Array.isArray(value)) return value.some(hasMediaRef);
  if (!value || typeof value !== 'object') return false;
  return Object.entries(value).some(([key, child]) => {
    const normalized = key.trim().toLowerCase().replace(/[-\s]/g, '_');
    if (
      [
        'attachment',
        'attachments',
        'audio',
        'file',
        'files',
        'image',
        'images',
        'media',
        'photo',
        'photos',
        'video',
      ].includes(normalized)
    ) {
      return true;
    }
    return hasMediaRef(child);
  });
}

function requireApproved(item, module) {
  if (item.approved !== true) {
    throw new Error(`${module} item missing approved=true`);
  }
  if (hasMediaRef(item) && item.agentsecretsApproved !== true) {
    throw new Error(`${module} media metadata requires agentsecretsApproved=true`);
  }
}

function isoish(value) {
  const raw = text(value);
  if (!raw) return '';
  const date = new Date(raw);
  return Number.isNaN(date.getTime()) ? compact(raw, 80) : date.toISOString();
}

function redactedSnippet(record, module) {
  const raw = text(record.redactedSnippet) || text(record.redacted_snippet);
  if (!raw) return '';
  if (
    record.redacted !== true &&
    record.redactionApproved !== true &&
    record.redactedApproved !== true
  ) {
    throw new Error(`${module} redacted snippet requires redacted=true or redactionApproved=true`);
  }
  if (looksLikeRawMedia(raw)) {
    throw new Error(`${module} redacted snippet must not contain raw media`);
  }
  return compact(raw, 160);
}

function sanitizeMessage(item) {
  const record = asRecord(item);
  assertNoRawContent(record);
  requireApproved(record, 'messages');
  const contact = compact(text(record.contact) || text(record.displayName) || text(record.name) || 'approved contact', 72);
  const threadId = compact(text(record.threadId) || text(record.chatIdentifier) || text(record.id), 96);
  const unreadCount = number(record.unreadCount ?? record.unread_count);
  const lastAt = isoish(record.lastMessageAt ?? record.last_at ?? record.updatedAt);
  return {
    contact,
    threadId,
    unreadCount,
    lastAt,
    topic: compact(text(record.topic) || text(record.summary), 120),
    redactedSnippet: redactedSnippet(record, 'messages'),
    hasAttachments: hasMediaRef(record),
  };
}

function sanitizeEmail(item) {
  const record = asRecord(item);
  assertNoRawContent(record);
  requireApproved(record, 'email');
  return {
    from: compact(text(record.from) || text(record.sender) || 'approved sender', 72),
    subject: compact(text(record.subject) || 'No subject', 120),
    folder: compact(text(record.folder) || 'INBOX', 48),
    receivedAt: isoish(record.receivedAt ?? record.date ?? record.updatedAt),
    unread: record.unread === true || record.read === false,
    redactedSnippet: redactedSnippet(record, 'email'),
    hasAttachments: hasMediaRef(record),
  };
}

function record({ module, summary, payload, labels, agentsecretsApproved }) {
  return {
    sourceApp: 'approved_communications',
    module,
    scope: 'approved',
    visibility: 'private',
    privacy: 'metadata',
    approved: true,
    agentsecretsApproved,
    freshnessSecs,
    labels: ['live-app-state', module, 'metadata', 'approved-communications', ...labels],
    summary,
    payload: {
      producer: 'approved-communications',
      sourceAppAlias: 'approved_communications',
      ...payload,
    },
  };
}

function recordsFromDocument(document, preferredModule = '') {
  const data = Array.isArray(document)
    ? { [preferredModule || 'messages']: document }
    : asRecord(document);
  const records = [];

  const hasMessages = Array.isArray(data.messages);
  const messages = asArray(data.messages);
  if (hasMessages) {
    const sanitized = messages.map(sanitizeMessage).slice(0, 12);
    const hasAttachments = sanitized.some((item) => item.hasAttachments);
    const hasRedactedSnippet = sanitized.some((item) => item.redactedSnippet);
    records.push(
      record({
        module: 'messages',
        labels: [
          'no-raw-chat',
          ...(hasRedactedSnippet ? ['redacted-snippet'] : []),
          ...(hasAttachments ? ['media-metadata'] : []),
        ],
        agentsecretsApproved: hasAttachments,
        summary:
          sanitized.length === 0
            ? 'messages: approved metadata loaded; conversations=0'
            : [
                `messages: approved metadata loaded; conversations=${sanitized.length}`,
                ...sanitized.slice(0, 8).map((item) => {
                  const unread = item.unreadCount > 0 ? ` | unread ${item.unreadCount}` : '';
                  const snippet = item.redactedSnippet ? ` | redacted: ${item.redactedSnippet}` : '';
                  return `- ${item.contact}${unread}${item.topic ? ` | ${item.topic}` : ''}${snippet}`;
                }),
              ].join('\n'),
        payload: { conversations: sanitized },
      }),
    );
  }

  const emailInput = hasOwn(data, 'email') ? data.email : data.emails;
  const hasEmail = Array.isArray(emailInput);
  const emails = asArray(emailInput);
  if (hasEmail) {
    const sanitized = emails.map(sanitizeEmail).slice(0, 12);
    const hasAttachments = sanitized.some((item) => item.hasAttachments);
    const hasRedactedSnippet = sanitized.some((item) => item.redactedSnippet);
    records.push(
      record({
        module: 'email',
        labels: [
          'no-raw-mail',
          ...(hasRedactedSnippet ? ['redacted-snippet'] : []),
          ...(hasAttachments ? ['media-metadata'] : []),
        ],
        agentsecretsApproved: hasAttachments,
        summary:
          sanitized.length === 0
            ? 'email: approved metadata loaded; inbox_items=0'
            : [
                `email: approved metadata loaded; inbox_items=${sanitized.length}`,
                ...sanitized.slice(0, 8).map((item) => {
                  const unread = item.unread ? ' | unread' : '';
                  const snippet = item.redactedSnippet ? ` | redacted: ${item.redactedSnippet}` : '';
                  return `- ${item.from}: ${item.subject}${unread}${snippet}`;
                }),
              ].join('\n'),
        payload: { emails: sanitized },
      }),
    );
  }

  return records;
}

if (files.length === 0 && !approvedEmpty) {
  writeSourceStatus({
    status: 'missing_approval',
    missing: ['messages', 'email'],
    lastError: 'no approved communications file or empty approval configured',
  });
  console.error('live-state-capture-approved-communications: no approved communications file or empty approval configured');
  process.exit(2);
}

let records = [];
try {
  records = [
    ...(approvedEmpty ? recordsFromDocument({ messages: [], email: [] }) : []),
    ...files.flatMap((file) => recordsFromDocument(readJson(file.path), file.module)),
  ];
} catch (error) {
  writeSourceStatus({
    status: 'invalid_approval',
    missing: ['messages', 'email'],
    lastError: compact(error.message, 240),
  });
  console.error(`live-state-capture-approved-communications: ${compact(error.message, 240)}`);
  process.exit(1);
}

if (records.length === 0) {
  writeSourceStatus({
    status: 'missing_approval',
    missing: ['messages', 'email'],
    lastError: 'no approved messages/email metadata found',
  });
  console.error('live-state-capture-approved-communications: no approved messages/email metadata found');
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
  writeSourceStatus({
    status: 'ingest_failed',
    produced: records.map((item) => item.module),
    missing: ['messages', 'email'].filter(
      (module) => !records.some((item) => item.module === module),
    ),
    lastError: compact(stderr || 'ingest failed', 240),
  });
  if (stderr) {
    console.error(`live-state-capture-approved-communications: ingest failed: ${compact(stderr, 240)}`);
  }
  process.exit(result.status ?? 1);
}
const produced = records.map((item) => item.module);
const missing = ['messages', 'email'].filter((module) => !produced.includes(module));
writeSourceStatus({
  status: missing.length === 0 ? 'ok' : 'partial',
  produced,
  missing,
  lastError: missing.length ? `approved communications missing: ${missing.join(',')}` : null,
});
console.error(
  `live-state-capture-approved-communications: captured records=${records.length} modules=${records
    .map((item) => item.module)
    .join(',')} privacy=metadata visibility=private`,
);
