//! T-3654 (OD-18 / R-67, R-68): `termlink main` against a real, throwaway hub.
//!
//! Each test starts its own `termlink hub start` in a temp runtime dir, HOME and identity
//! dir (unix socket only), so nothing outside the test is touched. Lease timing is shortened
//! through the `TERMLINK_MAIN_TEST_*` seams only, which are honoured only because each test
//! dir is marked as a test estate (T-3686, AL-16); `seam_outside_estate_is_unknown_naming_it`
//! is the control that the same seams are refused without the marker.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use serde_json::Value;
use termlink_test_utils::{ProcessGuard, TestDir};

const LEASE_MS: u64 = 1_500;
const QUIET_MS: u64 = 800;
const COOLDOWN_MS: u64 = 400;

struct Hub {
    dir: TestDir,
    sock: String,
    _guard: Option<ProcessGuard>,
}

fn base_cmd(dir: &Path) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), dir);
    c.env("HOME", dir)
        .env("TERMLINK_IDENTITY_DIR", dir.join("id"))
        .env("TERMLINK_MAIN_PROJECT", "t3654-proj")
        .env("TERMLINK_MAIN_TEST_LEASE_MS", LEASE_MS.to_string())
        .env("TERMLINK_MAIN_TEST_QUIET_MS", QUIET_MS.to_string())
        .env("TERMLINK_MAIN_TEST_COOLDOWN_MS", COOLDOWN_MS.to_string())
        .env("TERMLINK_AGENT_ID", "agent-t3654")
        .env_remove("TERMLINK_IDENTITY_FILE")
        .env_remove("CLAUDE_SESSION_ID")
        .env_remove("TERMLINK_SESSION_ID")
        .env_remove("TERMLINK_MAIN_OBLIGATION_TOPICS");
    c
}

/// Mark `dir` (HOME and runtime dir of every command here) as a test estate (T-3686).
fn mark_estate(dir: &Path) {
    use termlink_session::test_seam::{ESTATE_MARKER, ESTATE_MARKER_MAGIC};
    std::fs::write(dir.join(ESTATE_MARKER), format!("{ESTATE_MARKER_MAGIC}\n")).expect("estate marker");
}

fn start_hub(name: &str) -> Hub {
    let dir = TestDir::new(name);
    mark_estate(&dir.path);
    let child = base_cmd(&dir.path)
        .args(["hub", "start"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn hub");
    let guard = ProcessGuard::new(child, name);
    let sock = dir.path.join("hub.sock");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !sock.exists() {
        assert!(Instant::now() < deadline, "hub socket never appeared");
        std::thread::sleep(Duration::from_millis(50));
    }
    Hub { sock: sock.to_string_lossy().into_owned(), dir, _guard: Some(guard) }
}

impl Hub {
    /// Run `termlink main <verb> ... --json`; returns (exit code, parsed JSON).
    fn main(&self, args: &[&str]) -> (i32, Value) {
        let mut c = base_cmd(&self.dir.path);
        c.arg("main").args(args).args(["--hub", &self.sock, "--json"]);
        let out = c.output().expect("run termlink main");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let v: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
            panic!("not JSON ({e}): stdout={stdout} stderr={}", String::from_utf8_lossy(&out.stderr))
        });
        (out.status.code().unwrap_or(-1), v)
    }

    fn channel(&self, args: &[&str]) {
        let mut c = base_cmd(&self.dir.path);
        c.arg("channel").args(args).args(["--hub", &self.sock]);
        let out = c.output().expect("run termlink channel");
        assert!(out.status.success(), "channel {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }
}

fn wait_until(ms: u64) {
    let now = termlink_session::main_lease::now_ms();
    if ms > now {
        std::thread::sleep(Duration::from_millis(ms - now));
    }
}

fn recent_acquires(v: &Value) -> Vec<Value> {
    v["recent"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["event"]["kind"] == "acquire")
        .map(|r| r["event"].clone())
        .collect()
}

#[test]
fn two_sessions_one_main_and_takeover_after_lapse_quiet_and_cooldown() {
    let hub = start_hub("t3654-two");
    let (rc, a) = hub.main(&["claim", "--session", "S-A"]);
    assert_eq!(rc, 0, "{a}");
    assert_eq!(a["verdict"], "main");
    assert_eq!(a["generation"], 1);
    assert_eq!(a["holder"]["session"], "S-A");
    let (rc, b) = hub.main(&["claim", "--session", "S-B"]);
    assert_eq!(rc, 1, "{b}");
    assert_eq!(b["verdict"], "not-main");
    assert_eq!(b["holder"]["session"], "S-A", "the non-main is told who main is");
    assert_eq!(hub.main(&["am-i", "--session", "S-A"]).0, 0);
    assert_eq!(hub.main(&["am-i", "--session", "S-B"]).0, 1);

    // A busy holder keeps main by renewing.
    let (rc, r) = hub.main(&["renew", "--session", "S-A"]);
    assert_eq!((rc, r["action"].as_str()), (0, Some("renewed")), "{r}");
    let lapse_at = r["claimed_until_ms"].as_u64().unwrap();
    // B's renew never acquires a lease it did not hold.
    assert_eq!(hub.main(&["renew", "--session", "S-B"]).0, 1);

    // A stops renewing. After the lapse but inside the quiet period: B still waits.
    wait_until(lapse_at + 100);
    let (rc, w) = hub.main(&["claim", "--session", "S-B"]);
    assert_eq!(rc, 1, "inside the quiet period: {w}");
    assert!(w["holder"].is_null(), "vacant: {w}");
    let eligible = w["eligible_at_ms"].as_u64().expect("eligible_at_ms named");
    assert!(eligible >= lapse_at + QUIET_MS, "takeover not before lapse + quiet: {eligible} vs {lapse_at}");

    wait_until(eligible + 50);
    let (rc, t) = hub.main(&["claim", "--session", "S-B"]);
    assert_eq!(rc, 0, "{t}");
    assert_eq!(t["generation"], 2, "generation + 1 on takeover");
    let acq = recent_acquires(&t);
    let last = acq.last().unwrap();
    assert_eq!(last["reason"], "takeover-after-lapse");
    assert_eq!(last["from"]["session"], "S-A");
    assert!(last["at_ms"].as_u64().unwrap() >= lapse_at + QUIET_MS);

    // No automatic take-back; the old holder is told it lost main.
    let (rc, old) = hub.main(&["claim", "--session", "S-A"]);
    assert_eq!(rc, 1, "{old}");
    assert_eq!(old["holder"]["session"], "S-B");
    assert_eq!(old["lost"]["generation"], 2);
    assert_eq!(old["lost"]["to"]["session"], "S-B");

    // Clean release, then the cooldown/quiet rules hand it on with generation 3.
    let (rc, rel) = hub.main(&["release", "--session", "S-B"]);
    assert_eq!((rc, rel["action"].as_str()), (1, Some("released")), "{rel}");
    let (_, s) = hub.main(&["status"]);
    let eligible = s["eligible_at_ms"].as_u64().unwrap();
    wait_until(eligible + 50);
    let (rc, c) = hub.main(&["claim", "--session", "S-A"]);
    assert_eq!(rc, 0, "{c}");
    assert_eq!(c["generation"], 3);
    assert_eq!(recent_acquires(&c).last().unwrap()["reason"], "after-release");
}

#[test]
fn operator_pin_overrides_a_live_holder_and_is_audited() {
    let hub = start_hub("t3654-pin");
    assert_eq!(hub.main(&["claim", "--session", "S-A"]).0, 0);
    let (rc, p) = hub.main(&["pin", "--to", "S-B", "--reason", "operator's own session", "--by", "op"]);
    assert_eq!(rc, 0, "{p}");
    assert_eq!(p["pinned"], "S-B");
    assert_eq!(p["pin_record"]["by"], "op");
    // The pin alone moves nothing; S-A is still main until S-B claims.
    assert_eq!(hub.main(&["am-i", "--session", "S-A"]).0, 0);
    // A third session is not the pin and stays off.
    assert_eq!(hub.main(&["claim", "--session", "S-C"]).0, 1);
    let (rc, b) = hub.main(&["claim", "--session", "S-B"]);
    assert_eq!(rc, 0, "pinned session takes main at once: {b}");
    assert_eq!(b["generation"], 2);
    let last = recent_acquires(&b).last().unwrap().clone();
    assert_eq!(last["reason"], "pin-override");
    assert_eq!(last["from"]["session"], "S-A");
    assert_eq!(hub.main(&["am-i", "--session", "S-A"]).0, 1);
    // Clearing the pin is recorded too.
    let (rc, c) = hub.main(&["pin", "--clear"]);
    assert_eq!(rc, 0, "{c}");
    assert!(c["pinned"].is_null());
}

#[test]
fn hub_down_is_unknown_never_main() {
    let mut hub = start_hub("t3654-down");
    assert_eq!(hub.main(&["claim", "--session", "S-A"]).0, 0);
    hub._guard = None; // kill the hub
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let (rc, v) = hub.main(&["am-i", "--session", "S-A"]);
        if rc == 2 {
            assert_eq!(v["verdict"], "unknown");
            break;
        }
        assert!(Instant::now() < deadline, "hub kill not observed: {v}");
        std::thread::sleep(Duration::from_millis(100));
    }
    for verb in ["claim", "renew", "release", "status"] {
        let (rc, v) = hub.main(&[verb, "--session", "S-A"]);
        assert_eq!(rc, 2, "{verb}: {v}");
        assert_eq!(v["verdict"], "unknown", "{verb}");
    }
    // No session at all is unknown too, not a guess.
    let (rc, _) = hub.main(&["am-i"]);
    assert_eq!(rc, 2);
}

/// T-3686 (E-163.1, FCW-60): the same seams without the estate marker make every
/// `termlink main` verb answer unknown (exit 2) naming the variable, before any hub contact;
/// with the marker back the verbs work again (the seams apply).
#[test]
fn seam_outside_estate_is_unknown_naming_it() {
    let hub = start_hub("t3686-seam");
    let marker = hub.dir.path.join(termlink_session::test_seam::ESTATE_MARKER);
    std::fs::remove_file(&marker).expect("remove marker");
    for verb in ["claim", "renew", "release", "status", "am-i"] {
        let (rc, v) = hub.main(&[verb, "--session", "S-A"]);
        assert_eq!(rc, 2, "{verb}: {v}");
        assert_eq!(v["verdict"], "unknown", "{verb}");
        let msg = v["message"].as_str().unwrap_or_default();
        assert!(msg.contains("TERMLINK_MAIN_TEST_LEASE_MS") && msg.contains("outside a test estate"), "{verb}: {msg}");
        assert!(!msg.contains(&LEASE_MS.to_string()), "value echoed: {msg}");
    }
    mark_estate(&hub.dir.path);
    let (rc, v) = hub.main(&["claim", "--session", "S-A"]);
    assert_eq!(rc, 0, "{v}");
    let span = v["claimed_until_ms"].as_u64().unwrap() as i64 - termlink_session::main_lease::now_ms() as i64;
    assert!(span <= LEASE_MS as i64 + 500, "seamed lease length applies inside the estate: {span} ms");
}

/// R-68 test (1): the lease lapses while the holder has an open obligation (a claimed work
/// item). The takeover is visible in "who holds main", and the open claim is recorded for
/// the operator as uncertain.
#[test]
fn r68_lapse_during_an_open_obligation_ends_in_a_visible_takeover() {
    let hub = start_hub("t3654-r68a");
    hub.channel(&["create", "work:t3654"]);
    hub.channel(&["post", "work:t3654", "--payload", "role request"]);
    let (_, a) = hub.main(&["claim", "--session", "S-A"]);
    let lapse_at = a["claimed_until_ms"].as_u64().unwrap();
    // S-A takes the work item and then stalls (no renew of either).
    hub.channel(&["claim", "work:t3654", "0", "--claimer", "S-A", "--ttl-ms", "60000"]);
    wait_until(lapse_at + QUIET_MS.max(COOLDOWN_MS) + 100);
    let (rc, b) = hub.main(&["claim", "--session", "S-B", "--obligation-topic", "work:t3654"]);
    assert_eq!(rc, 0, "{b}");
    let (_, s) = hub.main(&["status"]);
    assert_eq!(s["holder"]["session"], "S-B", "takeover visible in who-holds-main");
    assert_eq!(s["generation"], 2);
    let acq = recent_acquires(&s);
    let obl = &acq.last().unwrap()["obligations"];
    assert_eq!(obl[0]["topic"], "work:t3654", "{obl}");
    assert_eq!(obl[0]["offset"], 0);
    assert_eq!(obl[0]["class"], "uncertain");
    assert_eq!(obl[0]["to"], "operator");
}

/// R-68 test (2): a stuck holder whose sidecar is alive. A sidecar only READS the lease (it
/// never renews: R-67.c), so readers polling the whole time do not keep a stuck holder main.
#[test]
fn r68_stuck_holder_with_live_reader_ends_in_a_visible_takeover() {
    let hub = start_hub("t3654-r68b");
    let (_, a) = hub.main(&["claim", "--session", "S-A"]);
    let lapse_at = a["claimed_until_ms"].as_u64().unwrap();
    // The "live sidecar": reads status and am-i throughout; claimed_until never moves.
    let stop = lapse_at + QUIET_MS.max(COOLDOWN_MS) + 100;
    while termlink_session::main_lease::now_ms() < stop {
        let (_, s) = hub.main(&["status"]);
        if let Some(until) = s["claimed_until_ms"].as_u64() {
            assert_eq!(until, lapse_at, "a reader must never extend the lease");
        }
        let _ = hub.main(&["am-i", "--session", "S-A"]);
        std::thread::sleep(Duration::from_millis(150));
    }
    let (rc, b) = hub.main(&["claim", "--session", "S-B"]);
    assert_eq!(rc, 0, "{b}");
    let (_, s) = hub.main(&["status"]);
    assert_eq!(s["holder"]["session"], "S-B");
    assert_eq!(s["generation"], 2);
    assert_eq!(recent_acquires(&s).last().unwrap()["from"]["session"], "S-A");
    assert_eq!(hub.main(&["am-i", "--session", "S-A"]).0, 1);
}

/// T-3660: the lease identity is the SESSION, not the process. A SessionEnd hook (or any later
/// process of the same session) has a different pid and agent from the process that claimed;
/// the hub checks ownership by the exact claimer string, which carries the pid. Release and
/// renew must still work for the same session, and must still refuse another session.
/// (Every `hub.main` call here is its own process; `--pid` makes them differ the way a real
/// hook's parent does, because the default `parent_id()` is this test runner for all of them.)
#[test]
fn same_session_from_another_process_renews_and_releases_other_sessions_refused() {
    let hub = start_hub("t3660-xproc");
    let (rc, a) = hub.main(&["claim", "--session", "S-A", "--pid", "1111", "--agent", "claimer-proc"]);
    assert_eq!(rc, 0, "{a}");
    let generation = a["generation"].as_u64().unwrap();

    // Another session, any pid: refused, nothing changes.
    let (rc, x) = hub.main(&["release", "--session", "S-X", "--pid", "1111"]);
    assert_eq!((rc, x["action"].as_str()), (1, Some("none")), "{x}");
    assert_eq!(hub.main(&["renew", "--session", "S-X", "--pid", "1111"]).0, 1);
    assert_eq!(hub.main(&["am-i", "--session", "S-A"]).0, 0, "S-A still main");

    // Same session, different process: renew keeps it.
    let (rc, r) = hub.main(&["renew", "--session", "S-A", "--pid", "2222"]);
    assert_eq!((rc, r["action"].as_str()), (0, Some("renewed")), "{r}");
    assert_eq!(r["generation"].as_u64(), Some(generation));

    // Same session, a third process (the SessionEnd hook): release frees it at once.
    let (rc, rel) = hub.main(&["release", "--session", "S-A", "--pid", "3333", "--agent", "hook-proc"]);
    assert_eq!((rc, rel["action"].as_str()), (1, Some("released")), "{rel}");
    let (_, s) = hub.main(&["status"]);
    assert!(s["holder"].is_null(), "lease vacant after release: {s}");
    assert_eq!(s["generation"].as_u64(), Some(generation));
    let release = s["recent"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|r| r["event"]["kind"] == "release")
        .unwrap_or_else(|| panic!("release recorded: {s}"))["event"]
        .clone();
    assert_eq!(release["generation"].as_u64(), Some(generation));
    assert_eq!(release["holder"]["session"], "S-A");
    assert_eq!(hub.main(&["am-i", "--session", "S-A"]).0, 1);
}
