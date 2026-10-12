//! Agent-side client verbs over the local API (B14): what an agent runs from its
//! own shell to talk to its sidecar.
//!
//! | verb | socket | request |
//! |---|---|---|
//! | `send --to A --to-instance I --conv C --body B [--priority N]` | send | `send` |
//! | `reply --conv C --in-reply-to ID --body B` | send | `reply` |
//! | `status [--msg-id ID] [--conv C] [--json]` | status | `status` |
//!
//! The send socket is `$TERMLINK_SIDECAR_SOCKET` (the same variable the hooks
//! use); the status socket is `$TERMLINK_SIDECAR_STATUS_SOCKET`, else the send
//! socket's sibling `<agent>.status.sock`. The admin socket is never derived
//! from the agent's: the agent has no verb for it (SI-14, SI-23).
//!
//! Operator verbs (OQ-2 ruling E), on the admin socket given explicitly by
//! `--socket` or `$TERMLINK_SIDECAR_ADMIN_SOCKET`:
//!
//! | verb | request |
//! |---|---|
//! | `admin held [--body]` | what is held for the operator, by name |
//! | `admin allow --key FP --why TEXT` | admit a key that knocked; release its held messages |
//! | `admin deny --key FP --why TEXT` | refuse it |
//!
//! Exit codes: 0 the sidecar answered `ok`, 1 it refused (the answer is printed),
//! 2 usage or the socket could not be reached.

use crate::hook;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;

pub const STATUS_SOCKET_ENV: &str = "TERMLINK_SIDECAR_STATUS_SOCKET";
pub const ADMIN_SOCKET_ENV: &str = "TERMLINK_SIDECAR_ADMIN_SOCKET";
const TIMEOUT: Duration = Duration::from_secs(20);

fn usage() -> i32 {
    eprintln!("usage: termlink-sidecar send --to <agent> --to-instance <instance> --conv <conv> --body <text> [--priority <n>]");
    eprintln!("       termlink-sidecar reply --conv <conv> --in-reply-to <msg_id> --body <text>");
    eprintln!("       termlink-sidecar status [--msg-id <id>] [--conv <conv>] [--json]");
    eprintln!("       termlink-sidecar admin held [--body] | allow|deny --key <fingerprint> --why <text>  [--socket <admin socket>]");
    2
}

/// `--flag value` pairs and bare `--json`; anything else is a usage error.
fn parse(args: &[&str]) -> Option<(Vec<(String, String)>, bool)> {
    let mut pairs = vec![];
    let mut as_json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--json" => {
                as_json = true;
                i += 1;
            }
            f if f.starts_with("--") && i + 1 < args.len() => {
                pairs.push((f.trim_start_matches("--").replace('-', "_"), args[i + 1].to_owned()));
                i += 2;
            }
            _ => return None,
        }
    }
    Some((pairs, as_json))
}

fn send_socket() -> Option<PathBuf> {
    std::env::var_os(hook::SOCKET_ENV).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// The status socket: explicit, else the send socket's sibling.
pub fn status_socket() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os(STATUS_SOCKET_ENV).filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(p));
    }
    let send = send_socket()?;
    let name = send.file_name()?.to_str()?;
    let agent = name.strip_suffix(".send.sock")?;
    Some(send.with_file_name(format!("{agent}.status.sock")))
}

/// UTC `YYYY-MM-DDTHH:MM:SS.mmmZ` for a millisecond timestamp.
pub fn utc(ms: u64) -> String {
    let secs = ms / 1000;
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    // Civil-from-days (H. Hinnant), valid for the whole u64-ms range we use.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z", rem / 3600, rem % 3600 / 60, rem % 60, ms % 1000)
}

/// The record print (BV-7): one block per message, the five stages in order,
/// each with its time or `(not yet)`.
pub fn render_status(v: &Value) -> String {
    let mut out = format!(
        "agent {}  hub {}  delivery {}\n",
        v["agent"].as_str().unwrap_or("?"),
        v["hub_id"].as_str().unwrap_or("?"),
        v["delivery"].as_str().unwrap_or("?")
    );
    for m in v["messages"].as_array().into_iter().flatten() {
        out.push_str(&format!("message {}  stage {}", m["msg_id"].as_str().unwrap_or("?"), m["stage"].as_str().unwrap_or("?")));
        if let Some(l) = m["label"].as_str() {
            out.push_str(&format!("  ({l})"));
        }
        if let Some(r) = m["refused"].as_str() {
            out.push_str(&format!("  refused: {r}"));
        }
        out.push('\n');
        for r in m["record"].as_array().into_iter().flatten() {
            let st = r["stage"].as_str().unwrap_or("?");
            match r["at_ms"].as_u64() {
                Some(t) => out.push_str(&format!("  {st:<12} {}  by {}\n", utc(t), r["by"].as_str().unwrap_or("?"))),
                None => out.push_str(&format!("  {st:<12} (not yet)\n")),
            }
        }
    }
    for why in v["missing_strength"].as_array().into_iter().flatten() {
        out.push_str(&format!("missing: {}\n", why.as_str().unwrap_or("?")));
    }
    out
}

pub fn main_client(verb: &str, args: &[&str]) -> i32 {
    // `admin <sub> ...`: the sub-verb is a bare word; `held --body` is the one
    // flag without a value (on `send` and `reply`, `--body` takes the text).
    let admin_body = verb == "admin" && args.contains(&"--body");
    let admin_rest: Vec<&str> = args.iter().skip(1).copied().filter(|a| *a != "--body").collect();
    let flags: &[&str] = if verb == "admin" { &admin_rest } else { args };
    let Some((pairs, as_json)) = parse(flags) else { return usage() };
    let get = |k: &str| pairs.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    let (socket, req) = match verb {
        "send" => {
            let (Some(to), Some(inst), Some(conv), Some(body)) = (get("to"), get("to_instance"), get("conv"), get("body")) else {
                return usage();
            };
            let mut r = json!({"op": "send", "to": to, "to_instance": inst, "conv": conv, "body": body});
            if let Some(p) = get("priority") {
                let Ok(n) = p.parse::<i64>() else { return usage() };
                r["priority"] = json!(n);
            }
            (send_socket(), r)
        }
        "reply" => {
            let (Some(conv), Some(irt), Some(body)) = (get("conv"), get("in_reply_to"), get("body")) else {
                return usage();
            };
            (send_socket(), json!({"op": "reply", "conv": conv, "in_reply_to": irt, "body": body}))
        }
        "status" => {
            let mut r = json!({"op": "status"});
            if let Some(m) = get("msg_id") {
                r["msg_id"] = json!(m);
            }
            if let Some(c) = get("conv") {
                r["conv"] = json!(c);
            }
            (status_socket(), r)
        }
        "admin" => {
            let socket = get("socket")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os(ADMIN_SOCKET_ENV).filter(|v| !v.is_empty()).map(PathBuf::from));
            let Some(socket) = socket else {
                eprintln!("termlink-sidecar admin: give --socket <admin socket> or set {ADMIN_SOCKET_ENV}");
                return 2;
            };
            let Some(sub) = args.first() else { return usage() };
            let req = match *sub {
                "held" => json!({"op": "held", "body": admin_body}),
                "allow" | "deny" => {
                    let Some(key) = get("key") else { return usage() };
                    json!({"op": sub, "key": key, "why": get("why").unwrap_or_default()})
                }
                _ => return usage(),
            };
            (Some(socket), req)
        }
        _ => return usage(),
    };
    let Some(socket) = socket else {
        eprintln!("termlink-sidecar {verb}: {} is not set", hook::SOCKET_ENV);
        return 2;
    };
    let resp = match hook::call(&socket, &req, TIMEOUT) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("termlink-sidecar {verb}: {e}");
            return 2;
        }
    };
    let ok = resp.get("code").is_none() && resp.get("ok").and_then(Value::as_bool) != Some(false);
    if verb == "status" && ok && !as_json {
        print!("{}", render_status(&resp));
    } else {
        println!("{resp}");
    }
    if ok { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_formats_known_instants() {
        assert_eq!(utc(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(utc(951_782_400_123), "2000-02-29T00:00:00.123Z");
        assert_eq!(utc(1_791_806_400_000), "2026-10-12T12:00:00.000Z");
    }

    #[test]
    fn body_takes_a_value_on_send() {
        let (p, _) = parse(&["--to", "b", "--body", "hello there"]).unwrap();
        assert!(p.contains(&("body".to_owned(), "hello there".to_owned())));
    }

    #[test]
    fn parse_rejects_dangling_flags_and_bare_words() {
        assert!(parse(&["--to"]).is_none());
        assert!(parse(&["word"]).is_none());
        let (p, j) = parse(&["--in-reply-to", "x", "--json"]).unwrap();
        assert_eq!(p, vec![("in_reply_to".to_owned(), "x".to_owned())]);
        assert!(j);
    }
}
