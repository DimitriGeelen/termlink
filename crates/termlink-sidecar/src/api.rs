//! C-4 local API (architecture note section 3) and the C-10 status view.
//!
//! One unix stream socket per scope under `<runtime>/sidecar/` (dir 0700, sockets
//! 0600): `<agent>.send.sock`, `<agent>.status.sock`, `<agent>.admin.sock`. On
//! accept the peer uid is read with `SO_PEERCRED`; any uid other than the owner's
//! is refused (SI-14). Framing is one JSON object per line. An op is served only
//! on the socket of its own scope, so the agent sockets cannot reach `delivery`.
//!
//! B5 serves `status` and the admin `delivery` flag; B6 adds `send` (through
//! `send::Sender`, `not-configured` when none is attached). The remaining verbs
//! are known, scope-checked and refused with `code: "not-implemented"`: they
//! belong to B11 (reply) and B7/B9 (pull, claim, handed-over). Never faked.
//!
//! No verb takes a path: every field is on a per-op allow-list and every name is
//! checked against the store's name charset, so a path or command cannot arrive.

use crate::estate;
use crate::send::Sender;
use crate::store::{ChainStatus, Store, TAIL_NOTE};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

/// Longest accepted request line.
pub const MAX_REQUEST: u64 = 64 * 1024;
/// The PH-1 stage vocabulary, in order. `status` uses nothing else for stages.
pub const STAGES: [&str; 5] = ["SENT", "RECEIVED", "STORED", "HANDED_OVER", "REPLIED"];
/// Flag file in the store dir; present means delivery is off (SI-23).
pub const DELIVERY_OFF_FLAG: &str = "delivery.off";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Send,
    Status,
    Admin,
}

impl Scope {
    pub const ALL: [Scope; 3] = [Scope::Send, Scope::Status, Scope::Admin];
    pub fn name(self) -> &'static str {
        match self {
            Scope::Send => "send",
            Scope::Status => "status",
            Scope::Admin => "admin",
        }
    }
}

/// Scope an op belongs to, and the fields it may carry.
fn op_spec(op: &str) -> Option<(Scope, &'static [&'static str])> {
    Some(match op {
        "send" => (Scope::Send, &["to", "to_instance", "conv", "body", "priority"]),
        "reply" => (Scope::Send, &["conv", "in_reply_to", "body"]),
        "pull" => (Scope::Send, &["session", "hook"]),
        "claim" => (Scope::Send, &["msg_id"]),
        "handed-over" => (Scope::Send, &["msg_id", "nonce"]),
        "status" => (Scope::Status, &["msg_id", "conv"]),
        "delivery" => (Scope::Admin, &["state", "why"]),
        _ => return None,
    })
}

pub struct ApiConfig {
    pub agent: String,
    pub runtime: PathBuf,
    /// Uid allowed to connect. Defaults to the effective uid of this process.
    pub owner_uid: u32,
    /// Declared hub id (FX-2); `status` names it, or says none is configured.
    pub hub_id: Option<String>,
    /// The send path (B6). Without it `send` is refused as not configured.
    pub sender: Option<Arc<Sender>>,
}

impl ApiConfig {
    pub fn new(agent: &str, runtime: &Path) -> Self {
        ApiConfig {
            agent: agent.to_string(),
            runtime: runtime.to_path_buf(),
            // SAFETY: geteuid has no preconditions.
            owner_uid: unsafe { libc::geteuid() },
            hub_id: None,
            sender: None,
        }
    }
}

struct Shared {
    cfg: ApiConfig,
    store: Arc<Store>,
}

pub struct Api {
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
    paths: BTreeMap<Scope, PathBuf>,
}

pub fn socket_path(runtime: &Path, agent: &str, scope: Scope) -> PathBuf {
    runtime.join("sidecar").join(format!("{agent}.{}.sock", scope.name()))
}

fn valid_name(n: &str) -> bool {
    !n.is_empty()
        && n != "."
        && n != ".."
        && n.len() <= 200
        && n.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':'))
}

fn err(code: &str, msg: &str, hint: &str) -> Value {
    json!({"error": msg, "code": code, "hint": hint})
}

/// Peer uid of a connected unix socket (`SO_PEERCRED`).
pub fn peer_uid(s: &UnixStream) -> io::Result<u32> {
    let mut cred: libc::ucred = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `cred` and `len` are valid for the call; the fd is owned by `s`.
    let rc = unsafe {
        libc::getsockopt(
            s.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut len,
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(cred.uid)
}

impl Api {
    /// Bind the three scope sockets and serve them. Refuses to run outside a
    /// marked estate.
    pub fn start(cfg: ApiConfig, store: Arc<Store>) -> io::Result<Api> {
        estate::guard(&cfg.runtime, &cfg.runtime, store.dir())
            .map_err(|e| io::Error::new(io::ErrorKind::PermissionDenied, e.to_string()))?;
        if !valid_name(&cfg.agent) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "bad agent name"));
        }
        let dir = cfg.runtime.join("sidecar");
        fs::DirBuilder::new().recursive(true).mode(0o700).create(&dir)?;
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))?;
        let shared = Arc::new(Shared { cfg, store });
        let stop = Arc::new(AtomicBool::new(false));
        let mut threads = vec![];
        let mut paths = BTreeMap::new();
        for scope in Scope::ALL {
            let path = socket_path(&shared.cfg.runtime, &shared.cfg.agent, scope);
            let _ = fs::remove_file(&path);
            let l = UnixListener::bind(&path)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
            l.set_nonblocking(true)?;
            paths.insert(scope, path);
            let (sh, st) = (shared.clone(), stop.clone());
            threads.push(std::thread::spawn(move || accept_loop(l, scope, sh, st)));
        }
        Ok(Api { stop, threads, paths })
    }

    pub fn path(&self, scope: Scope) -> &Path {
        &self.paths[&scope]
    }

    pub fn shutdown(mut self) {
        self.stop_all();
    }

    fn stop_all(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
        for p in self.paths.values() {
            let _ = fs::remove_file(p);
        }
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        self.stop_all();
    }
}

fn accept_loop(l: UnixListener, scope: Scope, sh: Arc<Shared>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        match l.accept() {
            Ok((s, _)) => {
                let sh = sh.clone();
                std::thread::spawn(move || {
                    let _ = serve(s, scope, &sh);
                });
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(10)),
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn serve(s: UnixStream, scope: Scope, sh: &Shared) -> io::Result<()> {
    s.set_nonblocking(false)?;
    s.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut w = s.try_clone()?;
    match peer_uid(&s) {
        Ok(uid) if uid == sh.cfg.owner_uid => {}
        Ok(uid) => {
            let e = err(
                "peer-refused",
                &format!("peer uid {uid} is not the owner"),
                "only the owning user may use this socket",
            );
            return writeln!(w, "{e}");
        }
        Err(e) => {
            // Cannot establish who is calling: refuse.
            let v = err("peer-unknown", &format!("cannot read peer credentials: {e}"), "refused");
            return writeln!(w, "{v}");
        }
    }
    let mut r = BufReader::new(s);
    loop {
        let mut line = Vec::new();
        let n = (&mut r).take(MAX_REQUEST + 1).read_until(b'\n', &mut line)?;
        if n == 0 {
            return Ok(());
        }
        let resp = if line.len() as u64 > MAX_REQUEST || line.last() != Some(&b'\n') {
            err("bad-request", "request line too long or unterminated", "one JSON object per line, under 64 KiB")
        } else {
            match serde_json::from_slice::<Value>(&line) {
                Ok(v) => handle(&v, scope, sh),
                Err(e) => err("bad-request", &format!("not valid JSON: {e}"), "send one JSON object per line"),
            }
        };
        writeln!(w, "{resp}")?;
        if line.len() as u64 > MAX_REQUEST {
            return Ok(());
        }
    }
}

fn handle(req: &Value, scope: Scope, sh: &Shared) -> Value {
    dispatch_with(req, scope, &sh.store, &sh.cfg.agent, sh.cfg.hub_id.as_deref(), sh.cfg.sender.as_deref())
}

/// Parse and dispatch one request; independent of any socket.
pub fn dispatch(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id: Option<&str>) -> Value {
    dispatch_with(req, scope, store, agent, hub_id, None)
}

/// [`dispatch`] with the send path attached.
pub fn dispatch_with(
    req: &Value,
    scope: Scope,
    store: &Store,
    agent: &str,
    hub_id: Option<&str>,
    sender: Option<&Sender>,
) -> Value {
    let Some(obj) = req.as_object() else {
        return err("bad-request", "request must be a JSON object", "send {\"op\": ...}");
    };
    let Some(op) = obj.get("op").and_then(Value::as_str) else {
        return err("bad-request", "missing op", "send {\"op\": ...}");
    };
    let Some((need, fields)) = op_spec(op) else {
        return err("unknown-op", &format!("unknown op {op:?}"), "ops: send reply pull claim handed-over status delivery");
    };
    if need != scope {
        return err(
            "wrong-scope",
            &format!("op {op:?} needs the {} socket, this is the {} socket", need.name(), scope.name()),
            "use the socket of the op's scope; the admin socket is not given to the agent",
        );
    }
    for (k, v) in obj {
        if k == "op" {
            continue;
        }
        if !fields.contains(&k.as_str()) {
            return err("unknown-field", &format!("field {k:?} is not accepted by {op}"), "no verb takes a path or a command");
        }
        if let Some(s) = v.as_str()
            && matches!(k.as_str(), "msg_id" | "conv" | "to" | "to_instance" | "in_reply_to" | "session" | "nonce")
            && !valid_name(s)
        {
            return err("bad-name", &format!("field {k:?} is not a plain name"), "letters, digits, . _ - : only");
        }
    }
    match op {
        "status" => status(obj, store, agent, hub_id, sender),
        "send" => send(obj, sender),
        "delivery" => delivery(obj, store),
        _ => err(
            "not-implemented",
            &format!("op {op:?} is not served yet"),
            "arrives with the send path (B6) and the receive and deliver paths (B7, B9)",
        ),
    }
}

fn send(obj: &serde_json::Map<String, Value>, sender: Option<&Sender>) -> Value {
    let Some(sender) = sender else {
        return err("not-configured", "this sidecar has no send path configured", "start it with a hub address and a hub id");
    };
    let text = |k: &str| obj.get(k).and_then(Value::as_str);
    let (Some(to), Some(to_instance), Some(body)) = (text("to"), text("to_instance"), text("body")) else {
        return err("bad-request", "send needs to, to_instance and body", "{\"op\":\"send\",\"to\":...,\"to_instance\":...,\"body\":...}");
    };
    let priority = match obj.get("priority") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(_) => return err("bad-request", "priority must be an integer", "-9 to 9"),
    };
    match sender.send(to, to_instance, text("conv"), body, priority.as_deref()) {
        // Only the outbox stage is ever claimed here (FX-7).
        Ok(s) => json!({"msg_id": s.msg_id, "conv": s.conv, "seq": s.seq, "ts_ms": s.ts_ms, "stage": "SENT"}),
        Err(e) => err(
            e.code(),
            &e.to_string(),
            "nothing was queued; fix the cause and send again",
        ),
    }
}

fn delivery(obj: &serde_json::Map<String, Value>, store: &Store) -> Value {
    let flag = store.dir().join(DELIVERY_OFF_FLAG);
    match obj.get("state").and_then(Value::as_str) {
        Some("off") => {
            let why = obj.get("why").and_then(Value::as_str).unwrap_or("");
            let r = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&flag)
                .and_then(|mut f| {
                    writeln!(f, "{why}")?;
                    f.sync_all()
                });
            match r {
                Ok(()) => json!({"ok": true, "delivery": "off"}),
                Err(e) => err("io", &format!("cannot write the flag: {e}"), "delivery state is unchanged"),
            }
        }
        Some("on") => match fs::remove_file(&flag) {
            Ok(()) => json!({"ok": true, "delivery": "on"}),
            Err(e) if e.kind() == io::ErrorKind::NotFound => json!({"ok": true, "delivery": "on"}),
            Err(e) => err("io", &format!("cannot clear the flag: {e}"), "delivery state is unchanged"),
        },
        _ => err("bad-request", "state must be \"on\" or \"off\"", "{\"op\":\"delivery\",\"state\":\"off\"}"),
    }
}

fn status(obj: &serde_json::Map<String, Value>, store: &Store, agent: &str, hub_id: Option<&str>, sender: Option<&Sender>) -> Value {
    let want_msg = obj.get("msg_id").and_then(Value::as_str);
    let want_conv = obj.get("conv").and_then(Value::as_str);
    let convs: Vec<String> = match want_conv {
        Some(c) => vec![c.to_string()],
        None => store.conversations(),
    };
    let mut rows = vec![];
    let mut chains = BTreeMap::new();
    let mut by_msg: BTreeMap<String, usize> = BTreeMap::new();
    for c in &convs {
        match store.chain_lines(c) {
            Ok(lines) => {
                for l in lines {
                    if want_msg.is_some_and(|m| m != l.msg_id) {
                        continue;
                    }
                    if let Some(i) = STAGES.iter().position(|s| *s == l.event) {
                        let e = by_msg.entry(l.msg_id.clone()).or_insert(0);
                        *e = (*e).max(i + 1);
                    }
                    rows.push(json!({"conv": l.conv, "seq": l.seq, "msg_id": l.msg_id, "stage": l.event, "sender": l.sender, "ts_ms": l.ts_ms}));
                }
            }
            Err(e) => {
                chains.insert(c.clone(), json!({"intact": false, "reason": e.to_string()}));
                continue;
            }
        }
        let v = match store.verify_chain(c) {
            Ok(ChainStatus::Intact { len, head }) => json!({"intact": true, "len": len, "head": head}),
            Ok(ChainStatus::Broken { seq, reason }) => json!({"intact": false, "first_break_seq": seq, "reason": reason}),
            Err(e) => json!({"intact": false, "reason": e.to_string()}),
        };
        chains.insert(c.clone(), v);
    }
    // The strongest stage reached per message; the word "delivered" appears only
    // for a message whose chain holds a HANDED_OVER line.
    let msgs: Vec<Value> = by_msg
        .iter()
        .map(|(m, top)| {
            let reached = STAGES[top - 1];
            let handed = rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == "HANDED_OVER");
            let mut v = json!({"msg_id": m, "stage": reached});
            // Sender-side labels for a message that has not been seen by anyone yet:
            // WAITING when a post attempt found the hub unreachable, until it is posted.
            if *top == 1 {
                let has = |ev: &str| rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == ev);
                if has(crate::send::WAITING) && !has(crate::send::POSTED) {
                    v["label"] = json!("WAITING");
                }
                if has(crate::send::DEAD_LETTER) {
                    v["label"] = json!("REJECTED BY HUB");
                }
            }
            if handed {
                v["delivered"] = json!(true);
            }
            v
        })
        .collect();
    let delivery_on = !store.dir().join(DELIVERY_OFF_FLAG).exists();
    let flags = strength_flags(hub_id);
    let missing: Vec<&str> = flags.iter().filter(|(_, present, _)| !present).map(|(_, _, why)| *why).collect();
    let strength: BTreeMap<&str, bool> = flags.iter().map(|(n, p, _)| (*n, *p)).collect();
    json!({
        "ok": true,
        "agent": agent,
        "hub_id": hub_id.map(Value::from).unwrap_or_else(|| json!("not configured")),
        "stages": STAGES,
        "rows": rows,
        "messages": msgs,
        "chains": chains,
        "delivery": if delivery_on { "on" } else { "off" },
        "strength": strength,
        "missing_strength": missing,
        "notes": [TAIL_NOTE],
        "outbox": sender.map(|s| json!({"pending": s.outbox_len(), "held": s.held()})),
    })
}

/// Own strength flags: (name, present, text shown when absent).
fn strength_flags(hub_id: Option<&str>) -> Vec<(&'static str, bool, &'static str)> {
    vec![
        ("per_agent_accounts", false, "no per-agent accounts: agents share the host user"),
        ("operator_key", false, "no operator key: nothing here is signed by the operator"),
        ("owner_check_separates_same_user", false, "the owner check cannot separate processes of the same user"),
        ("chain_tail_anchor", false, TAIL_NOTE),
        ("hub_id_pinned", hub_id.is_some(), "no hub id configured: the hub is not pinned"),
    ]
}
