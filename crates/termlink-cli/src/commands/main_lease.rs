//! T-3654 (OD-18 / R-67): `termlink main claim|renew|release|status|am-i|pin`.
//!
//! One answer per SESSION for hooks: exit 0 main, 1 not-main (the holder is printed),
//! 2 unknown (hub down, no project, no session, any error). Unknown is never main.

use std::io::Read as _;

use serde_json::Value;
use termlink_session::main_lease::{ClaimKind, ClaimRequest, DEFAULT_HARNESS, Holder, LeaseClient, Origin, Outcome, Timing};

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
    /// B-25 (T-3694): the claim's origin, basis, hook kind, interactivity and harness.
    pub origin: Option<String>,
    pub basis: Option<String>,
    pub kind: String,
    pub interactive: bool,
    pub harness: Option<String>,
}

pub(crate) enum MainVerb {
    Claim,
    Renew,
    Release,
    Status,
    AmI,
    Pin { session: Option<String>, clear: bool, reason: String, by: Option<String> },
    OriginReporting { off: bool, by: Option<String> },
}

/// B-25: the claim request from the flags. An origin or kind that does not parse is an error
/// (unknown, exit 2), never a guess.
fn claim_request(a: &MainArgs) -> Result<ClaimRequest, String> {
    let kind = ClaimKind::parse(&a.kind).ok_or_else(|| format!("--kind {:?}: expected prompt, session-start or tool", a.kind))?;
    let origin = match a.origin.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some(o) => Some(Origin::parse(o).ok_or_else(|| format!("--origin {o:?}: expected human, automated or unknown"))?),
    };
    let harness = a.harness.clone().or_else(|| env("TERMLINK_MAIN_HARNESS")).unwrap_or_else(|| DEFAULT_HARNESS.to_string());
    Ok(ClaimRequest { kind, origin, basis: a.basis.clone().filter(|b| !b.trim().is_empty()), interactive: Some(a.interactive), harness })
}

fn env(k: &str) -> Option<String> {
    std::env::var(k).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn resolve_session(a: &MainArgs) -> Option<String> {
    let stdin = a.session_from_stdin.then(|| {
        let mut buf = String::new();
        let _ = std::io::stdin().take(4 * 1024 * 1024).read_to_string(&mut buf);
        buf
    });
    pick_session(a.session.as_deref(), stdin.as_deref(), env)
}

/// T-3668: `--session`, else the hook input's non-empty `session_id`, else `$CLAUDE_SESSION_ID`,
/// else `$CLAUDE_CODE_SESSION_ID` (what Claude Code actually exports), else `$TERMLINK_SESSION_ID`
/// ONLY outside Claude Code. Inside Claude Code (`$CLAUDECODE` set) the TermLink session id is a
/// different identity from the session the hooks claim main with, so guessing it would answer
/// "not-main" for the real main, or claim main for an id nobody recognises (PL-373): refuse instead.
fn pick_session(flag: Option<&str>, stdin: Option<&str>, env: impl Fn(&str) -> Option<String>) -> Option<String> {
    let nonempty = |s: &str| Some(s.trim().to_string()).filter(|s| !s.is_empty());
    if let Some(s) = flag.and_then(nonempty) {
        return Some(s);
    }
    if let Some(s) = stdin
        .and_then(|b| serde_json::from_str::<Value>(b).ok())
        .and_then(|v| v.get("session_id").and_then(Value::as_str).and_then(nonempty))
    {
        return Some(s);
    }
    env("CLAUDE_SESSION_ID")
        .or_else(|| env("CLAUDE_CODE_SESSION_ID"))
        .or_else(|| if env("CLAUDECODE").is_some() { None } else { env("TERMLINK_SESSION_ID") })
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
        if let Some(v) = o.view.as_ref() {
            println!(
                "origin: {} | last_human_at: {} | state: {}",
                v.origin.as_deref().unwrap_or("-"),
                v.last_human_at_ms.map_or("-".to_string(), |t| format!("{t} ({}s ago)", v.now_ms.saturating_sub(t) / 1000)),
                v.state
            );
        }
        if let Some(n) = &o.note {
            println!("note: {n}");
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
    // T-3686 (AL-16, FCW-60): first, before any hub contact: a seam outside a test estate.
    let timing = match Timing::from_env() {
        Ok(t) => t,
        Err(e) => return unknown(session.as_deref(), e.to_string(), json),
    };
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
    let req = match claim_request(&a) {
        Ok(r) => r,
        Err(e) => return unknown(session.as_deref(), e, json),
    };
    let writes = matches!(verb, MainVerb::Claim | MainVerb::Renew | MainVerb::Release | MainVerb::Pin { .. } | MainVerb::OriginReporting { .. });
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
    let client = LeaseClient { addr, auth, identity, timing, obligation_topics };
    let me = |s: &str| Holder {
        session: s.to_string(),
        agent: a.agent.clone().or_else(|| env("TERMLINK_AGENT_ID")).unwrap_or_default(),
        host: hostname(),
        pid: a.pid.unwrap_or_else(std::os::unix::process::parent_id),
    };
    let need_session = || "no session: pass --session or --session-from-stdin, or set CLAUDE_SESSION_ID / CLAUDE_CODE_SESSION_ID (TERMLINK_SESSION_ID is used only outside Claude Code)".to_string();
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
                MainVerb::Claim => client.claim_with(&project, &h, &req).await,
                MainVerb::Renew => client.renew(&project, &h).await,
                _ => client.release(&project, &h).await,
            }
        }
        MainVerb::OriginReporting { off, by } => {
            let by = by.or_else(|| env("USER")).unwrap_or_else(|| "operator".into());
            let o = client.set_origin_reporting(&project, &req.harness, !off, &by).await;
            if o.verdict != termlink_session::main_lease::Verdict::Unknown {
                if json {
                    println!("{}", o.to_json());
                } else {
                    println!("origin reporting for harness {}: {}", req.harness, if off { "off" } else { "on" });
                }
                return 0;
            }
            o
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

#[cfg(test)]
mod tests {
    use super::pick_session;

    fn envs(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |k| pairs.iter().find(|(n, _)| *n == k).map(|(_, v)| v.to_string()).filter(|v| !v.is_empty())
    }

    #[test]
    fn flag_wins_then_hook_input() {
        let e = envs(&[("CLAUDE_CODE_SESSION_ID", "cc"), ("TERMLINK_SESSION_ID", "tl-x")]);
        assert_eq!(pick_session(Some(" s1 "), Some(r#"{"session_id":"s2"}"#), &e).as_deref(), Some("s1"));
        assert_eq!(pick_session(None, Some(r#"{"session_id":"s2"}"#), &e).as_deref(), Some("s2"));
    }

    #[test]
    fn empty_hook_session_id_is_absent() {
        let e = envs(&[("CLAUDE_CODE_SESSION_ID", "cc")]);
        assert_eq!(pick_session(None, Some(r#"{"session_id":""}"#), &e).as_deref(), Some("cc"));
        assert_eq!(pick_session(Some("  "), None, &e).as_deref(), Some("cc"));
    }

    #[test]
    fn claude_session_vars_in_order() {
        let both = envs(&[("CLAUDE_SESSION_ID", "a"), ("CLAUDE_CODE_SESSION_ID", "b"), ("CLAUDECODE", "1")]);
        assert_eq!(pick_session(None, None, &both).as_deref(), Some("a"));
        let code = envs(&[("CLAUDE_CODE_SESSION_ID", "b"), ("CLAUDECODE", "1"), ("TERMLINK_SESSION_ID", "tl-x")]);
        assert_eq!(pick_session(None, None, &code).as_deref(), Some("b"));
    }

    #[test]
    fn termlink_session_never_stands_in_for_a_claude_session() {
        let e = envs(&[("CLAUDECODE", "1"), ("TERMLINK_SESSION_ID", "tl-x")]);
        assert_eq!(pick_session(None, None, &e), None);
    }

    #[test]
    fn termlink_session_used_outside_claude_code() {
        let e = envs(&[("TERMLINK_SESSION_ID", "tl-x")]);
        assert_eq!(pick_session(None, None, &e).as_deref(), Some("tl-x"));
        assert_eq!(pick_session(None, None, envs(&[])), None);
    }
}
