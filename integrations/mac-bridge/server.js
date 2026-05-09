import express from 'express'
import { execFile } from 'child_process'
import { promisify } from 'util'
import { homedir, tmpdir } from 'os'
import { join, resolve, extname } from 'path'
import { createHash, timingSafeEqual, randomBytes } from 'crypto'
import { mkdirSync, chmodSync, lstatSync, readFileSync, writeFileSync } from 'fs'

const execFileP = promisify(execFile)
const app = express()
app.disable('x-powered-by') // don't leak server technology

// ── Input limits ────────────────────────────────────────────────────
const MAX_STRING_LENGTH = 10000
const MAX_ARRAY_LENGTH = 100

// Private temp directory for mac-bridge (owner-only permissions)
const BRIDGE_TMP = join(tmpdir(), 'mac-bridge-private')
const CALENDAR_CACHE = join(BRIDGE_TMP, 'calendar-cache.json')
try {
  // Prevent symlink attack: if path exists as symlink, refuse to use it
  try { if (lstatSync(BRIDGE_TMP).isSymbolicLink()) { throw new Error('BRIDGE_TMP is a symlink') } } catch (e) { if (e.code !== 'ENOENT') throw e }
  mkdirSync(BRIDGE_TMP, { recursive: true, mode: 0o700 })
  chmodSync(BRIDGE_TMP, 0o700)
} catch (e) { console.warn('Failed to create private temp dir:', e.code || e.message) }

// Sanitize user input for safe interpolation into JXA/AppleScript double-quoted strings
function safeJxaString(s) {
  return String(s)
    .slice(0, MAX_STRING_LENGTH)
    .replace(/\\/g, '\\\\')
    .replace(/`/g, '\\`')    // prevent JXA template literal breakout
    .replace(/\$/g, '\\$')   // prevent template expression injection
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '')
    .replace(/\0/g, '')      // strip null bytes
}

// Reject strings that look like CLI flags (argument injection prevention)
function assertNotFlag(s) {
  s = String(s) // coerce to string to prevent toString() bypass
  if (s.startsWith('-')) {
    throw new Error('invalid input: value must not start with -')
  }
  return s
}

app.use(express.json({ limit: '1mb' }))

const PORT = process.env.BRIDGE_PORT || 4100
const API_KEY = process.env.BRIDGE_API_KEY || ''
const HOME = homedir()

// ── Auth middleware ─────────────────────────────────────────────────

if (!API_KEY) {
  console.error('FATAL: BRIDGE_API_KEY is not set. Refusing to start without auth.')
  process.exit(1)
}

// Simple in-memory rate limiter with periodic cleanup
const rateBuckets = new Map()
function rateLimit(key, maxPerMinute) {
  const now = Date.now()
  const bucket = rateBuckets.get(key) || { count: 0, resetAt: now + 60000 }
  if (now > bucket.resetAt) { bucket.count = 0; bucket.resetAt = now + 60000 }
  bucket.count++
  rateBuckets.set(key, bucket)
  return bucket.count > maxPerMinute
}
// Prune expired rate-limit entries every 5 minutes to prevent memory leak
setInterval(() => {
  const now = Date.now()
  for (const [key, bucket] of rateBuckets) {
    if (now > bucket.resetAt) rateBuckets.delete(key)
  }
}, 300000).unref()

// ── Security headers ────────────────────────────────────────────────
app.use((_req, res, next) => {
  res.set('X-Content-Type-Options', 'nosniff')
  res.set('X-Frame-Options', 'DENY')
  res.set('Cache-Control', 'no-store')
  res.set('Content-Security-Policy', "default-src 'none'")
  next()
})

app.use((req, res, next) => {
  const key = req.headers['x-api-key'] || ''
  // Constant-time comparison to prevent timing attacks
  if (typeof key !== 'string' || key.length === 0) {
    return res.status(401).json({ error: 'unauthorized' })
  }
  try {
    const keyBuf = Buffer.from(key, 'utf-8')
    const expectedBuf = Buffer.from(API_KEY, 'utf-8')
    // timingSafeEqual requires same length — hash both to normalize
    const keyHash = createHash('sha256').update(keyBuf).digest()
    const expectedHash = createHash('sha256').update(expectedBuf).digest()
    if (!timingSafeEqual(keyHash, expectedHash)) {
      return res.status(401).json({ error: 'unauthorized' })
    }
  } catch {
    return res.status(401).json({ error: 'unauthorized' })
  }
  // Rate limit: 60 requests per minute per IP
  const clientIp = req.ip || 'unknown'
  if (rateLimit(clientIp, 60)) {
    return res.status(429).json({ error: 'rate limit exceeded' })
  }
  next()
})

// ── Helpers ─────────────────────────────────────────────────────────

// Sanitize error messages to prevent internal detail leakage
function safeError(err) {
  const msg = String(err?.message || 'unknown error')
  // Strip ALL filesystem paths (not just /Users/) and line:col references
  return msg
    .replace(/\/(?:Users|var|tmp|opt|private|Library|usr|etc)\/[^\s:]+/g, '<redacted-path>')
    .replace(/:\d+:\d+/g, '')  // strip line:column references
    .slice(0, 200)
}

async function remindctl(...args) {
  const { stdout } = await execFileP('remindctl', [...args, '--json'], { timeout: 10000 })
  try { return JSON.parse(stdout) } catch { throw new Error('remindctl returned invalid JSON') }
}

function reminderFilter(filter) {
  const value = String(filter || 'all').trim()
  if (value === 'incomplete') return 'open'
  if (value === 'scheduled') return 'upcoming'
  return value || 'all'
}

function reminderPriority(value) {
  if (value === undefined || value === null || value === '') return ''
  const text = String(value).trim().toLowerCase()
  if (['none', 'low', 'medium', 'high'].includes(text)) return text
  const number = Number(text)
  if (!Number.isFinite(number) || number <= 0) return 'none'
  if (number === 1) return 'high'
  if (number === 5) return 'medium'
  if (number === 9) return 'low'
  if (number <= 4) return 'high'
  if (number <= 8) return 'medium'
  return 'low'
}

function reminderDueValue(value) {
  if (value === undefined || value === null || value === '') return ''
  if (typeof value === 'object') {
    return String(value.date || value.datetime || value.iso || value.value || '').trim()
  }
  return String(value).trim()
}

async function updateReminder(req, res) {
  try {
    const id = assertNotFlag(String(req.params?.id || req.body?.id || '').slice(0, 500))
    if (!id) return res.status(400).json({ error: 'id required' })
    const args = ['edit', id]
    if (req.body?.title) args.push('--title', assertNotFlag(String(req.body.title).slice(0, 500)))
    if (req.body?.list) args.push('--list', assertNotFlag(String(req.body.list).slice(0, 200)))
    if (req.body?.notes !== undefined) args.push('--notes', assertNotFlag(String(req.body.notes).slice(0, 1000)))
    if (req.body?.dueDate === null || req.body?.due === null) {
      args.push('--clear-due')
    } else {
      const due = reminderDueValue(req.body?.dueDate || req.body?.due)
      if (due) args.push('--due', assertNotFlag(due.slice(0, 100)))
    }
    const priority = reminderPriority(req.body?.priority)
    if (priority) args.push('--priority', priority)
    if (req.body?.completed === true) args.push('--complete')
    if (req.body?.completed === false) args.push('--incomplete')
    if (args.length === 2) return res.status(400).json({ error: 'nothing to update' })
    res.json(await remindctl(...args))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
}

async function deleteReminder(req, res) {
  try {
    const id = assertNotFlag(String(req.params?.id || req.query?.id || req.body?.id || req.body?.ids?.[0] || '').slice(0, 500))
    if (!id) return res.status(400).json({ error: 'id required' })
    res.json(await remindctl('delete', id, '--force'))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
}

async function osascript(script) {
  const { stdout } = await execFileP('osascript', ['-e', script], { timeout: 15000 })
  return stdout.trim()
}

async function jxa(script) {
  const { stdout } = await execFileP('osascript', ['-l', 'JavaScript', '-e', script], { timeout: 15000 })
  try { return JSON.parse(stdout) } catch { throw new Error('JXA returned invalid JSON') }
}

function readCalendarCache() {
  try {
    const cached = JSON.parse(readFileSync(CALENDAR_CACHE, 'utf8'))
    if (Array.isArray(cached.events)) return cached
  } catch {}
  return null
}

function writeCalendarCache(events) {
  try {
    writeFileSync(CALENDAR_CACHE, JSON.stringify({ events, cachedAt: new Date().toISOString() }), { mode: 0o600 })
  } catch (err) {
    console.warn('Calendar cache write failed:', err?.message || err)
  }
}

function calendarDeletePayload(req) {
  const body = req.body && typeof req.body === 'object' ? req.body : {}
  const rawId = req.params?.id || req.query?.id || body.id || body.appleEventId || body.objectUrl
  const id = rawId ? assertNotFlag(String(rawId).slice(0, 500)) : ''
  const appleEventId = body.appleEventId ? assertNotFlag(String(body.appleEventId).slice(0, 500)) : ''
  const calendar = body.calendar ? String(body.calendar).slice(0, 500) : ''
  const title = body.title ? String(body.title).slice(0, 500) : ''
  const start = body.start ? String(body.start).slice(0, 100) : ''
  const end = body.end ? String(body.end).slice(0, 100) : ''
  const localId = body.localId || req.query?.localId
    ? assertNotFlag(String(body.localId || req.query.localId).slice(0, 100))
    : ''
  return { id, appleEventId, calendar, title, start, end, localId }
}

function normalizeCalendarInput(body) {
  const title = String(body?.title || '').trim().slice(0, 500)
  if (!title) throw new Error('title required')
  const start = String(body?.start || '').trim().slice(0, 100)
  if (!start) throw new Error('start required')
  const startDate = new Date(start)
  if (Number.isNaN(startDate.getTime())) throw new Error('invalid start')
  const allDay = Boolean(body?.allDay)
  const endRaw = String(body?.end || '').trim().slice(0, 100)
  const endDate = endRaw
    ? new Date(endRaw)
    : new Date(startDate.getTime() + (allDay ? 24 * 60 * 60 * 1000 : 60 * 60 * 1000))
  if (Number.isNaN(endDate.getTime())) throw new Error('invalid end')
  if (endDate <= startDate) throw new Error('end must be after start')
  const calendar = String(body?.calendar || '').trim().slice(0, 500)
  return { title, start: startDate.toISOString(), end: endDate.toISOString(), allDay, calendar }
}

async function createCalendarEvent(req, res) {
  try {
    const event = normalizeCalendarInput(req.body)
    const script = `
      const Calendar = Application("Calendar");
      Calendar.launch();
      const calendarName = "${safeJxaString(event.calendar)}";
      const calendars = Calendar.calendars();
      let target = null;
      if (calendarName) {
        for (const cal of calendars) {
          if (cal.name() === calendarName) {
            target = cal;
            break;
          }
        }
      }
      if (!target) {
        const preferred = ["Calendar", "Home", "Work"];
        for (const wanted of preferred) {
          for (const cal of calendars) {
            if (cal.name() === wanted) {
              target = cal;
              break;
            }
          }
          if (target) break;
        }
      }
      if (!target) target = calendars[0];
      if (!target) throw new Error("no calendar available");
      const ev = Calendar.Event({
        summary: "${safeJxaString(event.title)}",
        startDate: new Date("${safeJxaString(event.start)}"),
        endDate: new Date("${safeJxaString(event.end)}"),
        alldayEvent: ${event.allDay ? 'true' : 'false'}
      });
      target.events.push(ev);
      JSON.stringify({
        ok: true,
        event: {
          id: ev.id(),
          appleEventId: ev.id(),
          objectUrl: null,
          localId: null,
          title: ev.summary(),
          start: ev.startDate().toISOString(),
          end: ev.endDate().toISOString(),
          allDay: Boolean(ev.alldayEvent()),
          calendar: target.name()
        }
      });
    `
    res.json(await jxa(script))
  } catch (err) {
    res.status(400).json({ error: safeError(err) })
  }
}

async function updateCalendarEvent(req, res) {
  try {
    const payload = calendarDeletePayload(req)
    const targetId = payload.appleEventId || payload.id
    if (!targetId) return res.status(400).json({ error: 'id required' })
    const updates = normalizeCalendarInput({ ...req.body, title: req.body?.title || payload.title || 'Untitled event' })
    const script = `
      const Calendar = Application("Calendar");
      Calendar.launch();
      const eventId = "${safeJxaString(targetId)}";
      const calendarName = "${safeJxaString(payload.calendar)}";
      let found = null;
      let foundCalendar = null;
      for (const cal of Calendar.calendars()) {
        if (calendarName && cal.name() !== calendarName) continue;
        const matches = cal.events.whose({ id: eventId })();
        if (matches.length > 0) {
          found = matches[0];
          foundCalendar = cal;
          break;
        }
      }
      if (!found) {
        for (const cal of Calendar.calendars()) {
          const matches = cal.events.whose({ id: eventId })();
          if (matches.length > 0) {
            found = matches[0];
            foundCalendar = cal;
            break;
          }
        }
      }
      if (!found) throw new Error("calendar event not found");
      const nextStart = new Date("${safeJxaString(updates.start)}");
      const nextEnd = new Date("${safeJxaString(updates.end)}");
      const currentStart = found.startDate();
      const currentEnd = found.endDate();
      found.summary.set("${safeJxaString(updates.title)}");
      if (nextStart >= currentEnd) {
        found.endDate.set(nextEnd);
        found.startDate.set(nextStart);
      } else if (nextEnd <= currentStart) {
        found.startDate.set(nextStart);
        found.endDate.set(nextEnd);
      } else {
        found.startDate.set(nextStart);
        found.endDate.set(nextEnd);
      }
      found.alldayEvent.set(${updates.allDay ? 'true' : 'false'});
      JSON.stringify({
        ok: true,
        event: {
          id: found.id(),
          appleEventId: found.id(),
          objectUrl: null,
          localId: null,
          title: found.summary(),
          start: found.startDate().toISOString(),
          end: found.endDate().toISOString(),
          allDay: Boolean(found.alldayEvent()),
          calendar: foundCalendar.name()
        }
      });
    `
    res.json(await jxa(script))
  } catch (err) {
    res.status(400).json({ error: safeError(err) })
  }
}

async function deleteCalendarViaApp({ id, appleEventId, calendar }) {
  const targetId = appleEventId || id
  if (!targetId) return false
  const script = `
on deleteEvent(eventId, calendarName)
  launch application "Calendar"
  tell application "Calendar"
    if calendarName is not "" then
      repeat with cal in calendars
        if name of cal is calendarName then
          tell cal
            set matches to (events whose id is eventId)
            if (count of matches) > 0 then
              delete item 1 of matches
            else
              error "event not found in calendar"
            end if
          end tell
          return "deleted"
        end if
      end repeat
    end if

    repeat with cal in calendars
      try
        tell cal
          set matches to (events whose id is eventId)
          if (count of matches) > 0 then
            delete item 1 of matches
            return "deleted"
          end if
        end tell
      end try
    end repeat
  end tell
  error "event not found in Calendar.app"
end deleteEvent

on run
  return deleteEvent("${safeJxaString(targetId)}", "${safeJxaString(calendar)}")
end run
  `
  try {
    await osascript(script)
    return true
  } catch (err) {
    console.warn('Calendar.app delete failed:', err?.stderr || err?.message || err)
    return false
  }
}

async function hideCalendarSuggestion({ localId, title, start, end }) {
  if (!localId || !/^\d+$/.test(localId)) return false
  const calendarDb = join(HOME, 'Library/Group Containers/group.com.apple.calendar/Calendar.sqlitedb')
  const guards = [`ROWID = ${localId}`]
  if (title) guards.push(`COALESCE(summary, '') = '${String(title).replace(/'/g, "''")}'`)
  if (start) guards.push(`strftime('%Y-%m-%dT%H:%M:%SZ', start_date + 978307200, 'unixepoch') = '${String(start).replace(/'/g, "''")}'`)
  if (end) guards.push(`strftime('%Y-%m-%dT%H:%M:%SZ', end_date + 978307200, 'unixepoch') = '${String(end).replace(/'/g, "''")}'`)
  const sql = `UPDATE CalendarItem SET hidden = 1 WHERE ${guards.join(' AND ')}; SELECT changes();`
  try {
    const { stdout } = await execFileP('/usr/bin/sqlite3', [calendarDb, sql], { timeout: 10000 })
    return parseInt(String(stdout).trim(), 10) > 0
  } catch (err) {
    console.warn('Calendar sqlite hide failed:', err?.stderr || err?.message || err)
    return false
  }
}

async function deleteCalendarEvent(req, res) {
  try {
    const payload = calendarDeletePayload(req)
    if (!payload.id && !payload.appleEventId && !payload.localId) {
      return res.status(400).json({ error: 'id required' })
    }
    if (await deleteCalendarViaApp(payload)) {
      return res.json({ ok: true, source: 'calendar-app' })
    }
    if (await hideCalendarSuggestion(payload)) {
      return res.json({ ok: true, source: 'calendar-suggestion-hidden' })
    }
    res.status(404).json({ error: 'calendar event not found' })
  } catch (err) {
    res.status(500).json({ error: safeError(err) })
  }
}


// ── Health ──────────────────────────────────────────────────────────

app.get('/health', (_req, res) => {
  res.json({ ok: true, services: ['reminders', 'calendar', 'notes', 'contacts', 'findmy'] })
})

// ═══════════════════════════════════════════════════════════════════
// REMINDERS
// ═══════════════════════════════════════════════════════════════════

app.get('/reminders', async (req, res) => {
  try {
    const allowed = ['all', 'incomplete', 'open', 'completed', 'today', 'tomorrow', 'week', 'overdue', 'upcoming', 'scheduled']
    const filter = allowed.includes(req.query.filter) ? reminderFilter(req.query.filter) : 'all'
    res.json(await remindctl('show', filter))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.get('/reminders/lists', async (_req, res) => {
  try { res.json(await remindctl('list')) }
  catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.get('/reminders/lists/:name', async (req, res) => {
  try { res.json(await remindctl('list', assertNotFlag(req.params.name))) }
  catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.post('/reminders', async (req, res) => {
  try {
    const { title, list, due, dueDate, notes, priority } = req.body
    if (!title) return res.status(400).json({ error: 'title required' })
    const safeTitle = assertNotFlag(String(title).slice(0, MAX_STRING_LENGTH))
    const args = ['add', safeTitle]
    if (list) args.push('--list', assertNotFlag(String(list).slice(0, 200)))
    const dueValue = reminderDueValue(dueDate || due)
    if (dueValue) args.push('--due', assertNotFlag(dueValue.slice(0, 100)))
    if (notes !== undefined) args.push('--notes', assertNotFlag(String(notes).slice(0, 1000)))
    const priorityValue = reminderPriority(priority)
    if (priorityValue) args.push('--priority', priorityValue)
    res.json(await remindctl(...args))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.patch('/reminders/:id', updateReminder)
app.patch('/reminders', updateReminder)
app.post('/reminders/update', updateReminder)

app.post('/reminders/complete', async (req, res) => {
  try {
    const ids = Array.isArray(req.body.ids) ? req.body.ids.slice(0, MAX_ARRAY_LENGTH) : []
    if (req.body.id && ids.length === 0) ids.push(req.body.id)
    if (!ids.length) return res.status(400).json({ error: 'ids array required' })
    const safeIds = ids.map(id => assertNotFlag(String(id).slice(0, 200)))
    res.json(await remindctl('complete', ...safeIds))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.post('/reminders/uncomplete', async (req, res) => {
  req.body = { ...(req.body || {}), completed: false }
  return updateReminder(req, res)
})

app.delete('/reminders/:id', deleteReminder)
app.delete('/reminders', deleteReminder)
app.post('/reminders/delete', deleteReminder)

// ═══════════════════════════════════════════════════════════════════
// CALENDAR
// ═══════════════════════════════════════════════════════════════════

app.get('/calendar', async (req, res) => {
  try {
    const daysPast = Math.min(Math.max(parseInt(req.query.pastDays) || 30, 0), 366)
    const daysFuture = Math.min(Math.max(parseInt(req.query.futureDays) || 30, 1), 366)
    const calendarDb = join(HOME, 'Library/Group Containers/group.com.apple.calendar/Calendar.sqlitedb')
    const query = `
      SELECT
        ci.ROWID AS id,
        ci.ROWID AS localId,
        ci.UUID AS appleEventId,
        COALESCE(ci.external_id, '') AS objectUrl,
        COALESCE(ci.summary, '') AS title,
        strftime('%Y-%m-%dT%H:%M:%SZ', ci.start_date + 978307200, 'unixepoch') AS start,
        strftime('%Y-%m-%dT%H:%M:%SZ', ci.end_date + 978307200, 'unixepoch') AS end,
        CASE WHEN COALESCE(ci.all_day, 0) = 0 THEN 0 ELSE 1 END AS allDay,
        COALESCE(c.title, '') AS calendar
      FROM CalendarItem ci
      LEFT JOIN Calendar c ON c.ROWID = ci.calendar_id
      WHERE ci.start_date BETWEEN (strftime('%s','now','-${daysPast} days') - 978307200)
        AND (strftime('%s','now','+${daysFuture} days') - 978307200)
        AND COALESCE(ci.hidden, 0) = 0
      ORDER BY ci.start_date ASC
      LIMIT 500
    `
    try {
      const { stdout } = await execFileP('/usr/bin/sqlite3', ['-json', calendarDb, query], { timeout: 10000 })
      const events = JSON.parse(stdout || '[]').map(ev => ({
        id: String(ev.id),
        localId: ev.localId,
        appleEventId: ev.appleEventId || null,
        objectUrl: ev.objectUrl || null,
        title: ev.title || '',
        start: ev.start,
        end: ev.end,
        allDay: Boolean(ev.allDay),
        calendar: ev.calendar || '',
      }))
      writeCalendarCache(events)
      return res.json({ events })
    } catch (err) {
      console.warn('Calendar sqlite query failed:', err?.stderr || err?.message || err)
      // Fall back to Calendar.app automation when launchd's Node process lacks
      // Full Disk Access for the Calendar sqlite database.
    }

    const script = `
      const Calendar = Application("Calendar");
      const start = new Date(Date.now() - (${daysPast} * 24 * 60 * 60 * 1000));
      const end = new Date(Date.now() + (${daysFuture} * 24 * 60 * 60 * 1000));
      const rows = [];
      for (const cal of Calendar.calendars()) {
        const name = cal.name();
        const events = cal.events.whose({
          _and: [
            {startDate: {_lessThan: end}},
            {endDate: {_greaterThan: start}}
          ]
        })();
        for (const ev of events) {
          rows.push({
            id: ev.id(),
            appleEventId: ev.id(),
            objectUrl: null,
            localId: null,
            title: ev.summary(),
            start: ev.startDate().toISOString(),
            end: ev.endDate().toISOString(),
            allDay: Boolean(ev.alldayEvent()),
            calendar: name
          });
        }
      }
      JSON.stringify(rows.sort((a, b) => a.start.localeCompare(b.start)));
    `
    const events = await jxa(script)
    writeCalendarCache(events)
    res.json({ events })
  } catch (err) {
    console.warn('Calendar route failed:', err?.stderr || err?.message || err)
    const cached = readCalendarCache()
    if (cached) return res.json({ ...cached, source: 'cache' })
    res.status(500).json({ error: safeError(err) })
  }
})

app.post('/calendar', createCalendarEvent)
app.patch('/calendar/:id', updateCalendarEvent)
app.patch('/calendar', updateCalendarEvent)
app.delete('/calendar/:id', deleteCalendarEvent)
app.delete('/calendar', deleteCalendarEvent)
app.post('/calendar/delete', deleteCalendarEvent)

// ═══════════════════════════════════════════════════════════════════
// NOTES
// ═══════════════════════════════════════════════════════════════════

app.get('/notes', async (req, res) => {
  try {
    const limit = Math.min(Math.max(parseInt(req.query.limit) || 50, 1), 200)
    const folder = String(Array.isArray(req.query.folder) ? req.query.folder[0] : req.query.folder || '')
    const search = String(Array.isArray(req.query.search) ? req.query.search[0] : req.query.search || '')

    let script
    if (search) {
      script = `
        const Notes = Application("Notes");
        const results = Notes.notes.whose({name: {_contains: "${safeJxaString(search)}"}})();
        JSON.stringify(results.slice(0, ${limit}).map(n => ({
          id: n.id(), name: n.name(), body: n.plaintext().substring(0, 200),
          folder: n.container().name(), created: n.creationDate().toISOString(),
          modified: n.modificationDate().toISOString()
        })))
      `
    } else if (folder) {
      script = `
        const Notes = Application("Notes");
        const f = Notes.folders.byName("${safeJxaString(folder)}");
        const notes = f.notes();
        JSON.stringify(notes.slice(0, ${limit}).map(n => ({
          id: n.id(), name: n.name(), body: n.plaintext().substring(0, 200),
          folder: "${safeJxaString(folder)}", created: n.creationDate().toISOString(),
          modified: n.modificationDate().toISOString()
        })))
      `
    } else {
      script = `
        const Notes = Application("Notes");
        const notes = Notes.notes();
        JSON.stringify(notes.slice(0, ${limit}).map(n => ({
          id: n.id(), name: n.name(), body: n.plaintext().substring(0, 200),
          folder: n.container().name(), created: n.creationDate().toISOString(),
          modified: n.modificationDate().toISOString()
        })))
      `
    }
    res.json(await jxa(script))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.get('/notes/folders', async (_req, res) => {
  try {
    const script = `
      const Notes = Application("Notes");
      JSON.stringify(Notes.folders().map(f => ({ name: f.name(), count: f.notes().length })))
    `
    res.json(await jxa(script))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.get('/notes/:id', async (req, res) => {
  try {
    const id = safeJxaString(req.params.id)
    const script = `
      const Notes = Application("Notes");
      const n = Notes.notes.byId("${id}");
      JSON.stringify({
        id: n.id(), name: n.name(), body: n.plaintext(),
        html: n.body(), folder: n.container().name(),
        created: n.creationDate().toISOString(),
        modified: n.modificationDate().toISOString()
      })
    `
    res.json(await jxa(script))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

app.post('/notes', async (req, res) => {
  try {
    const { title, body, folder } = req.body
    if (!title) return res.status(400).json({ error: 'title required' })
    const safeTitle = safeJxaString(title)
    const safeBody = safeJxaString(body || '')
    const target = folder
      ? `folder "${safeJxaString(folder)}" of application "Notes"`
      : 'default account of application "Notes"'
    await osascript(`tell application "Notes" to make new note at ${target} with properties {name:"${safeTitle}", body:"${safeBody}"}`)
    res.json({ ok: true })
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

// ═══════════════════════════════════════════════════════════════════
// CONTACTS
// ═══════════════════════════════════════════════════════════════════

app.get('/contacts', async (req, res) => {
  try {
    const search = req.query.search || ''
    const limit = Math.min(Math.max(parseInt(req.query.limit) || 30, 1), 200)
    let script
    if (search) {
      const safe = safeJxaString(search)
      script = `
        const Contacts = Application("Contacts");
        const people = Contacts.people.whose({_or: [
          {firstName: {_contains: "${safe}"}},
          {lastName: {_contains: "${safe}"}}
        ]})();
        JSON.stringify(people.slice(0, ${limit}).map(p => ({
          id: p.id(), name: p.name(),
          phones: p.phones().map(ph => ({label: ph.label(), value: ph.value()})),
          emails: p.emails().map(e => ({label: e.label(), value: e.value()}))
        })))
      `
    } else {
      script = `
        const Contacts = Application("Contacts");
        const people = Contacts.people();
        JSON.stringify(people.slice(0, ${limit}).map(p => ({
          id: p.id(), name: p.name(),
          phones: p.phones().map(ph => ({label: ph.label(), value: ph.value()})),
          emails: p.emails().map(e => ({label: e.label(), value: e.value()}))
        })))
      `
    }
    res.json(await jxa(script))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

// ── Contact photos (must be before /contacts/:id) ────────────────

// Map: last 7 digits → private temp path
const photoMap = new Map()

async function buildPhotoCache() {
  // Use AppleScript to export all contact photos to private temp dir as TIFF, then convert to JPEG
  const script = `
    tell application "Contacts"
      set output to ""
      repeat with p in every person
        try
          set img to image of p
          if img is not missing value then
            set phoneList to value of every phone of p
            repeat with ph in phoneList
              set rawNum to ph as text
              set digits to do shell script "echo " & quoted form of rawNum & " | tr -cd '0-9'"
              if length of digits >= 7 then
                set last7 to text ((length of digits) - 6) thru (length of digits) of digits
                set tiffPath to "${BRIDGE_TMP}/mc-avatar-" & last7 & ".tiff"
                try
                  set fRef to open for access POSIX file tiffPath with write permission
                  set eof fRef to 0
                  write img to fRef
                  close access fRef
                  -- Convert TIFF to JPEG using sips
                  do shell script "sips -s format jpeg " & quoted form of tiffPath & " --out ${BRIDGE_TMP}/mc-avatar-" & last7 & ".jpg > /dev/null 2>&1 && rm -f " & quoted form of tiffPath
                  set output to output & last7 & linefeed
                end try
              end if
            end repeat
          end if
        end try
      end repeat
      return output
    end tell
  `
  try {
    const result = await osascript(script)
    const keys = result.split('\n').filter(k => k.length === 7)
    for (const key of keys) {
      photoMap.set(key, join(BRIDGE_TMP, `mc-avatar-${key}.jpg`))
    }
    console.log(`Photo cache built: ${photoMap.size} contact photos`)
  } catch (err) {
    console.warn('Photo cache build failed:', safeError(err))
  }
}

// Build in background on startup (can take 30-60s with many contacts)
buildPhotoCache()

app.get('/contacts/photo', (req, res) => {
  const address = req.query.address
  if (!address) return res.status(400).json({ error: 'address required' })

  const digits = String(address).replace(/\D/g, '')
  const last7 = digits.length >= 7 ? digits.slice(-7) : digits
  if (!last7) return res.status(404).json({ error: 'invalid address' })

  const photoPath = photoMap.get(last7)
  if (photoPath) {
    return res.sendFile(photoPath)
  }
  res.status(404).json({ error: 'no_photo' })
})

app.get('/contacts/:id', async (req, res) => {
  try {
    const id = safeJxaString(req.params.id)
    const script = `
      const Contacts = Application("Contacts");
      const p = Contacts.people.byId("${id}");
      JSON.stringify({
        id: p.id(), name: p.name(),
        firstName: p.firstName(), lastName: p.lastName(),
        organization: p.organization(),
        phones: p.phones().map(ph => ({label: ph.label(), value: ph.value()})),
        emails: p.emails().map(e => ({label: e.label(), value: e.value()})),
        addresses: p.addresses().map(a => ({
          label: a.label(), street: a.street(), city: a.city(),
          state: a.state(), zip: a.zip(), country: a.country()
        }))
      })
    `
    res.json(await jxa(script))
  } catch (err) { res.status(500).json({ error: safeError(err) }) }
})

// ═══════════════════════════════════════════════════════════════════
// FIND MY
// ═══════════════════════════════════════════════════════════════════

app.get('/findmy/devices', async (_req, res) => {
  try {
    // Read from Find My cache (requires Find My app to be running/synced)
    const cachePath = join(HOME, 'Library/Caches/com.apple.findmy.fmipcore/Items.data')
    const { readFile } = await import('fs/promises')
    const raw = await readFile(cachePath, 'utf-8')
    const items = JSON.parse(raw)
    const devices = items.map(d => ({
      id: d.identifier || d.serialNumber,
      name: d.name,
      model: d.productType?.type,
      battery: d.batteryLevel,
      batteryStatus: d.batteryStatus,
      location: d.location ? {
        lat: d.location.latitude,
        lng: d.location.longitude,
        accuracy: d.location.horizontalAccuracy,
        timestamp: d.location.timeStamp,
      } : null,
    }))
    res.json(devices)
  } catch (err) {
    // Fallback: try Devices.data
    try {
      const cachePath = join(HOME, 'Library/Caches/com.apple.findmy.fmipcore/Devices.data')
      const { readFile } = await import('fs/promises')
      const raw = await readFile(cachePath, 'utf-8')
      const items = JSON.parse(raw)
      const devices = items.map(d => ({
        id: d.baUUID || d.deviceDiscoveryId,
        name: d.name,
        model: d.deviceDisplayName,
        battery: d.batteryLevel,
        batteryStatus: d.batteryStatus,
        location: d.location ? {
          lat: d.location.latitude,
          lng: d.location.longitude,
          accuracy: d.location.horizontalAccuracy,
          timestamp: d.location.timeStamp,
        } : null,
      }))
      res.json(devices)
    } catch (err2) {
      res.status(500).json({ error: 'Find My cache not available. Open Find My app on this Mac first.' })
    }
  }
})

// ── Messages — mark chat as read via sqlite3 on chat.db ────────────

app.post('/messages/mark-read', async (req, res) => {
  const { chatGuid } = req.body
  if (!chatGuid || typeof chatGuid !== 'string') {
    return res.status(400).json({ error: 'chatGuid required' })
  }
  // Validate chatGuid matches iMessage GUID format: "iMessage;-;+1234567890" or "SMS;-;addr"
  // Strict format prevents SQL injection (no quotes, backslashes, parens, or whitespace)
  if (!/^(iMessage|SMS);[\-+];[a-zA-Z0-9_+\-@.]+$/.test(chatGuid) || chatGuid.length > 200) {
    return res.status(400).json({ error: 'Invalid chatGuid format' })
  }

  try {
    const { execFile: execFileCb } = await import('child_process')
    const { promisify } = await import('util')
    const execFileAsync = promisify(execFileCb)
    const dbPath = join(HOME, 'Library/Messages/chat.db')

    // date_read uses Apple Core Data nanoseconds since 2001-01-01
    // 978307200 = seconds between Unix epoch (1970) and Apple epoch (2001)
    const appleNow = (Math.floor(Date.now() / 1000) - 978307200) * 1000000000
    // chatGuid is already validated by strict regex above (safe chars only)
    const sql = `UPDATE message SET date_read = ${appleNow}
      WHERE ROWID IN (
        SELECT m.ROWID FROM message m
        JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
        JOIN chat c ON c.ROWID = cmj.chat_id
        WHERE c.guid = '${chatGuid}'
        AND m.is_from_me = 0
        AND m.date_read = 0
      );`

    await execFileAsync('sqlite3', [dbPath, sql])
    res.json({ ok: true })
  } catch (err) {
    console.error('mark-read error:', safeError(err))
    res.status(500).json({ error: safeError(err) })
  }
})

// ── Messages — serve raw attachment file by BB GUID (for HEIC/HEICS conversion) ──

app.get('/messages/attachment-raw', async (req, res) => {
  const guid = req.query.guid
  const originalName = req.query.name // original filename without .jpeg suffix
  if (!guid || typeof guid !== 'string') {
    return res.status(400).json({ error: 'guid required' })
  }
  // Validate guid format (e.g. at_0_UUID or at_UUID_UUID)
  if (!/^at_[a-zA-Z0-9_\-]+$/.test(guid)) {
    return res.status(400).json({ error: 'invalid guid' })
  }

  const attachDir = join(HOME, 'Library/Messages/Attachments')
  try {
    const { execFile: execFileCb } = await import('child_process')
    const { promisify: prom } = await import('util')
    const execAsync = prom(execFileCb)

    // Try to find the attachment file. macOS stores them at:
    // ~/Library/Messages/Attachments/XX/YY/<dir>/<filename>
    // The directory name may match the BB GUID or just be a UUID.

    // Strategy 1: search for directory matching BB GUID
    let foundFile = null
    const { stdout } = await execAsync('find', [attachDir, '-type', 'd', '-name', guid, '-maxdepth', '4'], { timeout: 5000 }).catch(() => ({ stdout: '' }))
    const dirs = stdout.trim().split('\n').filter(Boolean)

    if (dirs.length > 0) {
      const { readdir } = await import('fs/promises')
      const files = await readdir(dirs[0])
      const target = (originalName && files.includes(originalName)) ? originalName
        : files.find(f => /\.(heics|heic|apng|webp|gif|png)$/i.test(f)) || files[0]
      if (target) foundFile = join(dirs[0], target)
    }

    // Strategy 2: search by original filename directly
    if (!foundFile && originalName) {
      // Sanitize originalName to prevent path traversal in find
      const safeName = String(originalName).replace(/[^a-zA-Z0-9._\-]/g, '_').slice(0, 200)
      const { stdout: stdout2 } = await execAsync('find', [attachDir, '-type', 'f', '-name', safeName, '-maxdepth', '5'], { timeout: 5000 }).catch(() => ({ stdout: '' }))
      const files2 = stdout2.trim().split('\n').filter(Boolean)
      if (files2.length > 0) foundFile = files2[0]
    }

    if (!foundFile) return res.status(404).json({ error: 'attachment not found' })

    // Security: verify still under Attachments
    const resolved2 = resolve(foundFile)
    if (!resolved2.startsWith(attachDir + '/')) {
      return res.status(403).json({ error: 'path not allowed' })
    }

    const { readFile } = await import('fs/promises')
    const ext2 = extname(foundFile).toLowerCase()

    // HEIC/HEICS: convert to PNG using macOS sips (preserves alpha/transparency)
    if (ext2 === '.heic' || ext2 === '.heics') {
      // Use private temp dir with random suffix to prevent symlink attacks
      const tmpPng = join(BRIDGE_TMP, `sticker-${randomBytes(8).toString('hex')}.png`)
      try {
        await execAsync('sips', ['-s', 'format', 'png', resolved2, '--out', tmpPng], { timeout: 10000 })
        const pngData = await readFile(tmpPng)
        // Clean up temp file
        import('fs/promises').then(fs => fs.unlink(tmpPng).catch(() => {}))
        res.set('Content-Type', 'image/png')
        return res.send(pngData)
      } catch {
        // sips failed, fall through to raw file
      }
    }

    const data = await readFile(resolved2)
    const types = {
      '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.png': 'image/png',
      '.gif': 'image/gif', '.webp': 'image/webp', '.apng': 'image/apng',
      '.mp4': 'video/mp4', '.mov': 'video/quicktime', '.caf': 'audio/x-caf',
    }
    res.set('Content-Type', types[ext2] || 'application/octet-stream')
    res.send(data)
  } catch {
    res.status(404).json({ error: 'attachment not found' })
  }
})

// ── Start ───────────────────────────────────────────────────────────

// ── Global error handler (prevents Express from leaking stack traces) ──
app.use((err, _req, res, _next) => {
  console.error('Unhandled error:', safeError(err))
  res.status(500).json({ error: 'internal server error' })
})

const server = app.listen(PORT, '127.0.0.1', () => {
  console.log(`mac-bridge listening on 127.0.0.1:${PORT}`)
  console.log('Services: reminders, notes, contacts, messages, findmy')
})
// Prevent slowloris DoS — headersTimeout must be > keepAliveTimeout per Node.js docs
server.keepAliveTimeout = 30000
server.headersTimeout = 35000
