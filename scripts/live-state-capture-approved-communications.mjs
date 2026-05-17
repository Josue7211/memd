#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = new URL('..', import.meta.url).pathname.replace(/\/+$/, '');
const memdBin = process.env.MEMD_BIN || 'memd';
const memdOutput = process.env.MEMD_OUTPUT || join(root, '.memd');
const freshnessSecs = Math.max(60, Number(process.env.FRESHNESS_SECS || '3600'));
const dryRun = process.env.DRY_RUN === '1' || process.env.DRY_RUN === 'true';
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
    hasAttachments: hasMediaRef(record),
  };
}

function record({ module, summary, payload, labels, agentsecretsApproved }) {
  return {
    sourceApp: 'clawcontrol',
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
      sourceAppAlias: 'clawcontrol',
      ...payload,
    },
  };
}

function recordsFromDocument(document, preferredModule = '') {
  const data = Array.isArray(document)
    ? { [preferredModule || 'messages']: document }
    : asRecord(document);
  const records = [];

  const messages = asArray(data.messages);
  if (messages.length) {
    const sanitized = messages.map(sanitizeMessage).slice(0, 12);
    const hasAttachments = sanitized.some((item) => item.hasAttachments);
    records.push(
      record({
        module: 'messages',
        labels: ['no-raw-chat', ...(hasAttachments ? ['media-metadata'] : [])],
        agentsecretsApproved: hasAttachments,
        summary:
          sanitized.length === 0
            ? 'messages: approved metadata loaded; conversations=0'
            : [
                `messages: approved metadata loaded; conversations=${sanitized.length}`,
                ...sanitized.slice(0, 8).map((item) => {
                  const unread = item.unreadCount > 0 ? ` | unread ${item.unreadCount}` : '';
                  return `- ${item.contact}${unread}${item.topic ? ` | ${item.topic}` : ''}`;
                }),
              ].join('\n'),
        payload: { conversations: sanitized },
      }),
    );
  }

  const emails = asArray(data.email || data.emails);
  if (emails.length) {
    const sanitized = emails.map(sanitizeEmail).slice(0, 12);
    const hasAttachments = sanitized.some((item) => item.hasAttachments);
    records.push(
      record({
        module: 'email',
        labels: ['no-raw-mail', ...(hasAttachments ? ['media-metadata'] : [])],
        agentsecretsApproved: hasAttachments,
        summary:
          sanitized.length === 0
            ? 'email: approved metadata loaded; inbox_items=0'
            : [
                `email: approved metadata loaded; inbox_items=${sanitized.length}`,
                ...sanitized.slice(0, 8).map((item) => {
                  const unread = item.unread ? ' | unread' : '';
                  return `- ${item.from}: ${item.subject}${unread}`;
                }),
              ].join('\n'),
        payload: { emails: sanitized },
      }),
    );
  }

  return records;
}

if (files.length === 0) {
  console.error('live-state-capture-approved-communications: no approved communications file configured');
  process.exit(2);
}

let records = [];
try {
  records = files.flatMap((file) => recordsFromDocument(readJson(file.path), file.module));
} catch (error) {
  console.error(`live-state-capture-approved-communications: ${compact(error.message, 240)}`);
  process.exit(1);
}

if (records.length === 0) {
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
  if (stderr) {
    console.error(`live-state-capture-approved-communications: ingest failed: ${compact(stderr, 240)}`);
  }
  process.exit(result.status ?? 1);
}
console.error(
  `live-state-capture-approved-communications: captured records=${records.length} modules=${records
    .map((item) => item.module)
    .join(',')} privacy=metadata visibility=private`,
);
