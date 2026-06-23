//! T-2045 (T-2020 GO): `termlink agent find-idle` CLI verb.
//!
//! Calls the hub's `agent.find_idle` RPC (T-2045 slice 1) over the local
//! UDS socket and renders the result. Pure read — no state mutation.
//!
//! Local-hub-only by design (per T-2020 inception §5.4 "What's NOT in this
//! primitive"). Cross-hub finding is the orchestrator's job — it walks
//! `hubs.toml` and calls find-idle per hub.
//!
//! T-2078 added `--watch <secs>` continuous monitor (substrate primitive
//! #2 observability arc Slice 1) — periodic re-render of the idle roster
//! with diff scaffolding for future `--notify` / `--log` slices.

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use termlink_protocol::control::method;
use termlink_protocol::transport::TransportAddr;
use termlink_session::client;

/// T-2078: per-agent snapshot kept across watch ticks for the diff helper.
/// Only carries what `--notify` / `--log` will need in future slices —
/// agent_id is the BTreeMap key, so it lives outside the snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdleSnapshot {
    pub last_heartbeat_ms: i64,
    pub role: Option<String>,
    pub capabilities: Vec<String>,
}

/// T-2078: change-event kinds for the find-idle diff. Idle is binary
/// (in the result or not) — there is no "transition" because the only
/// state is "currently idle". An agent that goes busy disappears from
/// the result (Removed); an agent that frees up reappears (New). An
/// agent that simply re-heartbeats while still idle produces NO event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IdleChangeKind {
    New,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdleChangeEvent {
    pub agent_id: String,
    pub kind: IdleChangeKind,
    pub snap: IdleSnapshot,
}

/// T-2078: pure helper diffing two idle-roster snapshots into a list of
/// change events. Returns `New` for agents present in `curr` but absent
/// from `prev`, `Removed` for agents present in `prev` but absent from
/// `curr`. Agents present in both produce NO event (re-heartbeat is not
/// a state change — see IdleChangeKind doc). Extracted for unit testing
/// without spinning up a hub.
pub(crate) fn diff_idle_sets(
    prev: &std::collections::BTreeMap<String, IdleSnapshot>,
    curr: &std::collections::BTreeMap<String, IdleSnapshot>,
) -> Vec<IdleChangeEvent> {
    let mut out = Vec::new();
    // New = in curr, not in prev.
    for (agent_id, snap) in curr {
        if !prev.contains_key(agent_id) {
            out.push(IdleChangeEvent {
                agent_id: agent_id.clone(),
                kind: IdleChangeKind::New,
                snap: snap.clone(),
            });
        }
    }
    // Removed = in prev, not in curr. Carry the PRIOR snapshot since the
    // agent's current state is "not idle" (no snapshot available).
    for (agent_id, snap) in prev {
        if !curr.contains_key(agent_id) {
            out.push(IdleChangeEvent {
                agent_id: agent_id.clone(),
                kind: IdleChangeKind::Removed,
                snap: snap.clone(),
            });
        }
    }
    out
}

/// T-2078: parse the JSON-RPC `agent.find_idle` result envelope into a
/// snapshot map (agent_id → IdleSnapshot). Order is lost — BTreeMap is
/// alphabetical, but `--watch` re-renders fresh each tick so the ordering
/// shows up consistently. Extracted as a pure function so the watch loop
/// and the diff path share one parser.
pub(crate) fn parse_idle_result(
    result: &Value,
) -> std::collections::BTreeMap<String, IdleSnapshot> {
    let mut out = std::collections::BTreeMap::new();
    let arr = match result.get("idle").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return out,
    };
    for entry in arr {
        let agent_id = match entry.get("agent_id").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let last_heartbeat_ms = entry
            .get("last_heartbeat_ms")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let role = entry
            .get("role")
            .and_then(|v| v.as_str())
            .map(String::from);
        let capabilities: Vec<String> = entry
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        out.insert(
            agent_id,
            IdleSnapshot {
                last_heartbeat_ms,
                role,
                capabilities,
            },
        );
    }
    out
}

/// T-2078: render the idle roster as a human-readable table. Same shape
/// as the original snapshot output; extracted so both one-shot and watch
/// paths use one renderer.
fn render_idle_table(snap: &std::collections::BTreeMap<String, IdleSnapshot>, now_ms: i64) {
    if snap.is_empty() {
        println!("(no idle agents matching filter)");
        return;
    }
    for (agent_id, s) in snap {
        let age_s = ((now_ms - s.last_heartbeat_ms) / 1000).max(0);
        let role_str = s.role.as_deref().unwrap_or("-");
        let caps_str = if s.capabilities.is_empty() {
            "-".to_string()
        } else {
            s.capabilities.join(",")
        };
        println!(
            "{agent_id}\tage={age_s}s\trole={role_str}\tcapabilities={caps_str}"
        );
    }
}

/// T-2079: compute the per-event env vec for a given idle change event.
/// Extracted so unit tests can verify the env-var contract without
/// spawning subprocesses. Mirror of T-2072's `snapshot_env_triplet`.
///
/// Returns `Vec<(name, value)>` in stable order so tests can assert
/// positionally. All values are stringified; absent fields render as
/// `"-"` (role) or `""` (capabilities) — matching the `--watch` table
/// rendering's "no value" convention.
pub(crate) fn fire_idle_notify_env(
    ev: &IdleChangeEvent,
    now_secs: u64,
) -> Vec<(&'static str, String)> {
    let kind = match ev.kind {
        IdleChangeKind::New => "new",
        IdleChangeKind::Removed => "removed",
    };
    let role = ev.snap.role.clone().unwrap_or_else(|| "-".to_string());
    let caps = ev.snap.capabilities.join(",");
    vec![
        ("TERMLINK_IDLE_AGENT_ID", ev.agent_id.clone()),
        ("TERMLINK_IDLE_CHANGE_KIND", kind.to_string()),
        ("TERMLINK_IDLE_TS", crate::manifest::secs_to_rfc3339(now_secs)),
        ("TERMLINK_IDLE_ROLE", role),
        ("TERMLINK_IDLE_CAPABILITIES", caps),
        (
            "TERMLINK_IDLE_LAST_HEARTBEAT_MS",
            ev.snap.last_heartbeat_ms.to_string(),
        ),
    ]
}

/// T-2079: spawn the operator-provided notify command for one event,
/// fire-and-forget. Mirror of T-2072's `fire_claim_notify`. Drops the
/// child handle immediately — we don't wait, we don't reap, we don't
/// care if it succeeds. Hanging scripts cannot wedge the watch loop;
/// command-not-found returns an error that we log to stderr but do not
/// propagate.
fn fire_idle_notify(cmd: &str, ev: &IdleChangeEvent, now_secs: u64) {
    let env = fire_idle_notify_env(ev, now_secs);
    let mut command = tokio::process::Command::new("sh");
    command.arg("-c").arg(cmd);
    for (k, v) in env {
        command.env(k, v);
    }
    // stdin from /dev/null so the child can't hold a TTY open + accidentally
    // steal input from the watch user.
    command.stdin(std::process::Stdio::null());
    // Detach: kill_on_drop=false so the child outlives us — operator's
    // pager/Slack-post takes as long as it takes.
    command.kill_on_drop(false);
    match command.spawn() {
        Ok(child) => {
            // Drop the handle: fire-and-forget. The OS will reap the
            // child when it exits (we don't await).
            drop(child);
        }
        Err(e) => {
            // command-not-found / fork failure / etc — log but continue.
            eprintln!("# notify spawn failed: {e}");
        }
    }
}

/// T-2080: render one NDJSON line for an idle change event. Pure
/// function — caller is responsible for the IO. Mirror of T-2073's
/// `render_claim_log_line`. Schema is flat (no nested objects) so a
/// jq pipeline can grep on any field with a single `select(.x==y)`.
pub(crate) fn render_idle_log_line(ev: &IdleChangeEvent, now_secs: u64) -> String {
    let kind = match ev.kind {
        IdleChangeKind::New => "new",
        IdleChangeKind::Removed => "removed",
    };
    let ts = crate::manifest::secs_to_rfc3339(now_secs);
    // serde_json::to_string emits a single-line JSON object — no internal
    // newlines, no pretty-printing. Caller appends '\n' to delimit lines
    // in the NDJSON file.
    let obj = serde_json::json!({
        "ts": ts,
        "agent_id": ev.agent_id,
        "kind": kind,
        "role": ev.snap.role,
        "capabilities": ev.snap.capabilities,
        "last_heartbeat_ms": ev.snap.last_heartbeat_ms,
    });
    serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string())
}

/// T-2080: best-effort append of one log line. Mirror of T-2073's
/// `append_claim_log_line`. Parent directory auto-created; permission
/// or disk-full errors print a one-line stderr warning and return so
/// the watch loop continues. The watch must NEVER crash because the
/// audit trail can't be written — that would silently kill observability.
fn append_idle_log_line(path: &std::path::Path, ev: &IdleChangeEvent, now_secs: u64) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!(
                    "# find-idle log: failed to create parent dir {}: {e}",
                    parent.display()
                );
                return;
            }
        }
    }
    let mut line = render_idle_log_line(ev, now_secs);
    line.push('\n');
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(mut f) => {
            use std::io::Write;
            if let Err(e) = f.write_all(line.as_bytes()) {
                eprintln!(
                    "# find-idle log: write failed for {}: {e}",
                    path.display()
                );
            }
        }
        Err(e) => {
            eprintln!(
                "# find-idle log: open failed for {}: {e}",
                path.display()
            );
        }
    }
}

pub(crate) async fn cmd_agent_find_idle(
    role: Option<&str>,
    capabilities: &[String],
    limit: Option<u32>,
    json_output: bool,
    watch: Option<u64>,
    notify: Option<&str>,
    log: Option<&std::path::Path>,
) -> Result<()> {
    let sock_path = termlink_hub::server::hub_socket_path();
    if !sock_path.exists() {
        if json_output {
            println!("{}", json!({"ok": false, "error": "hub not running"}));
            std::process::exit(1);
        }
        return Err(anyhow!(
            "Hub is not running (no socket at {})",
            sock_path.display()
        ));
    }
    let addr = TransportAddr::unix(sock_path);

    // Build the param object once — same shape for one-shot and watch.
    let mut params_template = json!({});
    if let Some(r) = role {
        params_template["role"] = json!(r);
    }
    if !capabilities.is_empty() {
        params_template["capabilities"] = json!(capabilities);
    }
    if let Some(n) = limit {
        params_template["limit"] = json!(n);
    }

    // T-2078: --watch + --json are guarded at the clap layer via
    // conflicts_with — clap rejects before we get here. Belt-and-braces
    // assertion in case the variant is ever constructed by something
    // other than clap parsing.
    if watch.is_some() && json_output {
        anyhow::bail!(
            "--watch and --json are incompatible: --watch streams re-rendered \
             text frames; --json is one-shot. Pick one."
        );
    }

    if let Some(interval_raw) = watch {
        // T-2078: 5..=3600 clamp mirrors claims-summary --watch (T-2041)
        // — the idle roster updates at heartbeat cadence (~30s) so
        // sub-5s polling is pure noise.
        let interval = interval_raw.clamp(5, 3600);
        // T-2078: diff scaffolding for Slice 2 --notify. Prior tick state
        // kept across iterations; None on the first tick = baseline.
        let mut prior_state: Option<std::collections::BTreeMap<String, IdleSnapshot>> = None;
        loop {
            print!("\x1b[2J\x1b[H");
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let now_ms = now_secs as i64 * 1000;
            let now_str = crate::manifest::secs_to_rfc3339(now_secs);
            println!(
                "# agent find-idle --watch | interval={}s | {}",
                interval, now_str
            );
            let current_state: Option<std::collections::BTreeMap<String, IdleSnapshot>>;
            match client::rpc_call_addr(
                &addr,
                method::AGENT_FIND_IDLE,
                params_template.clone(),
            )
            .await
            {
                Ok(resp) => match client::unwrap_result(resp) {
                    Ok(result) => {
                        let snap = parse_idle_result(&result);
                        render_idle_table(&snap, now_ms);
                        current_state = Some(snap);
                    }
                    Err(e) => {
                        println!("# fetch error (will retry on next tick): {e}");
                        current_state = None;
                    }
                },
                Err(e) => {
                    println!("# fetch error (will retry on next tick): {e}");
                    current_state = None;
                }
            }
            // T-2078 + T-2079: diff against prior_state and dispatch
            // --notify per event. Only diff when BOTH prior_state and
            // current_state are Some (skip baseline + skip fetch-fail
            // ticks — otherwise we'd synthesize spurious `removed`
            // events for every agent on a transient fetch failure).
            if let (Some(prev), Some(curr)) = (prior_state.as_ref(), current_state.as_ref()) {
                let events = diff_idle_sets(prev, curr);
                for ev in &events {
                    if let Some(cmd) = notify {
                        fire_idle_notify(cmd, ev, now_secs);
                    }
                    if let Some(path) = log {
                        append_idle_log_line(path, ev, now_secs);
                    }
                }
            }
            if current_state.is_some() {
                prior_state = current_state;
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    }

    let resp = client::rpc_call_addr(&addr, method::AGENT_FIND_IDLE, params_template.clone())
        .await
        .context("agent.find_idle RPC failed")?;
    let result = client::unwrap_result(resp)
        .map_err(|e| anyhow!("Hub returned error for agent.find_idle: {e}"))?;

    let idle: Vec<Value> = result["idle"].as_array().cloned().unwrap_or_default();

    if json_output {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if idle.is_empty() {
        println!("(no idle agents matching filter)");
        return Ok(());
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let snap = parse_idle_result(&result);
    render_idle_table(&snap, now_ms);
    Ok(())
}

/// T-2081 (substrate primitive #2 obs arc Slice 4 — mirror of T-2074
/// `claim_log_path`): default log path for the find-idle audit trail.
/// Resolves `~/.termlink/find-idle.log`. Falls back to
/// `./.termlink/find-idle.log` when `$HOME` is unset (rare; CI / docker
/// minimal images) so the helper never panics — the caller is still
/// free to override via `--log <PATH>`.
pub(crate) fn find_idle_log_path() -> std::path::PathBuf {
    match std::env::var_os("HOME") {
        Some(home) => std::path::PathBuf::from(home)
            .join(".termlink")
            .join("find-idle.log"),
        None => std::path::PathBuf::from(".termlink").join("find-idle.log"),
    }
}

/// T-2081: per-agent aggregate counters for `find-idle-history`. Idle is
/// a binary state (no `transition` kind — see T-2078 design note), so we
/// only count `new` and `removed` events. Mirror of T-2074's
/// `ClaimsHistoryAgg` (drops `transitions`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FindIdleHistoryAgg {
    pub new_events: u64,
    pub removed_events: u64,
}

/// T-2081: pure helper — parse NDJSON log text into `(entries,
/// malformed_count)`. Each non-empty line that fails JSON parse OR lacks
/// required fields (`ts`, `agent_id`, `kind`) is skipped and counted;
/// the rest are returned in source order. Time-window filter
/// (`cutoff_secs`) and agent_id exact-match filter applied during the
/// walk. Mirror of T-2074's `parse_claims_log`.
///
/// `cutoff_secs` is "skip any entry whose ts is older than this Unix
/// epoch seconds". Caller computes `now - since_days * 86400`. Agent-id
/// filter `None` means "all agents".
pub(crate) fn parse_find_idle_log(
    text: &str,
    cutoff_secs: i64,
    agent_id_filter: Option<&str>,
    kind_filter: Option<&str>,
) -> (Vec<serde_json::Value>, usize) {
    let mut entries = Vec::new();
    let mut malformed = 0usize;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                malformed += 1;
                continue;
            }
        };
        let ts_str = match v.get("ts").and_then(|t| t.as_str()) {
            Some(s) => s,
            None => {
                malformed += 1;
                continue;
            }
        };
        let agent_id = match v.get("agent_id").and_then(|t| t.as_str()) {
            Some(s) => s,
            None => {
                malformed += 1;
                continue;
            }
        };
        let kind = match v.get("kind").and_then(|k| k.as_str()) {
            Some(s) => s,
            None => {
                malformed += 1;
                continue;
            }
        };
        if let Some(want) = agent_id_filter {
            if agent_id != want {
                continue;
            }
        }
        // T-2208: --kind filter mirrors queue-history. Permissive: any
        // value other than the actual emitted kinds (`new`, `removed`)
        // just yields zero matches without panic.
        if let Some(want_kind) = kind_filter {
            if kind != want_kind {
                continue;
            }
        }
        let entry_secs = rfc3339_to_unix_secs_local(ts_str);
        if entry_secs < cutoff_secs {
            continue;
        }
        entries.push(v);
    }
    (entries, malformed)
}

/// T-2081: stdlib-only RFC3339→epoch parser. Local copy of the same
/// helper used by T-2074 `channel::parse_claims_log` and T-2068
/// `remote::cmd_fleet_history` — kept module-private by deliberate
/// convention (stdlib-only across the crate; duplicating ~30 lines is
/// cheaper than introducing a cross-module dependency). Returns 0 on
/// any parse error (caller treats 0 as "very old").
fn rfc3339_to_unix_secs_local(ts: &str) -> i64 {
    if ts.len() < 20 || !ts.ends_with('Z') {
        return 0;
    }
    let bytes = ts.as_bytes();
    let parse_u = |start: usize, len: usize| -> Option<u32> {
        std::str::from_utf8(&bytes[start..start + len])
            .ok()?
            .parse()
            .ok()
    };
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) = (
        parse_u(0, 4),
        parse_u(5, 2),
        parse_u(8, 2),
        parse_u(11, 2),
        parse_u(14, 2),
        parse_u(17, 2),
    ) else {
        return 0;
    };
    let y = y as i64;
    let mo = mo as i64;
    let d = d as i64;
    let y_shift = if mo <= 2 { y - 1 } else { y };
    let era = if y_shift >= 0 {
        y_shift / 400
    } else {
        (y_shift - 399) / 400
    };
    let yoe = y_shift - era * 400;
    let mp = if mo > 2 { mo - 3 } else { mo + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days * 86_400 + (h as i64) * 3600 + (mi as i64) * 60 + s as i64
}

/// T-2081: pure helper — aggregate parsed entries into per-agent
/// counters. `BTreeMap` keeps iteration order stable for the human
/// footer (alphabetical agent_ids → reproducible test assertions).
/// Mirror of T-2074's `aggregate_claims_entries` (sans `transitions`).
pub(crate) fn aggregate_find_idle_entries(
    entries: &[serde_json::Value],
) -> std::collections::BTreeMap<String, FindIdleHistoryAgg> {
    let mut out: std::collections::BTreeMap<String, FindIdleHistoryAgg> =
        std::collections::BTreeMap::new();
    for e in entries {
        let agent_id = match e.get("agent_id").and_then(|t| t.as_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let kind = match e.get("kind").and_then(|k| k.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let agg = out.entry(agent_id).or_default();
        match kind {
            "new" => agg.new_events += 1,
            "removed" => agg.removed_events += 1,
            _ => {}
        }
    }
    out
}

/// T-2081: render one parsed entry as a single human-readable line.
/// Format chosen so the eye can scan a 50-line dump and pick out the
/// kind/agent_id columns. Mirror of T-2074's `render_claim_history_line`.
fn render_find_idle_history_line(e: &serde_json::Value) -> String {
    let ts = e.get("ts").and_then(|t| t.as_str()).unwrap_or("-");
    let agent_id = e.get("agent_id").and_then(|t| t.as_str()).unwrap_or("-");
    let kind = e.get("kind").and_then(|t| t.as_str()).unwrap_or("-");
    let role = match e.get("role") {
        Some(v) if v.is_null() => "-".to_string(),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => "-".to_string(),
    };
    let caps = match e.get("capabilities") {
        Some(serde_json::Value::Array(arr)) if !arr.is_empty() => arr
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(","),
        _ => "-".to_string(),
    };
    let hb = match e.get("last_heartbeat_ms") {
        Some(serde_json::Value::Number(n)) => format!("{}ms", n),
        _ => "-".to_string(),
    };
    format!(
        "{}  {}  {}  role={}  caps={}  last_heartbeat={}",
        ts, agent_id, kind, role, caps, hb
    )
}

/// T-2081: the `agent find-idle-history` command implementation.
/// Read-only: walks the log file, applies filters, renders. Never auths
/// or talks to a hub. Missing log file → operator hint pointing back at
/// the writer (`agent find-idle --watch --log`). Mirror of T-2074's
/// `cmd_channel_claims_history`.
pub(crate) async fn cmd_agent_find_idle_history(
    since_days: u32,
    agent_id_filter: Option<&str>,
    kind_filter: Option<&str>,
    log_override: Option<&std::path::Path>,
    json_out: bool,
) -> anyhow::Result<()> {
    let since_days = since_days.clamp(1, 365);
    let path: std::path::PathBuf = log_override
        .map(|p| p.to_path_buf())
        .unwrap_or_else(find_idle_log_path);
    let path_str = path.display().to_string();
    let text = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if json_out {
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "entries": [],
                        "summary": {
                            "total": 0,
                            "per_agent": {},
                            "since_days": since_days,
                            "agent_id_filter": agent_id_filter,
                            "kind_filter": kind_filter,
                            "malformed_lines_skipped": 0,
                            "log_path": path_str,
                            "note": "log file does not exist yet",
                        }
                    })
                );
                return Ok(());
            }
            println!(
                "(no log file at {} — write events first with `agent find-idle --watch --log {}`)",
                path_str, path_str
            );
            return Ok(());
        }
        Err(e) => anyhow::bail!("find-idle-history: read {:?} failed: {e}", path),
    };
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let cutoff_secs = now_secs - (since_days as i64) * 86_400;
    let (entries, malformed) =
        parse_find_idle_log(&text, cutoff_secs, agent_id_filter, kind_filter);
    let agg = aggregate_find_idle_entries(&entries);
    if json_out {
        let per_agent: serde_json::Map<String, serde_json::Value> = agg
            .iter()
            .map(|(id, a)| {
                (
                    id.clone(),
                    serde_json::json!({
                        // T-2254: `*_events` keys for CLI↔MCP parity (matches
                        // termlink_agent_find_idle_history + sibling queue-history).
                        "new_events": a.new_events,
                        "removed_events": a.removed_events,
                    }),
                )
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "entries": entries,
                "summary": {
                    "total": entries.len(),
                    "per_agent": per_agent,
                    "since_days": since_days,
                    "agent_id_filter": agent_id_filter,
                    "kind_filter": kind_filter,
                    "malformed_lines_skipped": malformed,
                    "log_path": path_str,
                }
            })
        );
        return Ok(());
    }
    if entries.is_empty() {
        let id_clause = agent_id_filter
            .map(|t| format!(" agent_id={:?}", t))
            .unwrap_or_default();
        let kind_clause = kind_filter
            .map(|t| format!(" kind={:?}", t))
            .unwrap_or_default();
        println!(
            "(no entries in last {} day(s){}{} — log: {})",
            since_days, id_clause, kind_clause, path_str
        );
        if malformed > 0 {
            println!("({} malformed line(s) skipped)", malformed);
        }
        return Ok(());
    }
    for e in &entries {
        println!("{}", render_find_idle_history_line(e));
    }
    println!();
    println!(
        "Aggregate (since {} day(s), {} entries{}):",
        since_days,
        entries.len(),
        if malformed > 0 {
            format!(", {} malformed lines skipped", malformed)
        } else {
            String::new()
        }
    );
    for (id, a) in &agg {
        println!("  {}  {} new  {} removed", id, a.new_events, a.removed_events);
    }
    println!("(log: {})", path_str);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(hb_ms: i64, role: Option<&str>, caps: &[&str]) -> IdleSnapshot {
        IdleSnapshot {
            last_heartbeat_ms: hb_ms,
            role: role.map(String::from),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn map(entries: &[(&str, IdleSnapshot)]) -> std::collections::BTreeMap<String, IdleSnapshot> {
        let mut m = std::collections::BTreeMap::new();
        for (k, v) in entries {
            m.insert((*k).to_string(), v.clone());
        }
        m
    }

    #[test]
    fn find_idle_watch_diff_detects_new_agents() {
        // alpha was idle, beta is now also idle → 1 New event for beta.
        let prev = map(&[("alpha", snap(1_000, Some("claude-code"), &["rust"]))]);
        let curr = map(&[
            ("alpha", snap(1_500, Some("claude-code"), &["rust"])),
            ("beta", snap(1_400, Some("claude-code"), &["docs"])),
        ]);
        let events = diff_idle_sets(&prev, &curr);
        assert_eq!(events.len(), 1, "only beta is new");
        assert_eq!(events[0].agent_id, "beta");
        assert_eq!(events[0].kind, IdleChangeKind::New);
        assert_eq!(events[0].snap.role.as_deref(), Some("claude-code"));
    }

    #[test]
    fn find_idle_watch_diff_detects_removed_agents() {
        // alpha and beta were idle, now only alpha → 1 Removed event for beta.
        let prev = map(&[
            ("alpha", snap(1_000, Some("claude-code"), &[])),
            ("beta", snap(1_000, Some("claude-code"), &["docs"])),
        ]);
        let curr = map(&[("alpha", snap(1_500, Some("claude-code"), &[]))]);
        let events = diff_idle_sets(&prev, &curr);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].agent_id, "beta");
        assert_eq!(events[0].kind, IdleChangeKind::Removed);
        // Removed event carries the LAST-known prior snapshot so
        // downstream --notify scripts have something to render.
        assert_eq!(events[0].snap.role.as_deref(), Some("claude-code"));
    }

    #[test]
    fn find_idle_watch_diff_re_heartbeat_is_not_an_event() {
        // Same set of agents in both ticks, only heartbeat advances → 0
        // events. Idle is binary: still idle = no state change.
        let prev = map(&[("alpha", snap(1_000, Some("claude-code"), &["rust"]))]);
        let curr = map(&[("alpha", snap(1_500, Some("claude-code"), &["rust"]))]);
        let events = diff_idle_sets(&prev, &curr);
        assert!(events.is_empty(), "re-heartbeat is not an event");
    }

    #[test]
    fn find_idle_watch_diff_handles_both_directions() {
        // Mixed: beta arrived idle, gamma went busy. Expect 1 New + 1 Removed.
        let prev = map(&[
            ("alpha", snap(1_000, None, &[])),
            ("gamma", snap(1_000, Some("claude-code"), &["test"])),
        ]);
        let curr = map(&[
            ("alpha", snap(1_500, None, &[])),
            ("beta", snap(1_500, Some("claude-code"), &[])),
        ]);
        let events = diff_idle_sets(&prev, &curr);
        assert_eq!(events.len(), 2);
        let new_count = events.iter().filter(|e| e.kind == IdleChangeKind::New).count();
        let rm_count = events.iter().filter(|e| e.kind == IdleChangeKind::Removed).count();
        assert_eq!(new_count, 1);
        assert_eq!(rm_count, 1);
    }

    // ---- T-2079 --notify env-vec helper tests ---------------------------

    #[test]
    fn find_idle_notify_env_for_new_event() {
        let ev = IdleChangeEvent {
            agent_id: "claude-alpha".to_string(),
            kind: IdleChangeKind::New,
            snap: snap(1_700_000_000_000, Some("claude-code"), &["rust", "docs"]),
        };
        // 1_700_000_000 unix = 2023-11-14T22:13:20Z; we just check the
        // shape + kind, not the exact timestamp string (manifest renderer
        // is tested elsewhere).
        let env = fire_idle_notify_env(&ev, 1_700_000_000);
        let by_key: std::collections::HashMap<&str, String> =
            env.iter().map(|(k, v)| (*k, v.clone())).collect();
        assert_eq!(by_key["TERMLINK_IDLE_AGENT_ID"], "claude-alpha");
        assert_eq!(by_key["TERMLINK_IDLE_CHANGE_KIND"], "new");
        assert_eq!(by_key["TERMLINK_IDLE_ROLE"], "claude-code");
        assert_eq!(by_key["TERMLINK_IDLE_CAPABILITIES"], "rust,docs");
        assert_eq!(
            by_key["TERMLINK_IDLE_LAST_HEARTBEAT_MS"],
            "1700000000000"
        );
        // RFC3339 should be present and look like one (Z suffix).
        assert!(
            by_key["TERMLINK_IDLE_TS"].ends_with('Z'),
            "TS should be RFC3339 UTC: {}",
            by_key["TERMLINK_IDLE_TS"]
        );
    }

    #[test]
    fn find_idle_notify_env_for_removed_event() {
        // Removed event with no role / no capabilities → "-" / "".
        let ev = IdleChangeEvent {
            agent_id: "beta".to_string(),
            kind: IdleChangeKind::Removed,
            snap: snap(1_000, None, &[]),
        };
        let env = fire_idle_notify_env(&ev, 1_700_000_000);
        let by_key: std::collections::HashMap<&str, String> =
            env.iter().map(|(k, v)| (*k, v.clone())).collect();
        assert_eq!(by_key["TERMLINK_IDLE_CHANGE_KIND"], "removed");
        assert_eq!(
            by_key["TERMLINK_IDLE_ROLE"], "-",
            "missing role renders as '-'"
        );
        assert_eq!(
            by_key["TERMLINK_IDLE_CAPABILITIES"], "",
            "empty caps render as empty string"
        );
    }

    // ---- T-2080 --log NDJSON line tests -------------------------------

    #[test]
    fn find_idle_log_line_is_single_line_jq_friendly() {
        // No internal newlines — `jq -c 'select(...)' file` MUST be able
        // to parse one line at a time.
        let ev = IdleChangeEvent {
            agent_id: "claude-alpha".to_string(),
            kind: IdleChangeKind::New,
            snap: snap(1_700_000_000_000, Some("claude-code"), &["rust", "docs"]),
        };
        let line = render_idle_log_line(&ev, 1_700_000_000);
        assert!(!line.contains('\n'), "line must not contain newline");
        // Round-trip: must parse as valid JSON.
        let v: serde_json::Value = serde_json::from_str(&line)
            .expect("rendered line must be valid JSON");
        assert_eq!(v["agent_id"], "claude-alpha");
        assert_eq!(v["kind"], "new");
        assert_eq!(v["role"], "claude-code");
        assert_eq!(v["last_heartbeat_ms"], 1_700_000_000_000_i64);
        let caps = v["capabilities"].as_array().expect("caps is array");
        assert_eq!(caps.len(), 2);
        assert_eq!(caps[0], "rust");
        assert_eq!(caps[1], "docs");
        assert!(v["ts"].as_str().unwrap().ends_with('Z'));
    }

    #[test]
    fn find_idle_log_line_serializes_kind_for_removed() {
        // The kind field MUST distinguish new/removed so jq can filter.
        let ev = IdleChangeEvent {
            agent_id: "beta".to_string(),
            kind: IdleChangeKind::Removed,
            snap: snap(1_000, None, &[]),
        };
        let line = render_idle_log_line(&ev, 1_700_000_000);
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["kind"], "removed");
        // Absent role serializes as null (NOT "-"); the "-" rendering is
        // for terminal display, NDJSON keeps it null so jq filters work
        // (`select(.role==null)`).
        assert!(v["role"].is_null(), "missing role must be JSON null, got {}", v["role"]);
        let caps = v["capabilities"].as_array().expect("caps is array");
        assert!(caps.is_empty(), "empty capabilities renders as []");
    }

    #[test]
    fn find_idle_watch_parses_empty_and_populated_results() {
        // Empty: missing idle array.
        let r = serde_json::json!({"ok": true});
        let s = parse_idle_result(&r);
        assert!(s.is_empty());

        // Populated: full shape.
        let r = serde_json::json!({
            "ok": true,
            "idle": [
                {"agent_id": "a", "last_heartbeat_ms": 100, "role": "claude-code", "capabilities": ["rust", "docs"]},
                {"agent_id": "b", "last_heartbeat_ms": 200}
            ]
        });
        let s = parse_idle_result(&r);
        assert_eq!(s.len(), 2);
        let a = s.get("a").expect("a present");
        assert_eq!(a.role.as_deref(), Some("claude-code"));
        assert_eq!(a.capabilities, vec!["rust".to_string(), "docs".to_string()]);
        let b = s.get("b").expect("b present");
        assert_eq!(b.last_heartbeat_ms, 200);
        assert!(b.role.is_none());
        assert!(b.capabilities.is_empty());
    }

    // ───────────────────────────────────────────────────────────────────
    // T-2081 — find-idle-history helper tests (mirror of T-2074 claims-
    // history tests). Idle is binary so we test new/removed kinds only
    // (no transition).
    // ───────────────────────────────────────────────────────────────────

    #[test]
    fn find_idle_history_parse_skips_malformed_and_counts() {
        // Mix of valid + invalid lines; helper returns only valid in
        // source order and counts the rest in `malformed`.
        let text = "\
{\"ts\":\"2030-01-01T00:00:00Z\",\"agent_id\":\"a\",\"kind\":\"new\"}\n\
not-json\n\
{\"ts\":\"2030-01-01T00:01:00Z\",\"agent_id\":\"b\"}\n\
{\"agent_id\":\"c\",\"kind\":\"new\"}\n\
\n\
{\"ts\":\"2030-01-01T00:02:00Z\",\"agent_id\":\"d\",\"kind\":\"removed\"}\n";
        // cutoff=0 → no time filter applied.
        let (entries, malformed) = parse_find_idle_log(text, 0, None, None);
        assert_eq!(entries.len(), 2, "only the two complete lines survive");
        assert_eq!(malformed, 3, "not-json + missing-kind + missing-ts all count");
        assert_eq!(entries[0]["agent_id"], "a");
        assert_eq!(entries[1]["agent_id"], "d");
    }

    #[test]
    fn find_idle_history_parse_applies_cutoff() {
        let text = "\
{\"ts\":\"2025-01-01T00:00:00Z\",\"agent_id\":\"old\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:00:00Z\",\"agent_id\":\"fresh\",\"kind\":\"new\"}\n";
        // 2026-01-01T00:00:00Z epoch = 1_767_225_600. Cut at 2027 → drop the old.
        let cutoff = rfc3339_to_unix_secs_local("2027-01-01T00:00:00Z");
        let (entries, _) = parse_find_idle_log(text, cutoff, None, None);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["agent_id"], "fresh");
    }

    #[test]
    fn find_idle_history_parse_applies_agent_id_filter() {
        let text = "\
{\"ts\":\"2030-01-01T00:00:00Z\",\"agent_id\":\"wanted\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:01:00Z\",\"agent_id\":\"other\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:02:00Z\",\"agent_id\":\"wanted\",\"kind\":\"removed\"}\n";
        let (entries, _) = parse_find_idle_log(text, 0, Some("wanted"), None);
        assert_eq!(entries.len(), 2);
        for e in &entries {
            assert_eq!(e["agent_id"], "wanted");
        }
    }

    #[test]
    fn find_idle_history_parse_applies_kind_filter() {
        // T-2208: --kind filter drops entries whose `kind` field doesn't match.
        let text = "\
{\"ts\":\"2030-01-01T00:00:00Z\",\"agent_id\":\"a\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:01:00Z\",\"agent_id\":\"a\",\"kind\":\"removed\"}\n\
{\"ts\":\"2030-01-01T00:02:00Z\",\"agent_id\":\"b\",\"kind\":\"new\"}\n";
        let (only_new, _) = parse_find_idle_log(text, 0, None, Some("new"));
        assert_eq!(only_new.len(), 2);
        for e in &only_new {
            assert_eq!(e["kind"], "new");
        }
        let (only_removed, _) = parse_find_idle_log(text, 0, None, Some("removed"));
        assert_eq!(only_removed.len(), 1);
        assert_eq!(only_removed[0]["agent_id"], "a");
        // Permissive — unknown kind yields zero matches without panic.
        let (zero, _) = parse_find_idle_log(text, 0, None, Some("transition"));
        assert_eq!(zero.len(), 0);
    }

    #[test]
    fn find_idle_history_aggregate_counts_kinds() {
        let text = "\
{\"ts\":\"2030-01-01T00:00:00Z\",\"agent_id\":\"a\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:01:00Z\",\"agent_id\":\"a\",\"kind\":\"removed\"}\n\
{\"ts\":\"2030-01-01T00:02:00Z\",\"agent_id\":\"a\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:03:00Z\",\"agent_id\":\"b\",\"kind\":\"new\"}\n\
{\"ts\":\"2030-01-01T00:04:00Z\",\"agent_id\":\"b\",\"kind\":\"unknown_kind\"}\n";
        let (entries, _) = parse_find_idle_log(text, 0, None, None);
        let agg = aggregate_find_idle_entries(&entries);
        let a = agg.get("a").expect("a present");
        assert_eq!(a.new_events, 2, "a flipped from busy→idle twice");
        assert_eq!(a.removed_events, 1);
        let b = agg.get("b").expect("b present");
        assert_eq!(b.new_events, 1);
        // Unknown kind is silently dropped (forward-compatible parser).
        assert_eq!(b.removed_events, 0);
    }

    #[test]
    fn find_idle_history_render_line_handles_null_role_and_empty_caps() {
        // Same NDJSON shape T-2080's `render_idle_log_line` writes:
        // role can be null, capabilities can be [].
        let e = serde_json::json!({
            "ts": "2030-01-01T00:00:00Z",
            "agent_id": "x",
            "kind": "removed",
            "role": null,
            "capabilities": [],
            "last_heartbeat_ms": 1234,
        });
        let line = render_find_idle_history_line(&e);
        assert!(line.contains("2030-01-01T00:00:00Z"));
        assert!(line.contains("  x  "));
        assert!(line.contains("removed"));
        assert!(line.contains("role=-"));
        assert!(line.contains("caps=-"));
        assert!(line.contains("last_heartbeat=1234ms"));
    }
}
