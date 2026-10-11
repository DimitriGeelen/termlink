//! C-4 local API (architecture note section 3) and the C-10 status view.
//!
//! One unix stream socket per scope under `<runtime>/sidecar/` (dir 0700, sockets
//! 0600): `<agent>.send.sock`, `<agent>.status.sock`, `<agent>.admin.sock`. On
//! accept the peer uid is read with `SO_PEERCRED`; any uid other than the owner's
//! is refused (SI-14). Framing is one JSON object per line. An op is served only
//! on the socket of its own scope, so the agent sockets cannot reach `delivery`.
//!
//! B5 serves `status` and the admin `delivery` flag; B6 adds `send` (through
//! `send::Sender`), B9 adds `pull`, `claim` and `handed-over` (through
//! `deliver::Deliverer`); each answers `not-configured` when its path is not
//! attached. `reply` is known, scope-checked and refused with
//! `code: "not-implemented"` until B11. Never faked.
//!
//! No verb takes a path: every field is on a per-op allow-list and every name is
//! checked against the store's name charset, so a path or command cannot arrive.

use crate::estate;
use crate::deliver::{Deliverer, Pulled, Verdict};
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
pub use crate::policy::DELIVERY_OFF_FLAG;
/// How long a message may sit at STORED on the recipient before the sender's
/// view calls it STUCK instead of WAITING FOR RECIPIENT (FC-6, FC-7).
pub const STUCK_AFTER_MS: u64 = 10 * 60 * 1000;
/// Conversation that holds the delivery switch's log lines (SI-23).
pub const ADMIN_CONV: &str = "_admin";

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
        "pull" => (Scope::Send, &["session", "hook", "tool_use_id"]),
        "claim" => (Scope::Send, &["msg_id"]),
        "handed-over" => (Scope::Send, &["msg_id", "nonce", "final"]),
        "ready" => (Scope::Send, &["session", "hook"]),
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
    /// The deliver path (B9). Without it `pull`, `claim`, `handed-over` are refused as not configured.
    pub deliverer: Option<Arc<Deliverer>>,
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
            deliverer: None,
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
    // The peer passed the owner check, so its uid is the owner's.
    let who = format!("admin-uid-{}", sh.cfg.owner_uid);
    let ctx = Ctx { sender: sh.cfg.sender.as_deref(), deliverer: sh.cfg.deliverer.as_deref(), who: &who };
    dispatch_inner(req, scope, &sh.store, &sh.cfg.agent, sh.cfg.hub_id.as_deref(), ctx)
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
    dispatch_inner(req, scope, store, agent, hub_id, Ctx { sender, deliverer: None, who: "admin-direct" })
}

/// [`dispatch_with`] with the deliver path attached as well.
pub fn dispatch_full(
    req: &Value,
    scope: Scope,
    store: &Store,
    agent: &str,
    hub_id: Option<&str>,
    sender: Option<&Sender>,
    deliverer: Option<&Deliverer>,
) -> Value {
    dispatch_inner(req, scope, store, agent, hub_id, Ctx { sender, deliverer, who: "admin-direct" })
}

#[derive(Clone, Copy)]
struct Ctx<'a> {
    sender: Option<&'a Sender>,
    deliverer: Option<&'a Deliverer>,
    who: &'a str,
}

fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id: Option<&str>, ctx: Ctx<'_>) -> Value {
    let (sender, who) = (ctx.sender, ctx.who);
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
            && matches!(
                k.as_str(),
                "msg_id" | "conv" | "to" | "to_instance" | "in_reply_to" | "session" | "nonce" | "tool_use_id" | "hook"
            )
            && !valid_name(s)
        {
            return err("bad-name", &format!("field {k:?} is not a plain name"), "letters, digits, . _ - : only");
        }
    }
    match op {
        "status" => status_at(obj, store, agent, hub_id, sender, crate::store::now_ms(), STUCK_AFTER_MS),
        "send" => send(obj, sender),
        "delivery" => delivery(obj, store, who),
        "pull" => pull(obj, ctx.deliverer),
        "claim" => claim(obj, ctx.deliverer),
        "handed-over" => handed_over(obj, ctx.deliverer),
        "ready" => ready(obj, ctx.deliverer),
        _ => err(
            "not-implemented",
            &format!("op {op:?} is not served yet"),
            "arrives with the reply path (B11)",
        ),
    }
}

fn no_deliverer() -> Value {
    err("not-configured", "this sidecar has no deliver path configured", "start it with a store, a policy and a send path")
}

fn pull(obj: &serde_json::Map<String, Value>, d: Option<&Deliverer>) -> Value {
    let Some(d) = d else { return no_deliverer() };
    let text = |k: &str| obj.get(k).and_then(Value::as_str);
    let (Some(session), Some(hook)) = (text("session"), text("hook")) else {
        return err("bad-request", "pull needs session and hook", "{\"op\":\"pull\",\"session\":...,\"hook\":\"post-tool-use\"}");
    };
    if !matches!(hook, "post-tool-use" | "stop") {
        return err("bad-request", "hook must be post-tool-use or stop", "the hook that is calling");
    }
    match d.pull(session, hook, text("tool_use_id")) {
        Ok(Pulled::Nothing { why }) => json!({"none": true, "why": why}),
        Ok(Pulled::Frame { frame, waiting }) => json!({
            "frame": frame.text,
            "nonce": frame.nonce,
            "msg_ids": frame.msg_ids,
            "waiting": waiting,
            "stage": "CLAIMED",
        }),
        Err(e) => err("deliver", &e, "nothing was handed out; the claims taken stay open and are named in needs-attention"),
    }
}

fn claim(obj: &serde_json::Map<String, Value>, d: Option<&Deliverer>) -> Value {
    let Some(d) = d else { return no_deliverer() };
    let Some(id) = obj.get("msg_id").and_then(Value::as_str) else {
        return err("bad-request", "claim needs msg_id", "{\"op\":\"claim\",\"msg_id\":...}");
    };
    match d.claim_one(id) {
        Ok((attempt, nonce)) => json!({"claimed": true, "attempt": attempt, "nonce": nonce}),
        Err(code) => err(&code, &format!("not claimed: {code}"), "held: another attempt is open; not-deliverable: not stored, refused, withheld or handed over"),
    }
}

fn handed_over(obj: &serde_json::Map<String, Value>, d: Option<&Deliverer>) -> Value {
    let Some(d) = d else { return no_deliverer() };
    let Some(nonce) = obj.get("nonce").and_then(Value::as_str) else {
        return err("bad-request", "handed-over needs nonce", "{\"op\":\"handed-over\",\"nonce\":...}");
    };
    if let Some(id) = obj.get("msg_id").and_then(Value::as_str) {
        match d.attempt(nonce) {
            Some(rec) if rec.items.iter().any(|i| i.msg_id == id) => {}
            Some(_) => return err("bad-request", "that message is not in this attempt", "use the nonce pull returned with it"),
            None => {}
        }
    }
    let final_check = obj.get("final").and_then(Value::as_bool).unwrap_or(false);
    // A caller cannot assert the hand-over: the verdict comes from the transcript only.
    match d.finalize(nonce, final_check) {
        Ok(Verdict::HandedOver) => json!({"stage": "HANDED_OVER"}),
        Ok(Verdict::Absent) => json!({"stage": "STORED", "attempt": "closed: not in the transcript, offered again"}),
        Ok(Verdict::Uncertain(why)) => json!({"stage": "STORED", "attempt": "open", "label": "UNCERTAIN", "why": why}),
        Ok(Verdict::Pending) => json!({"stage": "STORED", "attempt": "open", "label": "PENDING"}),
        Err(e) => err("bad-request", &e, "use the nonce pull returned"),
    }
}

fn ready(obj: &serde_json::Map<String, Value>, d: Option<&Deliverer>) -> Value {
    let Some(d) = d else { return no_deliverer() };
    let text = |k: &str| obj.get(k).and_then(Value::as_str);
    let (Some(session), Some(hook)) = (text("session"), text("hook")) else {
        return err("bad-request", "ready needs session and hook", "{\"op\":\"ready\",\"session\":...,\"hook\":\"session-start\"}");
    };
    if !matches!(hook, "session-start" | "stop") {
        return err("bad-request", "hook must be session-start or stop", "the hook that is calling");
    }
    match d.mark_ready(session, crate::store::now_ms()) {
        Ok(()) => json!({"ok": true, "ready": true}),
        Err(e) => err("io", &format!("cannot record the session: {e}"), "the hook fails open; mail waits for the next tool call"),
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

/// Log a switch flip on the chain: who (the admin socket's uid), when (the
/// line's time), why (the reason field) (SI-23).
fn log_switch(store: &Store, event: &str, who: &str, why: &str) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(why.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let id = format!("switch-{}", crate::store::now_ms());
    let reason = if why.is_empty() { "no reason given".to_owned() } else { why.to_owned() };
    store
        .append_event_unless(
            ADMIN_CONV,
            event,
            &id,
            who,
            &digest,
            crate::store::LineExtra { reason: Some(reason), ..Default::default() },
            |_| false,
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn delivery(obj: &serde_json::Map<String, Value>, store: &Store, who: &str) -> Value {
    let flag = store.dir().join(DELIVERY_OFF_FLAG);
    let why = obj.get("why").and_then(Value::as_str).unwrap_or("");
    let logged = |state: &str, event: &str| match log_switch(store, event, who, why) {
        Ok(()) => json!({"ok": true, "delivery": state}),
        Err(e) => err(
            "log",
            &format!("delivery is {state} but the switch could not be logged: {e}"),
            "the flag is set; check the store",
        ),
    };
    match obj.get("state").and_then(Value::as_str) {
        Some("off") => {
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
                Ok(()) => logged("off", "DELIVERY_OFF"),
                Err(e) => err("io", &format!("cannot write the flag: {e}"), "delivery state is unchanged"),
            }
        }
        Some("on") => match fs::remove_file(&flag) {
            Ok(()) => logged("on", "DELIVERY_ON"),
            Err(e) if e.kind() == io::ErrorKind::NotFound => logged("on", "DELIVERY_ON"),
            Err(e) => err("io", &format!("cannot clear the flag: {e}"), "delivery state is unchanged"),
        },
        _ => err("bad-request", "state must be \"on\" or \"off\"", "{\"op\":\"delivery\",\"state\":\"off\"}"),
    }
}

/// The `status` view at a given clock (tests move the clock; the socket uses now).
pub fn status_view(req: &Value, store: &Store, agent: &str, hub_id: Option<&str>, now_ms: u64, stuck_after_ms: u64) -> Value {
    match req.as_object() {
        Some(obj) => status_at(obj, store, agent, hub_id, None, now_ms, stuck_after_ms),
        None => err("bad-request", "request must be a JSON object", "send {\"op\": \"status\"}"),
    }
}

fn status_at(
    obj: &serde_json::Map<String, Value>,
    store: &Store,
    agent: &str,
    hub_id: Option<&str>,
    sender: Option<&Sender>,
    now_ms: u64,
    stuck_after_ms: u64,
) -> Value {
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
                    rows.push(json!({"conv": l.conv, "seq": l.seq, "msg_id": l.msg_id, "stage": l.event, "sender": l.sender, "ts_ms": l.ts_ms, "peer_ts_ms": l.peer_ts_ms, "reason": l.reason}));
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
            // Sender side: the recipient stored it, nothing handed it over yet.
            // No hook has fired (the agent is idle or has no hook): waiting; past
            // the deadline: stuck (FC-6, FC-7). Storage is never called delivery.
            let sent_here = rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == "SENT");
            if sent_here && reached == "STORED" {
                let stored_at = rows
                    .iter()
                    .filter(|r| r["msg_id"] == *m && r["stage"] == "STORED")
                    .filter_map(|r| r["peer_ts_ms"].as_u64().or_else(|| r["ts_ms"].as_u64()))
                    .min();
                let stuck = stored_at.is_some_and(|t| now_ms.saturating_sub(t) > stuck_after_ms);
                v["label"] = json!(if stuck { "STUCK" } else { "WAITING FOR RECIPIENT" });
            }
            // Stored but withheld (e.g. not-allowed), or refused outright: say why.
            if let Some(r) = rows.iter().find(|r| r["msg_id"] == *m && r["stage"] == "REFUSED") {
                v["refused"] = r["reason"].clone();
            }
            v
        })
        .collect();
    let delivery_on = crate::policy::delivery_enabled(store);
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
