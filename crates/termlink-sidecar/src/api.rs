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
//! attached. B11 adds `reply` (through the send path). Every op answers with
//! what actually happened; none is faked.
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
/// FX-7 / FC-16: the only stage and label words a sender may see for a message
/// that has no HANDED_OVER. Nothing in it says or implies delivery.
pub const PRE_HANDOVER_VOCAB: [&str; 12] = [
    "SENT",
    "RECEIVED",
    "STORED",
    "WAITING",
    "WAITING FOR RECIPIENT",
    "STUCK",
    "UNKNOWN",
    "REJECTED BY HUB",
    "REPLIED",
    HELD_BY_RECIPIENT,
    REFUSED_BY_RECIPIENTS_OPERATOR,
    HELD_FOR_OPERATOR,
];
/// Sender-side label: the recipient holds the message for its operator (OQ-2 E).
pub const HELD_BY_RECIPIENT: &str = "HELD BY RECIPIENT (awaiting their operator)";
/// Sender-side label: the recipient's operator refused this sender.
pub const REFUSED_BY_RECIPIENTS_OPERATOR: &str = "REFUSED BY RECIPIENT'S OPERATOR";
/// Receiver-side label: a message from outside the allow-list, awaiting `allow`/`deny`.
pub const HELD_FOR_OPERATOR: &str = "HELD FOR OPERATOR";
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
        "held" => (Scope::Admin, &["body"]),
        "allow" => (Scope::Admin, &["key", "why"]),
        "deny" => (Scope::Admin, &["key", "why"]),
        "drain" => (Scope::Admin, &["why"]),
        "attention" => (Scope::Admin, &["since"]),
        "undrain" => (Scope::Admin, &["why"]),
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
    /// marked estate or an operator-marked production root.
    pub fn start(cfg: ApiConfig, store: Arc<Store>) -> io::Result<Api> {
        estate::guard_any(&cfg.runtime, &cfg.runtime, store.dir())
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
        return err("unknown-op", &format!("unknown op {op:?}"), "ops: send reply pull claim handed-over ready status delivery held allow deny");
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
                "msg_id" | "conv" | "to" | "to_instance" | "in_reply_to" | "session" | "nonce" | "tool_use_id" | "hook" | "key"
            )
            && !valid_name(s)
        {
            return err("bad-name", &format!("field {k:?} is not a plain name"), "letters, digits, . _ - : only");
        }
    }
    match op {
        "status" => status_at(obj, store, agent, hub_id, sender, crate::store::now_ms(), &crate::deadline::Deadlines::R44E),
        "send" => send(obj, sender),
        "delivery" => delivery(obj, store, who),
        "held" => held(obj, store),
        "allow" => decide(obj, store, agent, sender, who, true),
        "deny" => decide(obj, store, agent, sender, who, false),
        "drain" => drain(obj, store, ctx.deliverer, who),
        "attention" => attention(obj, store),
        "undrain" => undrain(obj, store, who),
        "pull" => pull(obj, ctx.deliverer),
        "claim" => claim(obj, ctx.deliverer),
        "handed-over" => handed_over(obj, ctx.deliverer),
        "ready" => ready(obj, ctx.deliverer),
        "reply" => reply(obj, sender),
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
    if !matches!(hook, "session-start" | "stop" | "session-end") {
        return err("bad-request", "hook must be session-start, stop or session-end", "the hook that is calling");
    }
    // A session that ended is no longer ready (PH-2 row P2): its stored mail
    // reads WAITING FOR RECIPIENT, never an overdue hand-over.
    if hook == "session-end" {
        return match d.mark_unready(session) {
            Ok(()) => json!({"ok": true, "ready": false}),
            Err(e) => err("io", &format!("cannot clear the session: {e}"), "the hook fails open"),
        };
    }
    match d.mark_ready(session, crate::store::now_ms()) {
        Ok(()) => json!({"ok": true, "ready": true}),
        Err(e) => err("io", &format!("cannot record the session: {e}"), "the hook fails open; mail waits for the next tool call"),
    }
}

fn reply(obj: &serde_json::Map<String, Value>, sender: Option<&Sender>) -> Value {
    let Some(sender) = sender else {
        return err("not-configured", "this sidecar has no send path configured", "start it with a hub address and a hub id");
    };
    let text = |k: &str| obj.get(k).and_then(Value::as_str);
    let (Some(conv), Some(orig), Some(body)) = (text("conv"), text("in_reply_to"), text("body")) else {
        return err("bad-request", "reply needs conv, in_reply_to and body", "{\"op\":\"reply\",\"conv\":...,\"in_reply_to\":...,\"body\":...}");
    };
    match sender.reply(conv, orig, body) {
        // Only the outbox stage is ever claimed here (FX-7).
        Ok(s) => json!({"msg_id": s.msg_id, "conv": s.conv, "seq": s.seq, "ts_ms": s.ts_ms, "stage": "SENT", "in_reply_to": orig}),
        Err(e) => err(e.code(), &e.to_string(), "nothing was queued; reply only to a message stored here"),
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

/// The needs-attention list, by name (PH-2 row P5, R-37). Every entry, numbered
/// from 1 in the order written; `since` skips the first N (what a reader has
/// already shown). Kinds starting with `urgent-` are alarms and are counted apart;
/// everything else only accumulates, with counts per kind.
fn attention(obj: &serde_json::Map<String, Value>, store: &Store) -> Value {
    let since = obj.get("since").and_then(Value::as_u64).unwrap_or(0) as usize;
    let all = store.attention_entries();
    let mut entries = vec![];
    let mut by_kind: BTreeMap<String, u64> = BTreeMap::new();
    let mut urgent = 0u64;
    for (i, line) in all.iter().enumerate() {
        let v: Value = serde_json::from_str(line).unwrap_or_else(|_| json!({"kind": "unreadable", "detail": line}));
        let kind = v["kind"].as_str().unwrap_or("unknown").to_owned();
        let is_urgent = kind.starts_with("urgent-");
        if i >= since {
            *by_kind.entry(kind.clone()).or_insert(0) += 1;
            if is_urgent {
                urgent += 1;
            }
            entries.push(json!({"n": i + 1, "kind": kind, "detail": v["detail"], "ts_ms": v["ts_ms"], "urgent": is_urgent}));
        }
    }
    json!({"ok": true, "total": all.len(), "since": since, "entries": entries, "by_kind": by_kind, "urgent": urgent})
}

/// State file holding the last drain's inventory (PH-2 row P4).
pub const DRAIN_MANIFEST: &str = "drain-manifest";
/// How long a drain waits after setting its flag before it counts, so a receive
/// pass that was mid-message when the flag appeared has finished storing it.
pub const DRAIN_SETTLE: std::time::Duration = std::time::Duration::from_millis(1500);

/// FP 6.2.3.e: refuse new mail (it stays on the hub; senders keep chasing),
/// hand nothing over, and keep every stored message not yet handed over; count
/// what is kept against what is listed. Only a matching count says the old
/// receiver may start. Idempotent: a second drain counts again.
fn drain(obj: &serde_json::Map<String, Value>, store: &Store, d: Option<&Deliverer>, who: &str) -> Value {
    let Some(d) = d else { return no_deliverer() };
    let why = obj.get("why").and_then(Value::as_str).unwrap_or("");
    let flag = store.dir().join(crate::policy::DRAINED_FLAG);
    let set = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&flag)
        .and_then(|mut f| {
            writeln!(f, "{why}")?;
            f.sync_all()
        });
    if let Err(e) = set {
        return err("io", &format!("cannot write the drain flag: {e}"), "nothing changed; the sidecar still receives");
    }
    std::thread::sleep(DRAIN_SETTLE);
    let inv = d.drain_inventory();
    let at = crate::store::now_ms();
    let manifest = json!({"at_ms": at, "why": why, "by": who, "listed": inv.listed, "kept": inv.kept, "unreadable": inv.unreadable});
    if let Err(e) = store.write_state(DRAIN_MANIFEST, &manifest.to_string()) {
        return err("io", &format!("drained, but the manifest could not be written: {e}"), "the sidecar is drained; run drain again");
    }
    let summary = format!("listed={} kept={} unreadable={} {why}", inv.listed, inv.kept.len(), inv.unreadable.len());
    if let Err(e) = log_switch(store, "DRAIN", who, &summary) {
        return err("log", &format!("drained, but the drain could not be logged: {e}"), "the flag is set; check the store");
    }
    let mut v = json!({
        "ok": inv.counts_match(),
        "drained": true,
        "listed": inv.listed,
        "kept": inv.kept.len(),
        "unreadable": inv.unreadable,
        "manifest": format!("state.{DRAIN_MANIFEST}"),
    });
    if !inv.counts_match() {
        let _ = store.attention("drain-count-mismatch", &summary);
        v["code"] = json!("count-mismatch");
        v["error"] = json!(format!("{} listed, {} kept intact: do not start the old receiver", inv.listed, inv.kept.len()));
    }
    v
}

/// Undo a drain: receive and hand over again.
fn undrain(obj: &serde_json::Map<String, Value>, store: &Store, who: &str) -> Value {
    let why = obj.get("why").and_then(Value::as_str).unwrap_or("");
    match fs::remove_file(store.dir().join(crate::policy::DRAINED_FLAG)) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return err("io", &format!("cannot remove the drain flag: {e}"), "the sidecar is still drained"),
    }
    match log_switch(store, "UNDRAIN", who, why) {
        Ok(()) => json!({"ok": true, "drained": false}),
        Err(e) => err("log", &format!("receiving again, but the undrain could not be logged: {e}"), "check the store"),
    }
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

/// Messages held for the operator (OQ-2 E): a HELD line with no ADMITTED and
/// no REFUSED line after it for the same message and sender.
fn held_lines(store: &Store) -> Vec<crate::store::ChainLine> {
    let mut out = vec![];
    for conv in store.conversations() {
        let Ok(lines) = store.chain_lines(&conv) else { continue };
        for h in lines.iter().filter(|l| l.event == crate::pull::HELD) {
            let decided = lines.iter().any(|l| {
                l.msg_id == h.msg_id && l.sender == h.sender && (l.event == crate::pull::ADMITTED || l.event == crate::pull::REFUSED)
            });
            if !decided {
                out.push(h.clone());
            }
        }
    }
    out
}

/// The stored message behind a chain line, from its STORED record.
fn stored_msg(store: &Store, line: &crate::store::ChainLine) -> Option<crate::wire::Msg> {
    let lines = store.chain_lines(&line.conv).ok()?;
    let st = lines.iter().find(|l| l.event == "STORED" && l.msg_id == line.msg_id && l.sender == line.sender)?;
    let (_, payload) = store.read_record(&line.conv, st.seq, &line.msg_id).ok()?;
    crate::wire::parse_payload(&payload).ok()
}

/// `held`: what is waiting for the operator, by name. `body: true` adds each
/// message's text, for the operator only (this is the admin socket).
fn held(obj: &serde_json::Map<String, Value>, store: &Store) -> Value {
    let with_body = obj.get("body").and_then(Value::as_bool).unwrap_or(false);
    let items: Vec<Value> = held_lines(store)
        .iter()
        .map(|l| {
            let m = stored_msg(store, l);
            let mut v = json!({
                "key": l.sender, "conv": l.conv, "msg_id": l.msg_id, "held_at_ms": l.ts_ms,
                "claims_agent": m.as_ref().map(|m| crate::pull::shown(&m.from_agent)),
                "claims_instance": m.as_ref().map(|m| crate::pull::shown(&m.from_instance)),
                "decide": format!("termlink-sidecar admin allow|deny --key {} --why <reason>", l.sender),
            });
            if with_body {
                v["body"] = json!(m.map(|m| m.body));
            }
            v
        })
        .collect();
    json!({"ok": true, "held": items})
}

/// `allow` / `deny` (OQ-2 E): the operator's decision about a key that knocked.
/// Only a key with a held message can be decided: this is an answer to a
/// question this side asked, not a general way to configure the allow-list.
/// `allow` admits the key (its held messages become deliverable at the agent's
/// next tool call); `deny` refuses them. Either way the decision is logged in
/// `_admin` with who, when and why, and the sender gets the notice.
fn decide(obj: &serde_json::Map<String, Value>, store: &Store, agent: &str, sender: Option<&Sender>, who: &str, admit: bool) -> Value {
    use crate::policy::{ADMITTED_KEYS, REFUSED_BY_OPERATOR, REFUSED_KEYS};
    let Some(key) = obj.get("key").and_then(Value::as_str) else {
        return err("bad-request", "missing key", "{\"op\":\"allow\",\"key\":\"<fingerprint>\",\"why\":\"...\"}");
    };
    let why = obj.get("why").and_then(Value::as_str).unwrap_or("");
    let pk = match store.read_state(&format!("{}{key}", crate::pull::HELD_KEY_PREFIX)) {
        Ok(Some(pk)) if crate::wire::parse_public_key(pk.trim()).is_some() => pk.trim().to_owned(),
        Ok(_) => {
            return err(
                "no-knock",
                &format!("no message from key {key} was ever held here"),
                "only a key that knocked can be decided; `held` lists them",
            );
        }
        Err(e) => return err("io", &format!("cannot read the held key: {e}"), "nothing was changed"),
    };
    let read_list = |name: &str| -> Result<Vec<String>, String> {
        Ok(store
            .read_state(name)
            .map_err(|e| e.to_string())?
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_owned())
            .filter(|l| !l.is_empty())
            .collect())
    };
    let (mut admitted, mut refused) = match (read_list(ADMITTED_KEYS), read_list(REFUSED_KEYS)) {
        (Ok(a), Ok(r)) => (a, r),
        (Err(e), _) | (_, Err(e)) => return err("io", &format!("cannot read the operator lists: {e}"), "nothing was changed"),
    };
    if admit {
        refused.retain(|k| k != key);
        if !admitted.contains(&pk) {
            admitted.push(pk.clone());
        }
    } else {
        admitted.retain(|k| *k != pk);
        if !refused.iter().any(|k| k == key) {
            refused.push(key.to_owned());
        }
    }
    let write = |name: &str, v: &[String]| store.write_state(name, &(v.join("\n") + "\n"));
    if let Err(e) = write(REFUSED_KEYS, &refused).and_then(|_| write(ADMITTED_KEYS, &admitted)) {
        return err("io", &format!("cannot write the operator lists: {e}"), "check the store; the decision may be partly applied");
    }
    let event = if admit { "ALLOW" } else { "DENY" };
    if let Err(e) = log_switch(store, event, who, &format!("key {key}: {}", if why.is_empty() { "no reason given" } else { why })) {
        return err("log", &format!("the decision is applied but could not be logged: {e}"), "check the store");
    }
    // The held messages of this key: mark them, and tell the sender.
    let (line_event, notice, reason) = if admit {
        (crate::pull::ADMITTED, crate::pull::ADMITTED, format!("by the operator: {why}"))
    } else {
        (crate::pull::REFUSED, crate::pull::DENIED, REFUSED_BY_OPERATOR.to_owned())
    };
    let mut decided = vec![];
    let mut notice_errors = vec![];
    for h in held_lines(store).into_iter().filter(|l| l.sender == key) {
        let extra = crate::store::LineExtra { reason: Some(reason.clone()), ..Default::default() };
        let (id, from) = (h.msg_id.clone(), h.sender.clone());
        if let Err(e) = store.append_event_unless(&h.conv, line_event, &h.msg_id, &h.sender, &h.digest, extra, move |ls| {
            ls.iter().any(|l| l.msg_id == id && l.sender == from && (l.event == crate::pull::ADMITTED || l.event == crate::pull::REFUSED))
        }) {
            notice_errors.push(format!("{}: {line_event} not recorded: {e}", h.msg_id));
            continue;
        }
        decided.push(h.msg_id.clone());
        match (sender, stored_msg(store, &h)) {
            (Some(s), Some(m)) => {
                if let Err(e) = crate::pull::build_stage(store, s.fingerprint(), agent, &m, notice).and_then(|st| s.post_stage(&m.from_agent, &st).map_err(|e| e.to_string())) {
                    notice_errors.push(format!("{}: {notice} notice not queued: {e}", h.msg_id));
                }
            }
            (None, _) => notice_errors.push(format!("{}: no send path, the sender was not told", h.msg_id)),
            (_, None) => notice_errors.push(format!("{}: stored record unreadable, the sender was not told", h.msg_id)),
        }
    }
    for e in &notice_errors {
        let _ = store.attention("decision-notice", e);
    }
    json!({"ok": true, "key": key, "decision": if admit { "allowed" } else { "denied" }, "messages": decided, "notice_errors": notice_errors})
}

/// The `status` view at a given clock (tests move the clock; the socket uses now).
pub fn status_view(req: &Value, store: &Store, agent: &str, hub_id: Option<&str>, now_ms: u64, dl: &crate::deadline::Deadlines) -> Value {
    match req.as_object() {
        Some(obj) => status_at(obj, store, agent, hub_id, None, now_ms, dl),
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
    dl: &crate::deadline::Deadlines,
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
            let has = |ev: &str| rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == ev);
            let first_ts = |ev: &str| {
                rows.iter()
                    .filter(|r| r["msg_id"] == *m && r["stage"] == ev)
                    .filter_map(|r| r["peer_ts_ms"].as_u64().or_else(|| r["ts_ms"].as_u64()))
                    .min()
            };
            // Urgent per the SENT line's recorded priority (R-44.e urgent deadlines).
            let urgent = rows
                .iter()
                .filter(|r| r["msg_id"] == *m && r["stage"] == "SENT")
                .filter_map(|r| r["reason"].as_str()?.strip_prefix("priority=")?.parse::<i64>().ok())
                .any(|p| p >= 5);
            // When the message left: first post, else the SENT line.
            let sent_at = first_ts(crate::send::POSTED).or_else(|| first_ts("SENT"));
            let stuck = |v: &mut Value, step: &str| {
                v["label"] = json!("STUCK");
                v["stuck_step"] = json!(step);
            };
            // Sender-side labels for a message that has not been seen by anyone yet:
            // WAITING when a post attempt found the hub unreachable, until it is posted.
            if *top == 1 {
                if has(crate::send::WAITING) && !has(crate::send::POSTED) {
                    v["label"] = json!("WAITING");
                }
                if has(crate::send::DEAD_LETTER) {
                    v["label"] = json!("REJECTED BY HUB");
                }
                // Posted, nothing back past the accept deadline. Without a liveness
                // signal (row P3) the recipient may simply not be running: UNKNOWN,
                // never STUCK (R-44.a).
                let posted = first_ts(crate::send::POSTED);
                if posted.is_some_and(|t| now_ms.saturating_sub(t) > dl.accept(urgent)) {
                    v["label"] = json!("UNKNOWN");
                }
            }
            // The record (BV-7): the five stages in order, each with its time
            // (the reporting side's own time for a stage a peer reported).
            let record: Vec<Value> = STAGES
                .iter()
                .map(|st| {
                    match rows.iter().find(|r| r["msg_id"] == *m && r["stage"] == *st) {
                        Some(r) => json!({
                            "stage": st,
                            "at_ms": r["peer_ts_ms"].as_u64().or_else(|| r["ts_ms"].as_u64()),
                            "by": r["sender"],
                        }),
                        None => json!({"stage": st, "absent": true}),
                    }
                })
                .collect();
            v["record"] = json!(record);
            if handed {
                v["delivered"] = json!(true);
            }
            // Sender side: the recipient stored it, nothing handed it over yet.
            // No hook has fired (the agent is idle or has no hook): waiting; past
            // the deadline: stuck (FC-6, FC-7). Storage is never called delivery.
            let sent_here = rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == "SENT");
            // R-44.e (PH-2 row P2). Received but never stored: the recipient is
            // demonstrably alive, so past the accept deadline it is STUCK.
            if sent_here && reached == "RECEIVED" && sent_at.is_some_and(|t| now_ms.saturating_sub(t) > dl.accept(urgent)) {
                stuck(&mut v, crate::deadline::STEP_NOT_ACCEPTED);
            }
            // Stored, not handed over: only the recipient knows when its agent became
            // ready, so STUCK only on its overdue notice; otherwise the recipient is
            // busy or away (no false STUCK on a busy agent).
            if sent_here && reached == "STORED" {
                if has(crate::deadline::OVERDUE_HANDOVER) {
                    stuck(&mut v, crate::deadline::STEP_NOT_HANDED_OVER);
                } else {
                    v["label"] = json!("WAITING FOR RECIPIENT");
                }
            }
            // Handed over, not answered: STUCK on the recipient's notice; past the
            // deadline without one, liveness is unknown (row P3).
            if sent_here && reached == "HANDED_OVER" {
                if has(crate::deadline::OVERDUE_ANSWER) {
                    stuck(&mut v, crate::deadline::STEP_NOT_ANSWERED);
                } else if first_ts("HANDED_OVER").is_some_and(|t| now_ms.saturating_sub(t) > dl.answer(urgent)) {
                    v["label"] = json!("UNKNOWN");
                }
            }
            // OQ-2 E: held for an operator, on either side.
            let ev = |e: &str| rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == e);
            if sent_here {
                if ev(crate::pull::DENIED) {
                    v["label"] = json!(REFUSED_BY_RECIPIENTS_OPERATOR);
                } else if ev(crate::pull::HELD) && !ev(crate::pull::ADMITTED) && !handed {
                    v["label"] = json!(HELD_BY_RECIPIENT);
                }
            } else if ev(crate::pull::HELD) && !ev(crate::pull::ADMITTED) && !ev("REFUSED") {
                v["label"] = json!(HELD_FOR_OPERATOR);
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
        "mode": store.mode().as_str(),
        "receiving": if crate::policy::receiving_enabled(store) { "on" } else { "drained" },
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
