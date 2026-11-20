//! T-3708 (P-24 slice a; 06a AL-29; design 3.4-3.5): what the host really provides.
//!
//! * `strength_host` (FCW-123): computed from `/proc` at each status call; a flag that cannot be
//!   computed reads as absent, never present.
//! * Clock skew (V-5): estimated from hub round trips (`hub_now_ms` in `channel.subscribe` replies);
//!   "clock skew" above 2 s, cross-host delay figures "unusable" above 30 s.
//! * NTP state: sampled in the background (timedatectl, else chronyc, else `unchecked`); status never
//!   runs a command.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

pub const SKEW_FLAG_MS: i64 = 2_000;
pub const SKEW_UNUSABLE_MS: i64 = 30_000;
const SKEW_SAMPLES: usize = 5;
/// How often the NTP state is sampled (its field bound is 1 h).
pub const NTP_EVERY: Duration = Duration::from_secs(1800);

/// One running sidecar found in `/proc`: its agent name and owner uid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    pub agent: String,
    pub uid: u32,
}

/// The agent a sidecar process serves, from its command line: the Rust sidecar
/// (`termlink sidecar start --config …/<agent>.json`) or the framework one (`--agent <agent>`).
pub fn sidecar_agent(argv: &[String]) -> Option<String> {
    let s: Vec<&str> = argv.iter().map(String::as_str).collect();
    if let Some(i) = s.iter().position(|a| *a == "--agent") {
        if s.iter().any(|a| a.ends_with("http_server.py")) {
            return s.get(i + 1).map(|a| a.to_string());
        }
    }
    if s.windows(2).any(|w| w == ["sidecar", "start"]) {
        let i = s.iter().position(|a| *a == "--config")?;
        let p = Path::new(s.get(i + 1)?);
        return p.file_stem().and_then(|f| f.to_str()).map(str::to_string);
    }
    None
}

/// Every sidecar process under `proc_root`; `None` when `/proc` cannot be read at all.
pub fn sidecars(proc_root: &Path) -> Option<Vec<Proc>> {
    let rd = std::fs::read_dir(proc_root).ok()?;
    let mut out = vec![];
    for e in rd.flatten() {
        let name = e.file_name();
        let Some(pid) = name.to_str().filter(|n| n.bytes().all(|b| b.is_ascii_digit())) else { continue };
        let dir = proc_root.join(pid);
        let Ok(raw) = std::fs::read(dir.join("cmdline")) else { continue };
        let argv: Vec<String> = raw.split(|b| *b == 0).filter(|a| !a.is_empty()).map(|a| String::from_utf8_lossy(a).into_owned()).collect();
        let Some(agent) = sidecar_agent(&argv) else { continue };
        let Some(uid) = std::fs::read_to_string(dir.join("status")).ok().and_then(|s| {
            s.lines().find_map(|l| l.strip_prefix("Uid:").and_then(|r| r.split_whitespace().nth(1)).and_then(|u| u.parse().ok()))
        }) else {
            continue;
        };
        out.push(Proc { agent, uid });
    }
    Some(out)
}

/// The strength flags as the host shows them (arch 6.7.1; E-58). `procs` is `None` when the process
/// table could not be read; `own_uid` is this sidecar's effective uid.
pub fn strength(procs: Option<&[Proc]>, own_uid: u32) -> Value {
    // r1 MAJOR 3: the agent sessions' accounts cannot be read from here (only the sidecars'), so
    // per-agent accounts are never claimed in PH-2; the flag names why it is absent (FCW-123).
    let (per_agent, shared) = match procs {
        None => ("no (could not check)".to_string(), None),
        Some(ps) => {
            let mut by_agent: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
            for p in ps {
                by_agent.entry(p.agent.as_str()).or_default().insert(p.uid);
            }
            let uids: BTreeSet<u32> = ps.iter().map(|p| p.uid).collect();
            if by_agent.len() < 2 {
                ("no (fewer than two sidecars visible: nothing to compare)".to_string(), None)
            } else if uids.len() < by_agent.len() || uids.contains(&0) {
                (format!("no: {} agents run under {} account(s)", by_agent.len(), uids.len()), Some(true))
            } else {
                ("no (sidecars run under separate accounts; the agent sessions' accounts are not checked)".to_string(), Some(false))
            }
        }
    };
    let same_user = if own_uid == 0 {
        "absent: agents run as root".to_string()
    } else {
        match shared {
            Some(true) => format!("absent: agents share accounts (this sidecar runs as uid {own_uid})"),
            _ => "absent (could not check)".to_string(),
        }
    };
    json!({
        "per_agent_accounts": per_agent,
        "same_user_separation": same_user,
        // Not built in PH-2: no operator key, no confirmation provider, no policy records exist to find.
        "operator_key": "no",
        "confirmation_provider": "no",
        "policy_records": "none (fixed defaults)",
    })
}

/// `strength_host` for this process.
pub fn strength_now() -> Value {
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    let procs = sidecars(Path::new("/proc"));
    strength(procs.as_deref(), uid)
}

/// The skew estimator: the median offset (hub − local midpoint) of the last few round trips.
#[derive(Default)]
pub struct Skew {
    samples: VecDeque<(i64, u64)>,
}

impl Skew {
    /// One round trip: local wall before and after the call, and the hub's `hub_now_ms`.
    pub fn sample(&mut self, before_ms: u64, after_ms: u64, hub_now_ms: u64) {
        let mid = before_ms / 2 + after_ms / 2;
        let offset = hub_now_ms as i64 - mid as i64;
        self.samples.push_back((offset, after_ms.saturating_sub(before_ms)));
        while self.samples.len() > SKEW_SAMPLES {
            self.samples.pop_front();
        }
    }

    /// `{offset_ms, rtt_ms, clock, cross_host_delays}`, or `None` before any sample.
    pub fn view(&self) -> Option<Value> {
        if self.samples.is_empty() {
            return None;
        }
        let mut o: Vec<i64> = self.samples.iter().map(|s| s.0).collect();
        o.sort();
        let offset = o[o.len() / 2];
        let rtt = self.samples.back().map(|s| s.1).unwrap_or(0);
        Some(judge(offset, rtt))
    }
}

/// The skew verdict for a median offset (V-5).
pub fn judge(offset_ms: i64, rtt_ms: u64) -> Value {
    let a = offset_ms.abs();
    json!({
        "offset_ms": offset_ms,
        "rtt_ms": rtt_ms,
        "clock": if a > SKEW_FLAG_MS { "clock skew" } else { "ok" },
        "cross_host_delays": if a > SKEW_UNUSABLE_MS { "unusable" } else { "usable" },
    })
}

fn skew_cell() -> &'static Mutex<Skew> {
    static S: OnceLock<Mutex<Skew>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Skew::default()))
}

/// Record a round trip in the process estimator and stamp the `skew` field.
pub fn record_round_trip(before_ms: u64, after_ms: u64, hub_now_ms: Option<u64>) {
    let Some(h) = hub_now_ms else { return };
    match skew_cell().lock() {
        Ok(mut s) => s.sample(before_ms, after_ms, h),
        Err(p) => p.into_inner().sample(before_ms, after_ms, h),
    }
    crate::fields::touch(crate::fields::SKEW);
}

/// The process estimator's view.
pub fn skew_view() -> Option<Value> {
    match skew_cell().lock() {
        Ok(s) => s.view(),
        Err(p) => p.into_inner().view(),
    }
}

/// Classify the NTP tools' output: timedatectl's `NTPSynchronized` value, else chronyc's tracking.
pub fn ntp_from(timedatectl: Option<&str>, chronyc: Option<&str>) -> &'static str {
    match timedatectl.map(str::trim) {
        Some("yes") => return "synchronised (timedatectl)",
        Some("no") => return "not synchronised (timedatectl)",
        _ => {}
    }
    if let Some(c) = chronyc {
        let leap = c.lines().find_map(|l| l.strip_prefix("Leap status").map(|r| r.trim_start_matches([' ', ':']).trim().to_string()));
        return match leap.as_deref() {
            Some("Normal") => "synchronised (chronyc)",
            Some(_) => "not synchronised (chronyc)",
            None => "unchecked",
        };
    }
    "unchecked"
}

fn run_bounded(cmd: &str, args: &[&str]) -> Option<String> {
    let mut child = std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => {
                let mut s = String::new();
                use std::io::Read as _;
                child.stdout.take()?.read_to_string(&mut s).ok()?;
                return Some(s);
            }
            Ok(Some(_)) => return None,
            Ok(None) if std::time::Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn ntp_cell() -> &'static Mutex<Option<&'static str>> {
    static N: OnceLock<Mutex<Option<&'static str>>> = OnceLock::new();
    N.get_or_init(|| Mutex::new(None))
}

/// Sample the NTP state now (P-23's commands, 2 s bound each) and stamp the `ntp` field.
pub fn sample_ntp() {
    let t = run_bounded("timedatectl", &["show", "-p", "NTPSynchronized", "--value"]);
    let c = if t.as_deref().map(str::trim).is_some_and(|v| v == "yes" || v == "no") { None } else { run_bounded("chronyc", &["tracking"]) };
    let state = ntp_from(t.as_deref(), c.as_deref());
    match ntp_cell().lock() {
        Ok(mut n) => *n = Some(state),
        Err(p) => *p.into_inner() = Some(state),
    }
    crate::fields::touch(crate::fields::NTP);
}

/// The last sampled NTP state.
pub fn ntp_view() -> Option<&'static str> {
    match ntp_cell().lock() {
        Ok(n) => *n,
        Err(p) => *p.into_inner(),
    }
}

/// Start the background NTP sampler (once per process).
pub fn start_ntp_sampler() {
    static STARTED: OnceLock<()> = OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }
    let _ = std::thread::Builder::new().name("ntp-sampler".into()).spawn(|| loop {
        sample_ntp();
        std::thread::sleep(NTP_EVERY);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(agent: &str, uid: u32) -> Proc {
        Proc { agent: agent.into(), uid }
    }

    #[test]
    fn root_host_reads_absent_never_present() {
        let ps = [p("a", 0), p("b", 0)];
        let v = strength(Some(&ps), 0);
        assert_eq!(v["per_agent_accounts"], "no: 2 agents run under 1 account(s)");
        assert_eq!(v["same_user_separation"], "absent: agents run as root");
        assert_eq!(v["operator_key"], "no");
        assert_eq!(v["confirmation_provider"], "no");
        assert_eq!(v["policy_records"], "none (fixed defaults)");
    }

    #[test]
    fn unreadable_process_table_reads_absent() {
        assert_eq!(strength(None, 1000)["per_agent_accounts"], "no (could not check)");
    }

    #[test]
    fn separate_sidecar_accounts_are_never_claimed_as_per_agent_accounts() {
        // r1 MAJOR 3: the sessions may still share one account; nothing here can see them
        for ps in [vec![p("a", 1001), p("b", 1002)], vec![p("a", 1001), p("a", 1003), p("b", 1002)]] {
            let v = strength(Some(&ps), 1001);
            assert!(v["per_agent_accounts"].as_str().unwrap().starts_with("no"), "{v}");
            assert!(v["same_user_separation"].as_str().unwrap().starts_with("absent"), "{v}");
        }
        assert_eq!(strength(None, 1001)["same_user_separation"], "absent (could not check)");
        // a root sidecar among separated ones is shared, not separated
        assert_eq!(strength(Some(&[p("a", 0), p("b", 1001)]), 1001)["per_agent_accounts"], "no: 2 agents run under 2 account(s)");
    }

    #[test]
    fn sidecar_agent_from_both_kinds_of_command_line() {
        let a = |s: &str| s.split(' ').map(str::to_string).collect::<Vec<_>>();
        assert_eq!(sidecar_agent(&a("/root/.cargo/bin/termlink sidecar start --config /etc/termlink-sidecar/claude-termlink.json")).as_deref(), Some("claude-termlink"));
        assert_eq!(sidecar_agent(&a("/usr/bin/python3 /opt/x/lib/sidecar/http_server.py --port 0 --agent 010-termlink")).as_deref(), Some("010-termlink"));
        assert_eq!(sidecar_agent(&a("termlink register --name x --agent y")), None);
    }

    #[test]
    fn skew_flags_at_the_stated_thresholds() {
        for (off, clock, delays) in [(0, "ok", "usable"), (3_000, "clock skew", "usable"), (45_000, "clock skew", "unusable"), (-45_000, "clock skew", "unusable")] {
            let mut s = Skew::default();
            s.sample(1_000_000, 1_000_010, (1_000_005i64 + off) as u64);
            let v = s.view().unwrap();
            assert_eq!(v["offset_ms"], off, "offset {off}");
            assert_eq!(v["clock"], clock, "offset {off}");
            assert_eq!(v["cross_host_delays"], delays, "offset {off}");
        }
    }

    #[test]
    fn skew_is_the_median_so_one_slow_round_trip_does_not_flag() {
        let mut s = Skew::default();
        for _ in 0..4 {
            s.sample(1_000, 1_010, 1_005);
        }
        s.sample(1_000, 9_000, 9_000); // one outlier
        assert_eq!(s.view().unwrap()["clock"], "ok");
    }

    #[test]
    fn ntp_reads_unchecked_without_tools() {
        assert_eq!(ntp_from(None, None), "unchecked");
        assert_eq!(ntp_from(Some("yes\n"), None), "synchronised (timedatectl)");
        assert_eq!(ntp_from(Some("no"), None), "not synchronised (timedatectl)");
        assert_eq!(ntp_from(Some(""), Some("Reference ID : X\nLeap status     : Normal\n")), "synchronised (chronyc)");
        assert_eq!(ntp_from(None, Some("Leap status     : Not synchronised\n")), "not synchronised (chronyc)");
        assert_eq!(ntp_from(None, Some("garbage")), "unchecked");
    }
}
