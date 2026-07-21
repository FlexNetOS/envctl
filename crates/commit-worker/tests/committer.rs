#![cfg(feature = "pg-integration")]

//! ARCHBP-042 red tests: envctl is the exclusive production committer.
//! Grants deny every non-envctl write at the database; the drain worker
//! commits ordered staging records durably with exact commit identity and
//! a chained witness; the reconciliation cursor advances atomically with
//! the commit (never before); replay is idempotent across restarts; and
//! committed state projects back deterministically through the redb
//! owner's versioned UDS protocol.
//!
//! Every test uses `ENVCTL_PG_CONN` explicitly against a disposable
//! PostgreSQL service — never a production database.

use envctl_commit_worker::{
    COMMITTED_TABLE, COMMITTER_ROLE, CURSOR_TABLE, OWNER_PROTOCOL_VERSION, STAGING_TABLE,
    apply_role_and_grant_policy, committed_records, drain_and_commit, reconciliation_cursor,
    return_projection, verify_witness_chain,
};
use postgres::{Client, NoTls};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

static PG_LOCK: Mutex<()> = Mutex::new(());

fn disposable_conn() -> String {
    std::env::var("ENVCTL_PG_CONN")
        .expect("ENVCTL_PG_CONN must select the explicit disposable PostgreSQL test service")
}

fn admin(conn: &str) -> Client {
    Client::connect(conn, NoTls).expect("connect admin")
}

fn reset(client: &mut Client) {
    client
        .batch_execute(&format!(
            "DROP TABLE IF EXISTS {COMMITTED_TABLE};\
             DROP TABLE IF EXISTS {CURSOR_TABLE};\
             DROP TABLE IF EXISTS {STAGING_TABLE};"
        ))
        .expect("reset tables");
}

fn seed_staging(client: &mut Client, seqs: &[i64]) {
    client
        .batch_execute(&format!(
            "CREATE TABLE IF NOT EXISTS {STAGING_TABLE} (\
                 seq BIGINT PRIMARY KEY,\
                 contract_version TEXT NOT NULL,\
                 blob_sha256 TEXT NOT NULL,\
                 job JSONB NOT NULL,\
                 synced_at TIMESTAMPTZ NOT NULL DEFAULT now()\
             )"
        ))
        .expect("staging table");
    for seq in seqs {
        client
            .execute(
                &format!(
                    "INSERT INTO {STAGING_TABLE} (seq, contract_version, blob_sha256, job) \
                     VALUES ($1, 'codedb.outbox-export.v0', $2, $3::text::jsonb) \
                     ON CONFLICT (seq) DO NOTHING"
                ),
                &[
                    seq,
                    &format!("{seq:064x}"),
                    &format!("{{\"seq\":{seq},\"model_name\":\"m\"}}"),
                ],
            )
            .expect("seed staging row");
    }
}

#[test]
fn grants_deny_every_non_envctl_write_at_the_database() {
    let _guard = PG_LOCK.lock().expect("pg lock");
    let conn = disposable_conn();
    let mut client = admin(&conn);
    reset(&mut client);
    apply_role_and_grant_policy(&conn).expect("policy applies");

    // An intruder role (no envctl membership) is denied by PostgreSQL.
    client
        .batch_execute(
            "DO $$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='archbp042_intruder')\
             THEN CREATE ROLE archbp042_intruder NOLOGIN; END IF; END $$;",
        )
        .expect("intruder role");
    client
        .batch_execute("GRANT USAGE ON SCHEMA lifeos_runtime TO archbp042_intruder")
        .expect("usage only");
    let denied = client.batch_execute(&format!(
        "SET ROLE archbp042_intruder;\
         INSERT INTO {COMMITTED_TABLE} (seq, blob_sha256, contract_version, job,\
             commit_txid, commit_lsn, generation, witness)\
         VALUES (999, 'x', 'v', '{{}}'::jsonb, 't', 'l', 1, 'w');"
    ));
    client.batch_execute("RESET ROLE").expect("reset role");
    let error = denied.expect_err("non-envctl write must be denied");
    assert!(
        error.to_string().contains("permission denied"),
        "denial must come from PostgreSQL grants, got: {error}"
    );

    // The committer role itself is allowed.
    seed_staging(&mut client, &[1]);
    let receipt = drain_and_commit(&conn, 16, false).expect("committer drains");
    assert_eq!(receipt.committed, vec![1]);
}

#[test]
fn drain_is_ordered_idempotent_and_restart_safe_with_exact_identity() {
    let _guard = PG_LOCK.lock().expect("pg lock");
    let conn = disposable_conn();
    let mut client = admin(&conn);
    reset(&mut client);
    apply_role_and_grant_policy(&conn).expect("policy applies");
    seed_staging(&mut client, &[1, 2, 3, 4, 5]);

    // Batched drain commits everything in order.
    let receipt = drain_and_commit(&conn, 2, false).expect("drain");
    assert_eq!(receipt.committed, vec![1, 2, 3, 4, 5]);
    assert_eq!(receipt.acknowledged_seq, 5);

    // Replay after restart: nothing commits twice.
    let replay = drain_and_commit(&conn, 2, false).expect("replay");
    assert_eq!(replay.committed, Vec::<i64>::new());
    assert_eq!(replay.acknowledged_seq, 5);

    // New staging rows drain from the cursor, not from zero.
    seed_staging(&mut client, &[6, 7]);
    let tail = drain_and_commit(&conn, 16, false).expect("tail drain");
    assert_eq!(tail.committed, vec![6, 7]);

    // Every committed record carries its exact commit identity.
    let records = committed_records(&conn).expect("read back");
    assert_eq!(
        records.iter().map(|r| r.seq).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 6, 7]
    );
    for record in &records {
        assert!(!record.commit_txid.is_empty(), "txid missing on {}", record.seq);
        assert!(!record.commit_lsn.is_empty(), "lsn missing on {}", record.seq);
        assert!(record.generation >= 1);
        assert_eq!(record.witness.len(), 64, "witness must be a sha256 digest");
    }
    // Rows in one batch share a transaction; batches differ.
    assert_eq!(records[0].commit_txid, records[1].commit_txid);
    assert_ne!(records[1].commit_txid, records[2].commit_txid);

    // The witness chain verifies end to end.
    verify_witness_chain(&records).expect("witness chain holds");
}

#[test]
fn acknowledgement_never_precedes_durable_commit() {
    let _guard = PG_LOCK.lock().expect("pg lock");
    let conn = disposable_conn();
    let mut client = admin(&conn);
    reset(&mut client);
    apply_role_and_grant_policy(&conn).expect("policy applies");
    seed_staging(&mut client, &[1, 2]);

    // The failpoint aborts the transaction after all writes, before COMMIT.
    let failed = drain_and_commit(&conn, 16, true);
    assert!(failed.is_err(), "the aborted commit surfaces an error");
    // Nothing is visible and nothing is acknowledged.
    assert!(committed_records(&conn).expect("read back").is_empty());
    let cursor = reconciliation_cursor(&conn).expect("cursor");
    assert_eq!(cursor.acknowledged_seq, 0, "no acknowledgement before durability");

    // The retry commits exactly once.
    let receipt = drain_and_commit(&conn, 16, false).expect("retry");
    assert_eq!(receipt.committed, vec![1, 2]);
    let cursor = reconciliation_cursor(&conn).expect("cursor after retry");
    assert_eq!(cursor.acknowledged_seq, 2);
    assert!(!cursor.last_witness.is_empty());
}

#[test]
fn committed_state_projects_back_deterministically_through_the_owner_protocol() {
    let _guard = PG_LOCK.lock().expect("pg lock");
    let conn = disposable_conn();
    let mut client = admin(&conn);
    reset(&mut client);
    apply_role_and_grant_policy(&conn).expect("policy applies");
    seed_staging(&mut client, &[1, 2, 3]);
    drain_and_commit(&conn, 16, false).expect("drain");

    // A minimal in-test owner speaking the exact versioned protocol records
    // every authenticated put.
    let root = std::env::temp_dir().join(format!(
        "envctl-return-projection-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("owner root");
    std::fs::write(root.join("owner.token"), "test-owner-token").expect("token");
    let socket_path = root.join("owner.sock");
    std::fs::remove_file(&socket_path).ok();
    let listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind");
    let recorded = std::sync::Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let record_sink = std::sync::Arc::clone(&recorded);
    let server = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { break };
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut writer = stream;
            let mut line = String::new();
            while reader.read_line(&mut line).map(|n| n > 0).unwrap_or(false) {
                let request: serde_json::Value =
                    serde_json::from_str(line.trim()).expect("request json");
                assert_eq!(request["protocol_version"], OWNER_PROTOCOL_VERSION);
                assert_eq!(request["token"], "test-owner-token");
                if request["op"] == "put" {
                    record_sink.lock().expect("sink").push((
                        request["key"].as_str().expect("key").to_string(),
                        request["value"].as_str().expect("value").to_string(),
                    ));
                }
                writeln!(writer, "{}", serde_json::json!({"ok": true, "seq": 1}))
                    .expect("respond");
                line.clear();
            }
            break;
        }
    });

    let first = return_projection(&conn, &root).expect("projection");
    server.join().expect("owner server");
    let recorded_puts = recorded.lock().expect("sink").clone();
    assert_eq!(recorded_puts, first, "the projection is what actually reached the owner");
    assert!(
        first
            .iter()
            .any(|(k, _)| k == "envctl/return-projection/acknowledged_seq"),
        "projection must carry the acknowledged sequence"
    );

    // Determinism: identical committed state projects identical pairs.
    let second_root = std::env::temp_dir().join(format!(
        "envctl-return-projection-b-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&second_root).expect("owner root b");
    std::fs::write(second_root.join("owner.token"), "test-owner-token").expect("token b");
    let socket_b = second_root.join("owner.sock");
    std::fs::remove_file(&socket_b).ok();
    let listener_b = std::os::unix::net::UnixListener::bind(&socket_b).expect("bind b");
    let server_b = std::thread::spawn(move || {
        for stream in listener_b.incoming() {
            let Ok(stream) = stream else { break };
            let mut reader = BufReader::new(stream.try_clone().expect("clone"));
            let mut writer = stream;
            let mut line = String::new();
            while reader.read_line(&mut line).map(|n| n > 0).unwrap_or(false) {
                writeln!(writer, "{}", serde_json::json!({"ok": true, "seq": 1}))
                    .expect("respond");
                line.clear();
            }
            break;
        }
    });
    let second = return_projection(&conn, &second_root).expect("projection again");
    server_b.join().expect("owner server b");
    assert_eq!(first, second, "identical committed state must project identically");

    std::fs::remove_dir_all(&root).ok();
    std::fs::remove_dir_all(&second_root).ok();

    // The committer role string is part of the public contract.
    assert_eq!(COMMITTER_ROLE, "lifeos_envctl");
}
