//! T-3749 (c): `termlink fleet join` — one command for a new device to join a hub.
//!
//! Field case (ring20 T-2657): the operator's Mac could not reach the .122 hub,
//! so he carried messages by hand. Joining took four separate verbs (hub probe,
//! a TOFU pin, a secret copy, profile add), none of which proved the result.
//! This composes the existing pieces and ends in a real authenticated round
//! trip; every step is printed, and a failure names the step and exits 1.
//!
//! R2: the credential arrives only through an out-of-band channel the operator
//! names (`--bootstrap-from file:|ssh:`, or a hub token file already copied
//! over). It is never sent over the bus and never printed — only a sha256
//! fingerprint of it is.

use anyhow::Result;
use serde_json::json;

use crate::config::{HubEntry, load_hubs_config_strict, save_hubs_config, termlink_config_dir};

/// What to do with the TOFU pin, given what is pinned, what the hub presents
/// and (optionally) the fingerprint the operator confirmed out of band.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PinDecision {
    AlreadyPinned,
    PinNew,
    /// Replace a stale pin: only when the operator's expected fp matches the wire.
    Repin { old: String },
    /// The hub presents a different cert than the one the operator expects.
    RefuseExpectedMismatch { expected: String },
    /// A different cert is pinned and the operator gave no fingerprint to settle it.
    RefuseTofuMismatch { pinned: String },
}

fn norm_fp(fp: &str) -> String {
    let t = fp.trim().to_ascii_lowercase();
    if t.starts_with("sha256:") { t } else { format!("sha256:{t}") }
}

pub(crate) fn decide_pin(pinned: Option<&str>, wire: &str, expected: Option<&str>) -> PinDecision {
    let wire = norm_fp(wire);
    if let Some(exp) = expected {
        let exp = norm_fp(exp);
        if exp != wire {
            return PinDecision::RefuseExpectedMismatch { expected: exp };
        }
    }
    match pinned.map(norm_fp) {
        None => PinDecision::PinNew,
        Some(p) if p == wire => PinDecision::AlreadyPinned,
        Some(p) if expected.is_some() => PinDecision::Repin { old: p },
        Some(p) => PinDecision::RefuseTofuMismatch { pinned: p },
    }
}

/// Validate a join address: plain `host:port`, no scheme.
pub(crate) fn validate_join_addr(addr: &str) -> std::result::Result<(String, u16), String> {
    if addr.contains("://") {
        return Err(format!("address '{addr}' has a URL scheme (wrong scheme); give host:port"));
    }
    match addr.rsplit_once(':') {
        Some((h, p)) if !h.is_empty() && !h.contains('/') => match p.parse::<u16>() {
            Ok(port) if port != 0 => Ok((h.to_string(), port)),
            _ => Err(format!("address '{addr}': '{p}' is not a valid port")),
        },
        _ => Err(format!("address '{addr}' must be host:port, e.g. 192.168.10.122:9100")),
    }
}

pub(crate) struct JoinOpts {
    pub address: String,
    pub name: String,
    pub bootstrap_from: Option<String>,
    pub token_file: Option<String>,
    pub fingerprint: Option<String>,
    pub json: bool,
}

struct Report {
    json: bool,
    steps: Vec<serde_json::Value>,
}

impl Report {
    fn ok(&mut self, step: &str, detail: String) {
        if !self.json {
            println!("[OK]   {step:<11} {detail}");
        }
        self.steps.push(json!({"step": step, "ok": true, "detail": detail}));
    }
    fn note(&mut self, step: &str, detail: String) {
        if !self.json {
            println!("[NOTE] {step:<11} {detail}");
        }
        self.steps.push(json!({"step": step, "ok": true, "note": detail}));
    }
    fn fail(mut self, step: &str, detail: String) -> ! {
        self.steps.push(json!({"step": step, "ok": false, "error": detail}));
        if self.json {
            println!("{}", json!({"ok": false, "failed_step": step, "error": detail, "steps": self.steps}));
        } else {
            eprintln!("[FAIL] {step:<11} {detail}");
            eprintln!("NOT JOINED: fix the step above and re-run the same command (it is idempotent).");
        }
        std::process::exit(1);
    }
}

pub(crate) async fn cmd_fleet_join(o: JoinOpts) -> Result<()> {
    let mut r = Report { json: o.json, steps: Vec::new() };

    // 1. Arguments.
    let (host, port) = match validate_join_addr(&o.address) {
        Ok(hp) => hp,
        Err(e) => r.fail("address", e),
    };
    let name = o.name.trim().to_string();
    if name.is_empty() || name.contains(['/', ':', ' ']) {
        r.fail("name", format!("profile name '{}' must be non-empty, without '/', ':' or spaces", o.name));
    }
    match (&o.bootstrap_from, &o.token_file) {
        (Some(_), Some(_)) => r.fail("credential", "pass --bootstrap-from OR --token-file, not both".into()),
        (None, None) => r.fail(
            "credential",
            "no credential source: pass --bootstrap-from ssh:<hub-host> (reads the hub secret over \
             ssh) or file:<path>, or --token-file <path> holding a token minted on the hub host \
             with `termlink hub token --out <file> --scope interact`"
                .into(),
        ),
        (Some(b), None) if !(b.starts_with("ssh:") || b.starts_with("file:")) => r.fail(
            "credential",
            format!("--bootstrap-from must start with ssh: or file: (got '{b}')"),
        ),
        _ => {}
    }
    let addr = format!("{host}:{port}");

    // 2. Probe the hub's certificate (TLS handshake, no auth, no pin).
    let wire = match termlink_session::tofu::probe_cert_with_timeout(
        &addr,
        std::time::Duration::from_secs(10),
    )
    .await
    {
        Ok((_der, fp)) => fp,
        Err(e) => r.fail("probe", format!("{addr} unreachable over TLS: {e}")),
    };
    r.ok("probe", format!("{addr} presents {wire}"));

    // 3. TOFU pin.
    let store = termlink_session::tofu::KnownHubStore::default_store();
    let pinned = store.get(&addr);
    match decide_pin(pinned.as_deref(), &wire, o.fingerprint.as_deref()) {
        PinDecision::AlreadyPinned => r.ok("pin", "already pinned to this certificate".into()),
        PinDecision::PinNew => {
            if let Err(e) = store.accept(&addr, &wire) {
                r.fail("pin", format!("could not write the pin: {e}"));
            }
            if o.fingerprint.is_some() {
                r.ok("pin", "pinned (matches the fingerprint you gave)".into());
            } else {
                r.ok("pin", "pinned on first use".into());
                r.note(
                    "pin",
                    format!(
                        "no --fingerprint given: confirm {wire} matches `termlink hub fingerprint` \
                         run on the hub host"
                    ),
                );
            }
        }
        PinDecision::Repin { old } => {
            store.remove(&addr);
            if let Err(e) = store.accept(&addr, &wire) {
                r.fail("pin", format!("could not write the pin: {e}"));
            }
            r.ok("pin", format!("replaced stale pin {old} (you confirmed {wire})"));
        }
        PinDecision::RefuseExpectedMismatch { expected } => r.fail(
            "pin",
            format!(
                "TOFU/fingerprint mismatch: you expect {expected} but {addr} presents {wire}. \
                 Do not join; check you have the right address and the hub's real fingerprint"
            ),
        ),
        PinDecision::RefuseTofuMismatch { pinned } => r.fail(
            "pin",
            format!(
                "TOFU mismatch: {addr} is pinned to {pinned} but now presents {wire} (hub cert \
                 rotated, or not the same hub). Confirm the new one on the hub host with \
                 `termlink hub fingerprint`, then re-run with --fingerprint {wire}"
            ),
        ),
    }

    // 4. Credential (out of band; never printed).
    let (secret_file, token_file) = if let Some(src) = &o.bootstrap_from {
        let hex = match super::remote::fetch_bootstrap_secret(src)
            .and_then(|raw| super::remote::normalize_and_validate_secret_hex(&raw))
        {
            Ok(h) => h,
            Err(e) => r.fail("credential", format!("{src}: {e:#}")),
        };
        let path = termlink_config_dir().join("secrets").join(format!("{name}.hex"));
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            r.fail("credential", format!("create {}: {e}", parent.display()));
        }
        let persisted = match super::remote::write_secret_file(&path, &hex) {
            Ok(p) => p,
            Err(e) => r.fail("credential", format!("{e:#}")),
        };
        r.ok(
            "credential",
            format!(
                "secret from {src} written to {} (mode 600, fingerprint {})",
                path.display(),
                super::remote::secret_fingerprint(&persisted)
            ),
        );
        (Some(path.display().to_string()), None)
    } else {
        let tf = o.token_file.clone().expect("checked above");
        if let Err(e) = super::remote::load_hub_token_file(&tf, "interact") {
            r.fail("credential", format!("{e:#}"));
        }
        r.ok("credential", format!("token file {tf} is a valid interact-scope hub token"));
        (None, Some(tf))
    };

    // 5. Profile.
    let mut cfg = match load_hubs_config_strict() {
        Ok(c) => c,
        Err(e) => r.fail("profile", format!("{e:#}")),
    };
    if let Some((other, _)) = cfg.hubs.iter().find(|(n, e)| e.address == addr && **n != name) {
        r.note("profile", format!("profile '{other}' already points at {addr}; adding '{name}' too"));
    }
    let action = if cfg.hubs.contains_key(&name) { "updated" } else { "added" };
    cfg.hubs.insert(
        name.clone(),
        HubEntry {
            address: addr.clone(),
            secret_file,
            secret: None,
            scope: None,
            token_file,
            bootstrap_from: o.bootstrap_from.clone(),
        },
    );
    if let Err(e) = save_hubs_config(&cfg) {
        r.fail("profile", format!("{e:#}"));
    }
    r.ok("profile", format!("{action} '{name}' -> {addr} in {}", crate::config::hubs_config_path().display()));

    // 6. Real round trip: TLS (pinned) + hub.auth + an authenticated read.
    let t0 = std::time::Instant::now();
    let taddr = termlink_protocol::TransportAddr::tcp(&host, port);
    let resp = match super::channel::rpc_call_authed(
        &taddr,
        termlink_protocol::control::method::CHANNEL_LIST,
        json!({}),
    )
    .await
    {
        Ok(resp) => resp,
        Err(e) => r.fail("round-trip", format!("connect/auth to {addr} failed: {e}")),
    };
    let topics = match termlink_session::client::unwrap_result(resp) {
        Ok(v) => v.get("topics").and_then(|t| t.as_array()).map(|a| a.len()).unwrap_or(0),
        Err(e) => r.fail("round-trip", format!("hub refused the authenticated call: {e}")),
    };
    r.ok(
        "round-trip",
        format!("authenticated channel.list answered in {} ms ({topics} topics)", t0.elapsed().as_millis()),
    );

    if o.json {
        println!(
            "{}",
            json!({"ok": true, "profile": name, "address": addr, "fingerprint": wire, "steps": r.steps})
        );
    } else {
        println!("JOINED: '{name}' ({addr}). Send with --hub {name}, e.g.");
        println!("  termlink channel post inbox:<hub-id>/<agent> --payload '...' --hub {name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "sha256:aaaa";
    const B: &str = "sha256:bbbb";

    #[test]
    fn pin_new_when_nothing_pinned() {
        assert_eq!(decide_pin(None, A, None), PinDecision::PinNew);
        assert_eq!(decide_pin(None, A, Some("AAAA")), PinDecision::PinNew);
    }

    #[test]
    fn already_pinned_is_a_noop() {
        assert_eq!(decide_pin(Some(A), A, None), PinDecision::AlreadyPinned);
    }

    #[test]
    fn expected_fingerprint_mismatch_refuses_even_with_no_pin() {
        assert_eq!(
            decide_pin(None, A, Some(B)),
            PinDecision::RefuseExpectedMismatch { expected: B.into() }
        );
    }

    #[test]
    fn stale_pin_without_confirmation_is_a_tofu_refusal() {
        assert_eq!(decide_pin(Some(B), A, None), PinDecision::RefuseTofuMismatch { pinned: B.into() });
    }

    #[test]
    fn stale_pin_is_replaced_only_with_a_matching_confirmation() {
        assert_eq!(decide_pin(Some(B), A, Some(A)), PinDecision::Repin { old: B.into() });
        assert!(matches!(decide_pin(Some(B), A, Some(B)), PinDecision::RefuseExpectedMismatch { .. }));
    }

    #[test]
    fn join_addr_rejects_scheme_and_bad_port() {
        assert!(validate_join_addr("tcp://1.2.3.4:9100").unwrap_err().contains("wrong scheme"));
        assert!(validate_join_addr("1.2.3.4").is_err());
        assert!(validate_join_addr("1.2.3.4:x").unwrap_err().contains("not a valid port"));
        assert_eq!(validate_join_addr("1.2.3.4:9100").unwrap(), ("1.2.3.4".into(), 9100));
    }
}
