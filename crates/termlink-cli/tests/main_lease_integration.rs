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
const SLOW_QUIET_MS: u64 = 4_000;

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

    /// `main` with the B-25 test timings: quiet (= provisional validity, = fast-takeover
    /// threshold, V-20) long enough for several CLI calls.
    fn main_slow(&self, args: &[&str]) -> (i32, Value) {
        let mut c = base_cmd(&self.dir.path);
        c.env("TERMLINK_MAIN_TEST_QUIET_MS", SLOW_QUIET_MS.to_string()).env("TERMLINK_MAIN_TEST_LEASE_MS", "8000");
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

// ------------------------------------------------------------------ B-25 (T-3694)

fn acquires_reason(v: &Value) -> Vec<String> {
    recent_acquires(v).iter().map(|a| a["reason"].as_str().unwrap_or("").to_string()).collect()
}

/// E-22 (NID): a harness the operator has not switched keeps the built lease; the switch is per
/// harness, set only by the operator act; after it, that harness's absent claims are ignored
/// while another harness's still renew. Status shows `origin` and `last_human_at_ms`.
#[test]
fn b25_e22_absent_origin_counts_as_built_until_the_operator_switches_the_harness() {
    let hub = start_hub("t3694-nid");
    let (rc, a) = hub.main(&["claim", "--session", "S-A"]);
    assert_eq!((rc, a["action"].as_str()), (0, Some("acquired")), "{a}");
    assert_eq!(a["origin"], "absent");
    assert!(a["note"].as_str().unwrap().contains("does not report prompt origin"), "{a}");
    // A claim carrying an origin never switches the harness (CR-6.6).
    let (rc, a) = hub.main(&["claim", "--session", "S-A", "--origin", "human", "--harness", "h2"]);
    assert_eq!((rc, a["action"].as_str()), (0, Some("renewed")), "{a}");
    // ...and a non-human origin from a harness not switched is recorded but decided as built.
    let (rc, a) = hub.main(&["claim", "--session", "S-A", "--origin", "automated", "--harness", "h2"]);
    assert_eq!((rc, a["action"].as_str()), (0, Some("renewed")), "{a}");
    let (_, s) = hub.main(&["status"]);
    assert_eq!(s["origin_reporting"], serde_json::json!([]), "{s}");
    assert!(s.get("last_human_at_ms").is_some() && s.get("origin").is_some(), "status carries the fields: {s}");
    // The operator act switches h2 only.
    let (rc, o) = hub.main(&["origin-reporting", "--harness", "h2"]);
    assert_eq!((rc, o["action"].as_str()), (0, Some("origin-reporting-on")), "{o}");
    let g = o["generation"].as_u64().unwrap();
    // h1 (old hooks, no origin) still renews: the control against "ignore absent everywhere".
    let (rc, a) = hub.main(&["claim", "--session", "S-A", "--harness", "h1"]);
    assert_eq!((rc, a["action"].as_str()), (0, Some("renewed")), "{a}");
    // h2's absent claim is now unknown: ignored, recorded, lease unchanged.
    let (rc, a) = hub.main(&["claim", "--session", "S-A", "--harness", "h2"]);
    assert_eq!((rc, a["action"].as_str()), (0, Some("ignored: origin")), "{a}");
    assert_eq!(a["generation"].as_u64(), Some(g));
    let (_, s) = hub.main(&["status"]);
    assert_eq!(s["origin_reporting"], serde_json::json!(["h2"]));
    let ig = s["ignored_recent"].as_array().unwrap();
    assert!(ig.iter().any(|r| r["event"]["harness"] == "h2" && r["event"]["origin"] == "absent"), "{s}");
}

/// E-20, E-21, E-23, E-24 on a real hub for a reporting harness: provisional SessionStart,
/// non-human claims ignored, tool activity never renews, Touch + renew, fast takeover after
/// the quiet period of holder human silence (and not before), with the record fields.
#[test]
fn b25_reporting_harness_provisional_ignored_touch_and_fast_takeover() {
    let hub = start_hub("t3694-tight");
    assert_eq!(hub.main_slow(&["origin-reporting"]).0, 0);
    // E-21/E-23: SessionStart on a vacant lease: provisional, main while it holds.
    let (rc, a) = hub.main_slow(&["claim", "--session", "S-A", "--kind", "session-start", "--interactive"]);
    assert_eq!((rc, a["action"].as_str(), a["state"].as_str()), (0, Some("acquired-provisional"), Some("provisional")), "{a}");
    assert_eq!(hub.main_slow(&["am-i", "--session", "S-A"]).0, 0);
    // A second SessionStart (not vacant) and a non-interactive one take nothing.
    let (rc, b) = hub.main_slow(&["claim", "--session", "S-B", "--kind", "session-start", "--interactive"]);
    assert_eq!((rc, b["action"].as_str()), (1, Some("none")), "{b}");
    // E-20: automated and unknown from S-B change nothing, recorded.
    for o in ["automated", "unknown"] {
        let (rc, b) = hub.main_slow(&["claim", "--session", "S-B", "--origin", o, "--basis", "doorbell"]);
        assert_eq!((rc, b["action"].as_str(), b["generation"].as_u64()), (1, Some("ignored: origin"), Some(1)), "{b}");
    }
    // E-23 (a): a human claim takes a provisional holding at once.
    let (rc, b) = hub.main_slow(&["claim", "--session", "S-B", "--origin", "human", "--basis", "pty-input"]);
    assert_eq!((rc, b["generation"].as_u64()), (0, Some(2)), "{b}");
    assert!(acquires_reason(&b).contains(&"over-provisional".to_string()), "{b}");
    // E-21: tool activity never renews or takes.
    let (rc, a) = hub.main_slow(&["claim", "--session", "S-A", "--kind", "tool", "--origin", "human"]);
    assert_eq!((rc, a["action"].as_str()), (1, Some("none")), "{a}");
    // Touch, then renew; last_human_at is shown.
    let (rc, b) = hub.main_slow(&["claim", "--session", "S-B", "--origin", "human"]);
    assert_eq!((rc, b["action"].as_str()), (0, Some("renewed")), "{b}");
    let lh = b["last_human_at_ms"].as_u64().expect("last_human_at_ms after a human renewal");
    // CR-6.21 (T-3694): this hub stamps every lease record, so the age is on the hub's clock.
    let (_, st) = hub.main_slow(&["status"]);
    assert!(st["last_human_rx_ms"].is_u64() && st["hub_now_ms"].is_u64(), "hub receive stamps in status: {st}");
    assert_eq!(st["age_clock"], "hub", "{st}");
    // E-24: inside the quiet period S-A is held off.
    let (rc, a) = hub.main_slow(&["claim", "--session", "S-A", "--origin", "human"]);
    assert_eq!(rc, 1, "fast takeover before the quiet period: {a}");
    // After it (holder silent > quiet): taken at once, generation +1, record names it.
    wait_until(lh + SLOW_QUIET_MS + 150);
    let (rc, a) = hub.main_slow(&["claim", "--session", "S-A", "--origin", "human"]);
    assert_eq!((rc, a["generation"].as_u64()), (0, Some(3)), "{a}");
    let fast = recent_acquires(&a).into_iter().find(|r| r["reason"] == "fast-takeover").expect("fast-takeover record");
    assert_eq!(fast["from"]["session"], "S-B");
    assert!(fast["holder_human_age_s"].is_u64(), "{fast}");
    assert_eq!(fast["origin"], "human");
    // The old holder is told.
    let (rc, b) = hub.main_slow(&["claim", "--session", "S-B", "--origin", "automated"]);
    assert_eq!(rc, 1, "{b}");
    assert_eq!(b["lost"]["to"]["session"], "S-A");
}

/// E-162: a Touch that cannot be posted is no renewal and unknown (exit 2); an Ignored record
/// that cannot be posted leaves the lease unchanged and says `ignored-unrecorded`.
#[test]
fn b25_e162_unpostable_touch_is_unknown_and_unpostable_ignored_is_unrecorded() {
    use termlink_session::main_lease::{ClaimKind, ClaimRequest, Holder, LeaseClient, Origin, Timing, Verdict};
    let hub = start_hub("t3694-e162");
    assert_eq!(hub.main(&["origin-reporting"]).0, 0);
    assert_eq!(hub.main(&["claim", "--session", "S-A", "--origin", "human"]).0, 0);
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    // No signing identity: every post fails, reads work.
    let c = LeaseClient {
        addr: termlink_protocol::TransportAddr::Unix { path: hub.sock.clone().into() },
        auth: None,
        identity: None,
        timing: Timing { lease_ms: LEASE_MS, quiet_ms: QUIET_MS, cooldown_ms: COOLDOWN_MS },
        obligation_topics: vec![],
    };
    let me = Holder { session: "S-A".into(), agent: "agent-t3654".into(), host: String::new(), pid: 1 };
    let human = ClaimRequest { kind: ClaimKind::Prompt, origin: Some(Origin::Human), ..ClaimRequest::default() };
    let before = hub.main(&["status"]).1["claimed_until_ms"].as_u64().unwrap();
    let o = rt.block_on(c.claim_with("t3654-proj", &me, &human));
    assert_eq!(o.verdict, Verdict::Unknown, "{}", o.to_json());
    assert!(o.message.contains("Touch"), "{}", o.message);
    assert_eq!(hub.main(&["status"]).1["claimed_until_ms"].as_u64().unwrap(), before, "no renewal without the Touch");
    let auto = ClaimRequest { origin: Some(Origin::Automated), ..human.clone() };
    let o = rt.block_on(c.claim_with("t3654-proj", &me, &auto));
    assert_eq!((o.verdict, o.action.as_str()), (Verdict::Main, "ignored-unrecorded"), "{}", o.to_json());
}
