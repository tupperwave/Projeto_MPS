//! SQLiteRepositories: persistência local em arquivo único (hosts, auditoria, workflows).
use crate::domain::*;
use crate::error::{NhError, Res};
use crate::ports::*;
use crate::util::{ip_in_target, now_iso, parse_target, sha256_hex};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS hosts(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  ip TEXT UNIQUE NOT NULL, hostname TEXT, mac TEXT, os TEXT,
  status TEXT NOT NULL DEFAULT 'unknown',
  ssh_user TEXT, ssh_port INTEGER, ssh_identity TEXT,
  facts TEXT NOT NULL DEFAULT '{}',
  first_seen TEXT NOT NULL, last_seen TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS ports(
  host_id INTEGER NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
  port INTEGER NOT NULL, protocol TEXT NOT NULL, state TEXT NOT NULL, service TEXT NOT NULL, product TEXT NOT NULL,
  PRIMARY KEY(host_id, port, protocol));
CREATE TABLE IF NOT EXISTS capabilities(
  host_id INTEGER NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
  name TEXT NOT NULL, state TEXT NOT NULL, detail TEXT NOT NULL, verified_at TEXT,
  PRIMARY KEY(host_id, name));
CREATE TABLE IF NOT EXISTS host_tags(
  host_id INTEGER NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
  tag TEXT NOT NULL, PRIMARY KEY(host_id, tag));
CREATE TABLE IF NOT EXISTS audit(
  id INTEGER PRIMARY KEY, ts TEXT NOT NULL, actor TEXT NOT NULL,
  host_id INTEGER, host_label TEXT NOT NULL, operation_id TEXT NOT NULL, risk TEXT NOT NULL,
  command TEXT NOT NULL, status TEXT NOT NULL, exit_code INTEGER, duration_ms INTEGER NOT NULL,
  stdout TEXT NOT NULL, stdout_sha256 TEXT NOT NULL, stderr TEXT NOT NULL, stderr_sha256 TEXT NOT NULL,
  truncated INTEGER NOT NULL, evidence TEXT NOT NULL, prev_hash TEXT NOT NULL, hash TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS idx_audit_host ON audit(host_id);
CREATE INDEX IF NOT EXISTS idx_audit_op ON audit(operation_id);
CREATE TABLE IF NOT EXISTS workflows(
  id TEXT PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL, definition TEXT NOT NULL);
"#;

pub struct SQLiteRepositories {
    conn: Mutex<Connection>,
}

#[derive(Serialize)]
struct Canon<'a> {
    id: i64,
    ts: &'a str,
    actor: &'a str,
    host_id: Option<i64>,
    host_label: &'a str,
    operation_id: &'a str,
    risk: &'a str,
    command: &'a str,
    status: &'a str,
    exit_code: Option<i32>,
    duration_ms: i64,
    stdout_sha256: &'a str,
    stderr_sha256: &'a str,
    evidence: &'a str,
    prev_hash: &'a str,
}

fn chain_hash(c: &Canon) -> String {
    let body = serde_json::to_string(c).unwrap_or_default();
    sha256_hex(format!("{}|{}", c.prev_hash, body).as_bytes())
}

impl SQLiteRepositories {
    pub fn open(path: &Path) -> Res<Self> {
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    // o banco guarda inventário e trilha de auditoria: restrito ao dono
                    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
                }
            }
        }
        let conn = Connection::open(path)?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Res<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Res<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(SCHEMA)?;
        Ok(SQLiteRepositories { conn: Mutex::new(conn) })
    }

    pub fn seed_workflows(&self, wfs: &[Workflow]) -> Res<()> {
        let conn = self.conn.lock().unwrap();
        for w in wfs {
            conn.execute(
                "INSERT INTO workflows(id,name,description,definition) VALUES(?1,?2,?3,?4)
                 ON CONFLICT(id) DO UPDATE SET name=excluded.name, description=excluded.description, definition=excluded.definition",
                params![w.id, w.name, w.description, serde_json::to_string(w)?],
            )?;
        }
        Ok(())
    }

    fn load_host(conn: &Connection, id: i64) -> Res<Option<Host>> {
        let base = conn
            .query_row(
                "SELECT id,ip,hostname,mac,os,status,ssh_user,ssh_port,ssh_identity,facts,first_seen,last_seen FROM hosts WHERE id=?1",
                params![id],
                |r| {
                    Ok(Host {
                        id: r.get(0)?,
                        ip: r.get(1)?,
                        hostname: r.get(2)?,
                        mac: r.get(3)?,
                        os: r.get(4)?,
                        status: r.get(5)?,
                        ssh_user: r.get(6)?,
                        ssh_port: r.get::<_, Option<i64>>(7)?.map(|p| p as u16),
                        ssh_identity: r.get(8)?,
                        facts: serde_json::from_str::<BTreeMap<String, String>>(&r.get::<_, String>(9)?).unwrap_or_default(),
                        first_seen: r.get(10)?,
                        last_seen: r.get(11)?,
                        ..Default::default()
                    })
                },
            )
            .optional()?;
        let Some(mut h) = base else { return Ok(None) };
        let mut st = conn.prepare("SELECT port,protocol,state,service,product FROM ports WHERE host_id=?1 ORDER BY port")?;
        h.ports = st
            .query_map(params![id], |r| {
                Ok(NetworkPort {
                    port: r.get::<_, i64>(0)? as u16,
                    protocol: r.get(1)?,
                    state: r.get(2)?,
                    service: r.get(3)?,
                    product: r.get(4)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        let mut st = conn.prepare("SELECT name,state,detail,verified_at FROM capabilities WHERE host_id=?1 ORDER BY name")?;
        h.capabilities = st
            .query_map(params![id], |r| {
                Ok(Capability {
                    name: r.get(0)?,
                    state: CapabilityState::parse(&r.get::<_, String>(1)?),
                    detail: r.get(2)?,
                    verified_at: r.get(3)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        let mut st = conn.prepare("SELECT tag FROM host_tags WHERE host_id=?1 ORDER BY tag")?;
        h.tags = st.query_map(params![id], |r| r.get(0))?.collect::<Result<_, _>>()?;
        Ok(Some(h))
    }
}

fn row_to_audit(r: &rusqlite::Row) -> rusqlite::Result<AuditRecord> {
    Ok(AuditRecord {
        id: r.get(0)?,
        ts: r.get(1)?,
        actor: r.get(2)?,
        host_id: r.get(3)?,
        host_label: r.get(4)?,
        operation_id: r.get(5)?,
        risk: RiskLevel::parse(&r.get::<_, String>(6)?).unwrap_or(RiskLevel::Low),
        command: r.get(7)?,
        status: OperationStatus::parse(&r.get::<_, String>(8)?).unwrap_or(OperationStatus::Failed),
        exit_code: r.get(9)?,
        duration_ms: r.get(10)?,
        stdout: r.get(11)?,
        stdout_sha256: r.get(12)?,
        stderr: r.get(13)?,
        stderr_sha256: r.get(14)?,
        truncated: r.get::<_, i64>(15)? != 0,
        evidence: r.get(16)?,
        prev_hash: r.get(17)?,
        hash: r.get(18)?,
    })
}

const AUDIT_COLS: &str =
    "id,ts,actor,host_id,host_label,operation_id,risk,command,status,exit_code,duration_ms,stdout,stdout_sha256,stderr,stderr_sha256,truncated,evidence,prev_hash,hash";

impl IHostRepository for SQLiteRepositories {
    fn upsert_discovered(&self, hosts: &[Host], target: &str) -> Res<Vec<i64>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = now_iso();
        let mut ids = vec![];
        for h in hosts {
            tx.execute(
                "INSERT INTO hosts(ip,hostname,mac,status,first_seen,last_seen) VALUES(?1,?2,?3,'up',?4,?4)
                 ON CONFLICT(ip) DO UPDATE SET
                   hostname=COALESCE(excluded.hostname, hosts.hostname),
                   mac=COALESCE(excluded.mac, hosts.mac),
                   status='up', last_seen=excluded.last_seen",
                params![h.ip, h.hostname, h.mac, now],
            )?;
            let id: i64 = tx.query_row("SELECT id FROM hosts WHERE ip=?1", params![h.ip], |r| r.get(0))?;
            tx.execute("DELETE FROM ports WHERE host_id=?1", params![id])?;
            for p in &h.ports {
                tx.execute(
                    "INSERT OR REPLACE INTO ports(host_id,port,protocol,state,service,product) VALUES(?1,?2,?3,?4,?5,?6)",
                    params![id, p.port as i64, p.protocol, p.state, p.service, p.product],
                )?;
            }
            ids.push(id);
        }
        // hosts do alvo que não responderam agora passam a "down" (mantém histórico e last_seen)
        if let Ok(tokens) = parse_target(target) {
            let all: Vec<(i64, String)> = {
                let mut st = tx.prepare("SELECT id, ip FROM hosts")?;
                let rows = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
                rows.collect::<Result<_, _>>()?
            };
            for (id, ip) in all {
                if !ids.contains(&id) && ip_in_target(&ip, &tokens) == Some(true) {
                    tx.execute("UPDATE hosts SET status='down' WHERE id=?1", params![id])?;
                }
            }
        }
        tx.commit()?;
        Ok(ids)
    }

    fn insert_manual(&self, ip: &str, hostname: Option<&str>) -> Res<Host> {
        let id = {
            let conn = self.conn.lock().unwrap();
            let now = now_iso();
            conn.execute(
                "INSERT INTO hosts(ip,hostname,status,first_seen,last_seen) VALUES(?1,?2,'unknown',?3,?3)
                 ON CONFLICT(ip) DO UPDATE SET hostname=COALESCE(excluded.hostname, hosts.hostname)",
                params![ip, hostname, now],
            )?;
            conn.query_row("SELECT id FROM hosts WHERE ip=?1", params![ip], |r| r.get::<_, i64>(0))?
        };
        self.get(id)?.ok_or_else(|| NhError::NotFound(ip.into()))
    }

    fn list(&self) -> Res<Vec<Host>> {
        let conn = self.conn.lock().unwrap();
        let ids: Vec<i64> = {
            let mut st = conn.prepare("SELECT id FROM hosts ORDER BY ip")?;
            let rows = st.query_map([], |r| r.get(0))?;
            rows.collect::<Result<_, _>>()?
        };
        let mut out = vec![];
        for id in ids {
            if let Some(h) = Self::load_host(&conn, id)? {
                out.push(h);
            }
        }
        out.sort_by_key(|h| h.ip.split('.').map(|o| o.parse::<u32>().unwrap_or(0)).collect::<Vec<_>>());
        Ok(out)
    }

    fn get(&self, id: i64) -> Res<Option<Host>> {
        let conn = self.conn.lock().unwrap();
        Self::load_host(&conn, id)
    }

    fn find(&self, key: &str) -> Res<Option<Host>> {
        let conn = self.conn.lock().unwrap();
        let id: Option<i64> = if let Ok(n) = key.parse::<i64>() {
            conn.query_row("SELECT id FROM hosts WHERE id=?1 OR ip=?2", params![n, key], |r| r.get(0)).optional()?
        } else {
            conn.query_row("SELECT id FROM hosts WHERE ip=?1 OR lower(hostname)=lower(?1) ORDER BY id LIMIT 1", params![key], |r| r.get(0))
                .optional()?
        };
        match id {
            Some(id) => Self::load_host(&conn, id),
            None => Ok(None),
        }
    }

    fn set_ssh(&self, id: i64, user: Option<&str>, port: Option<u16>, identity: Option<&str>) -> Res<()> {
        let conn = self.conn.lock().unwrap();
        if let Some(u) = user {
            conn.execute("UPDATE hosts SET ssh_user=?2 WHERE id=?1", params![id, u])?;
        }
        if let Some(p) = port {
            conn.execute("UPDATE hosts SET ssh_port=?2 WHERE id=?1", params![id, p as i64])?;
        }
        if let Some(k) = identity {
            conn.execute("UPDATE hosts SET ssh_identity=?2 WHERE id=?1", params![id, k])?;
        }
        Ok(())
    }

    fn save_capabilities(&self, id: i64, caps: &[Capability]) -> Res<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for c in caps {
            tx.execute(
                "INSERT INTO capabilities(host_id,name,state,detail,verified_at) VALUES(?1,?2,?3,?4,?5)
                 ON CONFLICT(host_id,name) DO UPDATE SET state=excluded.state, detail=excluded.detail, verified_at=excluded.verified_at",
                params![id, c.name, c.state.as_str(), c.detail, c.verified_at],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn set_identity(&self, id: i64, hostname: Option<&str>, os: Option<&str>, facts: &BTreeMap<String, String>) -> Res<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE hosts SET hostname=COALESCE(?2,hostname), os=COALESCE(?3,os), facts=?4 WHERE id=?1",
            params![id, hostname, os, serde_json::to_string(facts)?],
        )?;
        Ok(())
    }

    fn add_tag(&self, id: i64, tag: &str) -> Res<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO host_tags(host_id,tag) VALUES(?1,?2)", params![id, tag])?;
        Ok(())
    }

    fn remove_tag(&self, id: i64, tag: &str) -> Res<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM host_tags WHERE host_id=?1 AND tag=?2", params![id, tag])?;
        Ok(())
    }
}

impl IAuditRepository for SQLiteRepositories {
    fn append(&self, e: &NewAudit) -> Res<AuditRecord> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (prev_id, prev_hash): (i64, String) = tx
            .query_row("SELECT id, hash FROM audit ORDER BY id DESC LIMIT 1", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()?
            .unwrap_or((0, GENESIS.to_string()));
        let id = prev_id + 1;
        let canon = Canon {
            id,
            ts: &e.ts,
            actor: &e.actor,
            host_id: e.host_id,
            host_label: &e.host_label,
            operation_id: &e.operation_id,
            risk: e.risk.as_str(),
            command: &e.command,
            status: e.status.as_str(),
            exit_code: e.exit_code,
            duration_ms: e.duration_ms,
            stdout_sha256: &e.stdout_sha256,
            stderr_sha256: &e.stderr_sha256,
            evidence: &e.evidence,
            prev_hash: &prev_hash,
        };
        let hash = chain_hash(&canon);
        tx.execute(
            &format!("INSERT INTO audit({AUDIT_COLS}) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)"),
            params![
                id, e.ts, e.actor, e.host_id, e.host_label, e.operation_id, e.risk.as_str(), e.command,
                e.status.as_str(), e.exit_code, e.duration_ms, e.stdout, e.stdout_sha256, e.stderr,
                e.stderr_sha256, e.truncated as i64, e.evidence, prev_hash, hash
            ],
        )?;
        tx.commit()?;
        Ok(AuditRecord {
            id,
            ts: e.ts.clone(),
            actor: e.actor.clone(),
            host_id: e.host_id,
            host_label: e.host_label.clone(),
            operation_id: e.operation_id.clone(),
            risk: e.risk,
            command: e.command.clone(),
            status: e.status,
            exit_code: e.exit_code,
            duration_ms: e.duration_ms,
            stdout: e.stdout.clone(),
            stdout_sha256: e.stdout_sha256.clone(),
            stderr: e.stderr.clone(),
            stderr_sha256: e.stderr_sha256.clone(),
            truncated: e.truncated,
            evidence: e.evidence.clone(),
            prev_hash,
            hash,
        })
    }

    fn search(&self, f: &AuditFilter) -> Res<Vec<AuditRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = format!("SELECT {AUDIT_COLS} FROM audit WHERE 1=1");
        let mut args: Vec<String> = vec![];
        if let Some(id) = f.id {
            args.push(id.to_string());
            sql += &format!(" AND id=CAST(?{} AS INTEGER)", args.len());
        }
        if let Some(h) = &f.host {
            args.push(h.clone());
            let n = args.len();
            sql += &format!(" AND (host_label=?{n} COLLATE NOCASE OR host_id IN (SELECT id FROM hosts WHERE ip=?{n} OR lower(hostname)=lower(?{n})))");
        }
        if let Some(o) = &f.operation {
            args.push(format!("{o}%"));
            sql += &format!(" AND operation_id LIKE ?{}", args.len());
        }
        if let Some(s) = f.status {
            args.push(s.as_str().to_string());
            sql += &format!(" AND status=?{}", args.len());
        }
        if let Some(s) = &f.since {
            args.push(s.clone());
            sql += &format!(" AND ts>=?{}", args.len());
        }
        if let Some(t) = &f.text {
            args.push(format!("%{t}%"));
            let n = args.len();
            sql += &format!(" AND (command LIKE ?{n} OR host_label LIKE ?{n} OR operation_id LIKE ?{n})");
        }
        sql += " ORDER BY id DESC";
        if let Some(l) = f.limit {
            sql += &format!(" LIMIT {l}");
        }
        let mut st = conn.prepare(&sql)?;
        let rows = st.query_map(params_from_iter(args.iter()), row_to_audit)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    fn verify_chain(&self) -> Res<ChainReport> {
        let conn = self.conn.lock().unwrap();
        let mut st = conn.prepare(&format!("SELECT {AUDIT_COLS} FROM audit ORDER BY id ASC"))?;
        let rows = st.query_map([], row_to_audit)?;
        let mut prev = GENESIS.to_string();
        let mut total = 0usize;
        for r in rows {
            let r = r?;
            total += 1;
            let bad = |why: &str, prev: &str| ChainReport {
                total,
                ok: false,
                first_bad_id: Some(r.id),
                detail: why.to_string(),
                head_hash: prev.to_string(),
            };
            if r.prev_hash != prev {
                return Ok(bad("elo quebrado: o hash anterior não confere (registro removido, inserido ou reordenado)", &prev));
            }
            let canon = Canon {
                id: r.id,
                ts: &r.ts,
                actor: &r.actor,
                host_id: r.host_id,
                host_label: &r.host_label,
                operation_id: &r.operation_id,
                risk: r.risk.as_str(),
                command: &r.command,
                status: r.status.as_str(),
                exit_code: r.exit_code,
                duration_ms: r.duration_ms,
                stdout_sha256: &r.stdout_sha256,
                stderr_sha256: &r.stderr_sha256,
                evidence: &r.evidence,
                prev_hash: &r.prev_hash,
            };
            if chain_hash(&canon) != r.hash {
                return Ok(bad("conteúdo do registro foi alterado (hash não confere)", &prev));
            }
            if !r.truncated && (sha256_hex(r.stdout.as_bytes()) != r.stdout_sha256 || sha256_hex(r.stderr.as_bytes()) != r.stderr_sha256) {
                return Ok(bad("saída armazenada foi alterada (hash da saída não confere)", &prev));
            }
            prev = r.hash.clone();
        }
        Ok(ChainReport { total, ok: true, first_bad_id: None, detail: "cadeia íntegra".into(), head_hash: prev })
    }

    fn has_dry_run(&self, plan_hash: &str, since: &str) -> Res<bool> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit WHERE operation_id='dry-run' AND status='SUCCESS' AND ts>=?1 AND evidence LIKE ?2",
            params![since, format!("%\"plan_hash\":\"{plan_hash}\"%")],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }
}

impl IWorkflowRepository for SQLiteRepositories {
    fn find(&self, id: &str) -> Res<Option<Workflow>> {
        let conn = self.conn.lock().unwrap();
        let def: Option<String> = conn.query_row("SELECT definition FROM workflows WHERE id=?1", params![id], |r| r.get(0)).optional()?;
        match def {
            Some(d) => Ok(Some(serde_json::from_str(&d)?)),
            None => Ok(None),
        }
    }
    fn list(&self) -> Res<Vec<Workflow>> {
        let conn = self.conn.lock().unwrap();
        let mut st = conn.prepare("SELECT definition FROM workflows ORDER BY id")?;
        let rows = st.query_map([], |r| r.get::<_, String>(0))?;
        let mut out = vec![];
        for d in rows {
            out.push(serde_json::from_str(&d?)?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(op: &str, out: &str) -> NewAudit {
        NewAudit {
            ts: now_iso(),
            actor: "t".into(),
            host_id: None,
            host_label: "h".into(),
            operation_id: op.into(),
            risk: RiskLevel::Low,
            command: "c".into(),
            status: OperationStatus::Success,
            exit_code: Some(0),
            duration_ms: 1,
            stdout: out.into(),
            stdout_sha256: sha256_hex(out.as_bytes()),
            stderr: String::new(),
            stderr_sha256: sha256_hex(b""),
            truncated: false,
            evidence: "{}".into(),
        }
    }

    #[test]
    fn chain_detects_tampering() {
        let db = SQLiteRepositories::open_in_memory().unwrap();
        for i in 0..4 {
            db.append(&entry("x", &format!("out{i}"))).unwrap();
        }
        assert!(db.verify_chain().unwrap().ok);
        // adulteração de conteúdo
        db.conn.lock().unwrap().execute("UPDATE audit SET command='evil' WHERE id=2", []).unwrap();
        let r = db.verify_chain().unwrap();
        assert!(!r.ok);
        assert_eq!(r.first_bad_id, Some(2));
    }

    #[test]
    fn chain_detects_deletion_and_output_edit() {
        let db = SQLiteRepositories::open_in_memory().unwrap();
        for i in 0..4 {
            db.append(&entry("x", &format!("out{i}"))).unwrap();
        }
        db.conn.lock().unwrap().execute("DELETE FROM audit WHERE id=2", []).unwrap();
        let r = db.verify_chain().unwrap();
        assert!(!r.ok);
        assert_eq!(r.first_bad_id, Some(3));

        let db = SQLiteRepositories::open_in_memory().unwrap();
        db.append(&entry("x", "real output")).unwrap();
        db.conn.lock().unwrap().execute("UPDATE audit SET stdout='fake' WHERE id=1", []).unwrap();
        assert!(!db.verify_chain().unwrap().ok);
    }

    #[test]
    fn dry_run_ledger() {
        let db = SQLiteRepositories::open_in_memory().unwrap();
        let mut e = entry("dry-run", "");
        e.evidence = r#"{"plan_hash":"abc123"}"#.into();
        db.append(&e).unwrap();
        assert!(db.has_dry_run("abc123", "2000-01-01T00:00:00Z").unwrap());
        assert!(!db.has_dry_run("zzz", "2000-01-01T00:00:00Z").unwrap());
        assert!(!db.has_dry_run("abc123", "2999-01-01T00:00:00Z").unwrap());
    }

    #[test]
    fn upsert_marks_missing_hosts_down() {
        let db = SQLiteRepositories::open_in_memory().unwrap();
        let mk = |ip: &str| Host { ip: ip.into(), status: "up".into(), ..Default::default() };
        db.upsert_discovered(&[mk("10.0.0.1"), mk("10.0.0.2")], "10.0.0.0/24").unwrap();
        db.upsert_discovered(&[mk("10.0.0.1")], "10.0.0.0/24").unwrap();
        assert_eq!(IHostRepository::find(&db, "10.0.0.2").unwrap().unwrap().status, "down");
        assert_eq!(IHostRepository::find(&db, "10.0.0.1").unwrap().unwrap().status, "up");
    }
}
