#![forbid(unsafe_code)]

use anyhow::{anyhow, Result};
use codex_harness::{append_ledger, state_dir};
use rusqlite::Connection;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

const TABLES: &[&str] = &[
    "ledger_index",
    "agents",
    "teams",
    "tasks",
    "model_routes",
    "process_registry",
    "timers",
    "rule_breaks",
    "policy_breaks",
    "yolo_attempts",
    "network_grants",
    "github_actions",
    "browser_computer_actions",
    "memory_events",
    "plugin_events",
    "mcp_events",
    "open_decisions",
    "archives",
    "budgets",
];

fn db_path() -> PathBuf {
    state_dir().join("harness.sqlite3")
}

fn schema_sql() -> String {
    let mut sql = String::from(
        r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
CREATE TABLE IF NOT EXISTS ledger_index (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ledger TEXT NOT NULL,
  sequence INTEGER,
  ts_utc TEXT,
  event TEXT,
  session_id TEXT,
  parent_id TEXT,
  agent_id TEXT,
  team_id TEXT,
  task_id TEXT,
  cwd TEXT,
  command_hash TEXT,
  redacted_command_preview TEXT,
  decision TEXT,
  reason TEXT,
  previous_hash TEXT,
  line_hash TEXT,
  source_line INTEGER,
  indexed_at_utc TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
"#,
    );
    for table in TABLES
        .iter()
        .copied()
        .filter(|name| *name != "ledger_index")
    {
        sql.push_str(&format!(
            r#"
CREATE TABLE IF NOT EXISTS {table} (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  external_id TEXT,
  ts_utc TEXT DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
  state TEXT,
  decision TEXT,
  reason TEXT,
  redacted_preview TEXT,
  payload_json TEXT
);
"#
        ));
    }
    sql.push_str("PRAGMA user_version=1;\n");
    sql
}

fn init_db() -> Result<serde_json::Value> {
    fs::create_dir_all(state_dir())?;
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute_batch(&schema_sql())?;
    let value = json!({
        "ok": true,
        "action": "init",
        "db_path": path,
        "driver": "rusqlite",
        "tables_required": TABLES,
        "secret_printed": false
    });
    append_ledger(
        "index.jsonl",
        json!({"event":"sqlite_init","decision":"allow","result":value}),
    )?;
    Ok(value)
}

fn integrity_check() -> Result<serde_json::Value> {
    if !db_path().exists() {
        init_db()?;
    }
    let path = db_path();
    let conn = Connection::open(&path)?;
    let integrity = conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))?;
    let mut statement =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
    let present = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let missing = TABLES
        .iter()
        .filter(|name| !present.iter().any(|present_name| present_name == **name))
        .copied()
        .collect::<Vec<_>>();
    let ok = integrity == "ok" && missing.is_empty();
    let value = json!({
        "ok": ok,
        "action": "integrity",
        "db_path": path,
        "driver": "rusqlite",
        "integrity_check": integrity,
        "tables_present": present,
        "missing_tables": missing,
        "secret_printed": false
    });
    append_ledger(
        "index.jsonl",
        json!({"event":"sqlite_integrity","decision": if ok {"allow"} else {"deny"},"result":value}),
    )?;
    Ok(value)
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(cmd) = args.first().map(String::as_str) else {
        eprintln!("usage: codex-harness-db <init|integrity>");
        std::process::exit(2);
    };
    let value = match cmd {
        "init" => init_db()?,
        "integrity" | "check" => integrity_check()?,
        other => return Err(anyhow!("unknown db command {other}")),
    };
    let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    println!("{}", serde_json::to_string_pretty(&value)?);
    if !ok {
        std::process::exit(1);
    }
    Ok(())
}
