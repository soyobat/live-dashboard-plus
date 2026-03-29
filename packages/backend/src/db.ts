import { createClient, type Client } from "@libsql/client";

// Turso database configuration
const tursoUrl = process.env.TURSO_DATABASE_URL;
const tursoToken = process.env.TURSO_AUTH_TOKEN;

let db: Client;

if (tursoUrl && tursoToken) {
  // Production: Use Turso (libSQL over HTTP)
  console.log("[db] Connecting to Turso database:", tursoUrl.replace(/\/\/([^\/]+)\./, '//***.$1'));
  db = createClient({
    url: tursoUrl,
    authToken: tursoToken,
  });
} else {
  // Development: Fallback to local SQLite file
  console.warn("[db] TURSO_DATABASE_URL not set, using local SQLite (data will be lost on restart)");
  const { Database } = require("bun:sqlite") as typeof import("bun:sqlite");
  const DB_PATH = process.env.DB_PATH || "./live-dashboard.db";
  const localDb = new Database(DB_PATH, { create: true });
  
  // Wrap local SQLite to match libSQL API
  db = {
    async execute(sql: string, args?: any[]) {
      const stmt = localDb.prepare(sql);
      if (args) {
        stmt.bind(...args);
      }
      const results = stmt.all();
      return { rows: results, rowsAffected: 0 };
    },
    async transaction(fn: () => Promise<void>) {
      localDb.run("BEGIN TRANSACTION");
      try {
        await fn();
        localDb.run("COMMIT");
      } catch (e) {
        localDb.run("ROLLBACK");
        throw e;
      }
    },
    close() {
      localDb.close();
    }
  } as Client;
}

// Activities table
await db.execute(`
  CREATE TABLE IF NOT EXISTS activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL,
    device_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    app_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    window_title TEXT DEFAULT '',
    title_hash TEXT NOT NULL DEFAULT '',
    time_bucket INTEGER NOT NULL DEFAULT 0,
    started_at TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
  )
`);

// Dedup unique constraint
await db.execute(`
  CREATE UNIQUE INDEX IF NOT EXISTS idx_dedup
  ON activities(device_id, app_id, title_hash, time_bucket)
`);

// Query indexes
await db.execute(`
  CREATE INDEX IF NOT EXISTS idx_activities_device_started
  ON activities(device_id, started_at DESC)
`);
await db.execute(`
  CREATE INDEX IF NOT EXISTS idx_activities_started
  ON activities(started_at DESC)
`);
await db.execute(`
  CREATE INDEX IF NOT EXISTS idx_activities_created
  ON activities(created_at)
`);

// Device states table
await db.execute(`
  CREATE TABLE IF NOT EXISTS device_states (
    device_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    app_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    window_title TEXT DEFAULT '',
    last_seen_at TEXT NOT NULL,
    is_online INTEGER DEFAULT 1
  )
`);

// ── Schema migration: add display_title + extra columns ──

const KNOWN_TABLES = new Set(["activities", "device_states"]);

async function columnExists(table: string, column: string): Promise<boolean> {
  if (!KNOWN_TABLES.has(table)) {
    throw new Error(`columnExists: unknown table "${table}"`);
  }
  const rows = await db.execute(`PRAGMA table_info(${table})`);
  return rows.rows.some((r: any) => r.name === column);
}

// activities.display_title
if (!await columnExists("activities", "display_title")) {
  await db.execute("ALTER TABLE activities ADD COLUMN display_title TEXT DEFAULT ''");
}

// device_states.display_title
if (!await columnExists("device_states", "display_title")) {
  await db.execute("ALTER TABLE device_states ADD COLUMN display_title TEXT DEFAULT ''");
}

// device_states.extra (JSON string for battery, etc.)
if (!await columnExists("device_states", "extra")) {
  await db.execute("ALTER TABLE device_states ADD COLUMN extra TEXT DEFAULT '{}'");
}

// ── Health records table ──

await db.execute(`
  CREATE TABLE IF NOT EXISTS health_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL,
    type TEXT NOT NULL,
    value REAL NOT NULL,
    unit TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    end_time TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(device_id, type, recorded_at, end_time)
  )
`);

await db.execute(`
  CREATE INDEX IF NOT EXISTS idx_health_records_recorded
  ON health_records(recorded_at)
`);

await db.execute(`
  CREATE INDEX IF NOT EXISTS idx_health_records_type
  ON health_records(type, recorded_at)
`);

// ── HMAC hash secret validation ──

const HASH_SECRET = process.env.HASH_SECRET || "";
if (!HASH_SECRET) {
  console.error("[db] FATAL: HASH_SECRET not set. This is required for privacy-safe title hashing.");
  console.error("[db] Generate one with: openssl rand -hex 32");
  process.exit(1);
}

export function hmacTitle(title: string): string {
  const hmac = new Bun.CryptoHasher("sha256", HASH_SECRET);
  hmac.update(title);
  return hmac.digest("hex");
}

// Database operations (async for libSQL compatibility)
export async function insertActivity(
  device_id: string,
  device_name: string,
  platform: string,
  app_id: string,
  app_name: string,
  window_title: string,
  display_title: string,
  title_hash: string,
  time_bucket: number,
  started_at: string
) {
  const result = await db.execute({
    sql: `
      INSERT INTO activities (device_id, device_name, platform, app_id, app_name, window_title, display_title, title_hash, time_bucket, started_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      ON CONFLICT(device_id, app_id, title_hash, time_bucket) DO NOTHING
    `,
    args: [device_id, device_name, platform, app_id, app_name, window_title, display_title, title_hash, time_bucket, started_at]
  });
  return result;
}

export async function upsertDeviceState(
  device_id: string,
  device_name: string,
  platform: string,
  app_id: string,
  app_name: string,
  window_title: string,
  display_title: string,
  last_seen_at: string,
  extra: string
) {
  const result = await db.execute({
    sql: `
      INSERT INTO device_states (device_id, device_name, platform, app_id, app_name, window_title, display_title, last_seen_at, extra, is_online)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
      ON CONFLICT(device_id) DO UPDATE SET
        device_name = excluded.device_name,
        platform = excluded.platform,
        app_id = excluded.app_id,
        app_name = excluded.app_name,
        window_title = excluded.window_title,
        display_title = excluded.display_title,
        last_seen_at = excluded.last_seen_at,
        extra = excluded.extra,
        is_online = 1
    `,
    args: [device_id, device_name, platform, app_id, app_name, window_title, display_title, last_seen_at, extra]
  });
  return result;
}

export async function getAllDeviceStates() {
  const result = await db.execute(`
    SELECT * FROM device_states ORDER BY last_seen_at DESC
  `);
  return result.rows;
}

export async function getRecentActivities() {
  const result = await db.execute(`
    SELECT * FROM activities ORDER BY started_at DESC LIMIT 20
  `);
  return result.rows;
}

export async function getTimelineByDate(date: string) {
  const result = await db.execute({
    sql: `
      SELECT * FROM activities
      WHERE date(started_at) = ?
      ORDER BY started_at ASC
    `,
    args: [date]
  });
  return result.rows;
}

export async function getTimelineByDateAndDevice(date: string, device_id: string) {
  const result = await db.execute({
    sql: `
      SELECT * FROM activities
      WHERE date(started_at) = ? AND device_id = ?
      ORDER BY started_at ASC
    `,
    args: [date, device_id]
  });
  return result.rows;
}

export async function markOfflineDevices() {
  const result = await db.execute(`
    UPDATE device_states SET is_online = 0
    WHERE is_online = 1
    AND (last_seen_at IS NULL OR last_seen_at = '' OR datetime(last_seen_at) IS NULL
         OR datetime(last_seen_at) < datetime('now', '-1 minute'))
  `);
  return result;
}

export async function cleanupOldActivities() {
  const result = await db.execute(`
    DELETE FROM activities WHERE created_at < datetime('now', '-7 days')
  `);
  return result;
}

export default db;
export { db };
