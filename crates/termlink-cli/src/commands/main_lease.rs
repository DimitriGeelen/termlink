//! T-3654 (OD-18 / R-67): `termlink main claim|renew|release|status|am-i|pin`.
//!
//! One answer per SESSION for hooks: exit 0 main, 1 not-main (the holder is printed),
//! 2 unknown (hub down, no project, no session, any error). Unknown is never main.

use std::io::Read as _;

use serde_json::Value;
use termlink_session::main_lease::{Holder, LeaseClient, Outcome, Timing};

/// Where a verb gets its session/project/hub from.
pub(crate) struct MainArgs {
    pub project: Option<String>,
    pub hub: Option<String>,
    pub session: Option<String>,
    /// Read Claude Code's hook input JSON from stdin and take its `session_id`.
    pub session_from_stdin: bool,
    pub agent: Option<String>,
    pub pid: Option<u32>,
    pub obligation_topics: Vec<String>,
    pub json: bool,
}

pub(crate) enum MainVerb {
    Claim,
    Renew,
    Release,
    Status,
    AmI,
    Pin { session: Option<String>, clear: bool, reason: String, by: Option<String> },
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// `--session`, else the hook input's `session_id`, else `$CLAUDE_SESSION_ID`, else `$TERMLINK_SESSION_ID`.
fn resolve_session(a: &MainArgs) -> Option<String> {
    if let Some(s) = a.session.clone().filter(|s| !s.trim().is_empty()) {
        return Some(s.trim().to_string());
    }
    if a.session_from_stdin {
        let mut buf = String::new();
        let _ = std::io::stdin().take(4 * 1024 * 1024).read_to_string(&mut buf);
        if let Some(s) = serde_json::from_str::<Value>(&buf).ok().and_then(|v| v.get("session_id").and_then(Value::as_str).map(str::to_string)) {
            return Some(s);
        }
    }
    env("CLAUDE_SESSION_ID").or_else(|| env("TERMLINK_SESSION_ID"))
}

/// `--project`, else `$TERMLINK_MAIN_PROJECT`, else `.framework.yaml`'s project_name above the cwd.
pub(crate) fn resolve_project(p: Option<&str>) -> Option<String> {
    p.map(str::to_string)
        .or_else(|| env("TERMLINK_MAIN_PROJECT"))
        .or_else(|| std::env::current_dir().ok().and_then(|d| super::channel::resolve_project_name_from(&d)))
        .filter(|s| !s.trim().is_empty())
}

fn hostname() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: buf is valid for its length; gethostname NUL-terminates within it on success.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if rc != 0 {
        return String::new();
    }
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

fn print(o: &Outcome, json: bool) -> i32 {
    if json {
        println!("{}", o.to_json());
    } else {
        println!("{}", o.line());
        if let Some(l) = &o.lost
            && let Some(to) = l.get("to")
        {
            println!(
                "lost main at generation {} to {}",
                l.get("generation").and_then(Value::as_u64).unwrap_or(0),
                to.get("session").and_then(Value::as_str).unwrap_or("?")
            );
        }
        if let Some(p) = o.view.as_ref().and_then(|v| v.pinned.as_ref()) {
            println!("pinned by operator: {p}");
        }
    }
    o.verdict.exit_code()
}

fn unknown(session: Option<&str>, why: String, json: bool) -> i32 {
    print(&Outcome::unknown(session, why), json)
}

/// Run one verb; returns the process exit code.
pub(crate) async fn cmd_main(verb: MainVerb, a: MainArgs) -> i32 {
    let json = a.json;
    let session = resolve_session(&a);
    let Some(project) = resolve_project(a.project.as_deref()) else {
        return unknown(session.as_deref(), "no project: pass --project, set TERMLINK_MAIN_PROJECT, or run inside a project with .framework.yaml".into(), json);
    };
    let addr = match super::channel::hub_socket(a.hub.as_deref()) {
        Ok(x) => x,
        Err(e) => return unknown(session.as_deref(), format!("hub unreachable: {e}"), json),
    };
    let auth = match super::channel::claim_auth(&addr) {
        Ok(x) => x,
        Err(e) => return unknown(session.as_deref(), format!("no hub credential: {e}"), json),
    };
    let writes = matches!(verb, MainVerb::Claim | MainVerb::Renew | MainVerb::Release | MainVerb::Pin { .. });
    let identity = if writes {
        match super::channel::load_identity_or_create() {
            Ok(i) => Some(i),
            Err(e) => return unknown(session.as_deref(), format!("no signing identity for the lease record: {e}"), json),
        }
    } else {
        None
    };
    let mut obligation_topics = a.obligation_topics.clone();
    if let Some(e) = env("TERMLINK_MAIN_OBLIGATION_TOPICS") {
        obligation_topics.extend(e.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
    }
    let client = LeaseClient { addr, auth, identity, timing: Timing::from_env(), obligation_topics };
    let me = |s: &str| Holder {
        session: s.to_string(),
        agent: a.agent.clone().or_else(|| env("TERMLINK_AGENT_ID")).unwrap_or_default(),
        host: hostname(),
        pid: a.pid.unwrap_or_else(std::os::unix::process::parent_id),
    };
    let need_session = || "no session: pass --session, --session-from-stdin, or set CLAUDE_SESSION_ID / TERMLINK_SESSION_ID".to_string();
    let o = match verb {
        MainVerb::Status => client.status(&project, session.as_deref()).await,
        MainVerb::AmI => match &session {
            Some(s) => client.status(&project, Some(s)).await,
            None => return unknown(None, need_session(), json),
        },
        MainVerb::Claim | MainVerb::Renew | MainVerb::Release => {
            let Some(s) = &session else { return unknown(None, need_session(), json) };
            let h = me(s);
            match verb {
                MainVerb::Claim => client.claim(&project, &h).await,
                MainVerb::Renew => client.renew(&project, &h).await,
                _ => client.release(&project, &h).await,
            }
        }
        MainVerb::Pin { session: target, clear, reason, by } => {
            if target.is_none() && !clear {
                return unknown(None, "pin needs --session <id> or --clear".into(), json);
            }
            let by = by.or_else(|| env("USER")).unwrap_or_else(|| "operator".into());
            let o = client.pin(&project, if clear { None } else { target.as_deref() }, &by, &reason).await;
            // A recorded pin is success (exit 0) even though the caller is not "main".
            if o.verdict != termlink_session::main_lease::Verdict::Unknown {
                if json {
                    println!("{}", o.to_json());
                } else {
                    println!("pin recorded: {}", if clear { "cleared".to_string() } else { target.unwrap_or_default() });
                    println!("{}", o.message);
                }
                return 0;
            }
            o
        }
    };
    print(&o, json)
}
