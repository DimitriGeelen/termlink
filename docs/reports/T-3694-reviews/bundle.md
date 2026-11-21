# Review bundle: how one agent pair reports prompt origin to the main-session lease

## 1. The system (as built, verified on the host 2026-11-21)

TermLink is a messaging substrate for coding agents (Claude Code sessions). A project can have several sessions on
one agent identity. Mail must reach exactly ONE of them, the project's **main** session. Which session is main is a
**lease** held on the project's home hub (`termlink main claim|renew|release|status|am-i`). Hooks in the agent's
harness claim or renew the lease on activity.

**The tightening (plan unit P-4, built and tested, task T-3694).** A claim may say where the prompt came from:
`--origin human|automated|unknown` (plus `--basis`, `--kind prompt|session-start|tool`). The operator switches a
project's harness to "origin reporting" with `termlink main origin-reporting --harness claude-code` (a record on the
hub, never set by a claim). Once switched on:

1. only a `human` prompt renews main; automated prompts (wake-ups, task notifications, headless workers) do not;
2. a human prompt in another session can take main over quickly ("fast takeover") once the holder's last human
   prompt is old enough;
3. a claim with **no origin field is treated as `unknown` and IGNORED** (`main_lease.rs`, FCW-53, quoted in 4.3).

Before the switch, every claim is decided as before ("built behaviour", origin shown as `absent`). The **origin is
checked by the receiver, not trusted from the hook**: the hook sends a `prompt_submit` report to its agent's own
TermLink sidecar (a local daemon on a unix socket), the sidecar's meter classifies the prompt (from the harness's
prompt record and the PTY input counters), and replies `origin_checked` / `basis_checked`; the hook then claims with
that origin.

**Two hook stacks exist in the harness's settings file.**

1. **AEF hooks** (the framework vendored into every project, owned by another team, "AEF"): `UserPromptSubmit` runs
   AEF's `sidecar-inbox` hook, which claims the main lease on every prompt and surfaces waiting mail only in main.
   AEF's origin reporter "adapter/2" is the component that would send the `prompt_submit` report and pass the origin
   on the claim.
2. **TermLink's own "W1" hooks** (`termlink sidecar hook <event>`), used by this project's two production agents
   (the "W1 pair"): events session-start, post-tool-use, stop, session-end. They deliver mail and report turn events
   to the sidecar's meter. **They have no prompt-submit event and never claim the main lease.**

**AEF's guard.** adapter/2 switches itself off whenever TermLink's W1 hooks appear in a settings file ("never both
on one session"; quoted in 4.1), so a session never reports twice.

## 2. The problem (measured 2026-11-21)

On the project where the W1 pair runs (the project meant to prove the whole design end to end):

- `termlink main status` shows `origin: "absent"`, `last_human_at_ms: null`, `origin_reporting: []`, note "this
  harness's hooks do not report prompt origin; its claims count as built". Main is held (generation 3) by the
  operator's interactive session; `am-i` returns 0 there.
- Nothing in this session can report prompt origin: adapter/2 is off by its guard, and the W1 hooks have no prompt
  event. If the operator switched origin reporting on today, every claim (all origin-less) would be ignored and no
  session could take main.
- T-3694's last open criterion is "`termlink main status` shows `origin` and `last_human_at` on the W1 pair".

Other projects (which run only AEF hooks) can use adapter/2 and are unaffected. Not verified: whether the meter's
PTY-counter source (S1) is bound for the W1 pair's sessions; an unbound S1 makes the checked origin `unknown`, never
`human`.

## 3. Options as the agent framed them

- **A — TermLink adds a W1 prompt hook (TermLink code only).** New `termlink sidecar hook prompt-submit`: sends the
  `prompt_submit` report to its own sidecar, gets the checked origin, claims main with it. AEF's origin-less claim
  keeps running; after the switch it is ignored (shows in the lease's `ignored_recent` list).
- **B — AEF lets adapter/2 run beside the W1 hooks for prompts only (AEF change).** AEF narrows its guard.
- **C — AEF's claim asks TermLink's sidecar for the origin (AEF change, small).** When W1 hooks are present, AEF's
  `sidecar-inbox` hook sends one `prompt_submit` report to the W1 sidecar socket (`TERMLINK_SIDECAR_SOCKET`) and
  passes `origin_checked` on its claim. One claimant, no new hook. (Note: mechanically this is "adapter/2's
  prompt report only, into the W1 socket", so it sits close to B.)
- **D — Leave it.** Record that the W1 pair stays origin-absent (built behaviour) and close T-3694 without the
  tightening there. Changes the criterion; needs the operator.

Agent's scores (−2..+2 × weight; drivers D1 Antifragility 9, D2 Reliability 7, D3 Usability 5, D4 Portability 3,
F-AUTONOMY "unattended operation" 4): A +32, B −3, C +36, D −13. Agent recommends **C, with A as fallback if AEF
cannot take it within about a week**. A ruling would let the agent send AEF the request and build TermLink's side.
Switching origin reporting on for the project would come back to the operator as its own approval.

Standing rules that bind any option: nothing working is switched off without a proven replacement and the
operator's documented approval; TermLink never edits AEF's code (requests only); every operator action ships as one
idempotent, self-verifying script action.

## 4. Code excerpts

### 4.1 AEF adapter/2 guard (`lib/sidecar/adapter2.py`)

```python
def enabled() -> bool:
    """On only when the switch is 1, a socket is known, and no settings file runs TermLink's
    own sidecar hooks (never both on one session; a W1 hit is logged and keeps it off)."""
    if _config("TERMLINK_ADAPTER2") != "1" or not socket_path():
        return False
    w1 = w1_hook_files()
    if w1:
        ...  # logged as "w1-guard"
        return False

def w1_hook_files() -> list[str]:
    """Settings files that run TermLink's own sidecar hooks for this session."""
    # scans the project's settings file, its per-machine override file, and the user's settings file for the W1 hook command
```

### 4.2 AEF prompt hook (`lib/sidecar/hooks.py`, `prompt()`)

```python
    # T-5054: adapter/2 on -> BUSY + prompt_submit go to TermLink for every session
    # (headless ones report automated/non-interactive), and the claim below carries the
    # RECEIVER-CHECKED origin (17.5.e). Off -> None, and the claim is exactly T-5014's.
    a2 = adapter2.hook_reports("prompt", hook_input)
    origin_kw = {} if a2 is None else {k: a2[k] for k in ("origin", "basis", "kind")}
    ...
    if me.get("headless"):
        return []
    ...
    claimed = (main_session.claim(my_sid, **origin_kw) if main_session.may_claim(me)
               else main_session.INELIGIBLE)
```

`main_session.origin_args()` turns `origin/basis/kind` into `--origin/--basis/--kind` on `termlink main claim`;
empty when no origin is given ("the claim is then exactly today's (origin absent, counted as built until the
operator switches the harness to origin reporting)").

### 4.3 Lease decision (`crates/termlink-session/src/main_lease.rs`)

```rust
/// B-25 tightened plan (AL-11 table), for a harness the operator switched to origin reporting.
pub fn plan_reporting(v: &View, f: &Folded, me: &Holder, req: &ClaimRequest, last_human: Option<u64>) -> Plan {
    let origin = req.origin.unwrap_or(Origin::Unknown); // FCW-53: absent in a reporting harness is unknown
    match req.kind {
        ClaimKind::Tool => return Plan::AnswerOnly { why: "tool activity never renews main" },
        ...
// test: In a reporting harness an absent origin is unknown: ignored (FCW-53).
assert_eq!(plan_reporting(&v, &f, &h("s1"), &req(ClaimKind::Prompt, None), Some(0)), Plan::Ignored);
// test: a claim with an origin never switches a harness; only an OriginReporting record does, newest per harness wins.
```

### 4.4 TermLink sidecar: receiver-checked origin (`crates/termlink-sidecar/src/api.rs`, `meter.rs`)

```rust
// on the sidecar's send socket: an adapter report line, parsed strictly by the meter
let applied = m.intake(raw);
let ok = matches!(applied, crate::meter::Intake::Applied);
let mut v = report_reply(applied);
// CR-6.4 (T-3694, E-153): the hook claims the lease with the receiver-checked origin.
if ok && let Some((o, b)) = m.checked_origin_of(raw) {
    v["origin_checked"] = Value::from(o);
    v["basis_checked"] = Value::from(b);
}

pub fn checked_origin_of(&self, raw: &[u8]) -> Option<(&'static str, String)> {
    let v: Value = serde_json::from_slice(raw).ok()?;
    if v.get("event").and_then(Value::as_str) != Some("prompt_submit") { return None; }
    let p = self.last_prompt(v.get("session_id").and_then(Value::as_str)?)?;
    Some((p.origin.name(), p.basis))
}
```

`crates/termlink-sidecar/src/hook.rs` already has `pub fn checked_origin(socket, report, timeout) -> (origin, basis)`
(any failure is `(unknown, absent)`), used today only by tests and the meter-only trial sidecar.

### 4.5 W1 hook stack (`crates/termlink-sidecar/src/hook.rs` header)

```
| event | does |
| post-tool-use | pull; prints the frame as additionalContext when one came back, then spawns a detached finalizer |
| session-start | ready (records the session) |
| vendor-channel | PreToolUse for SendMessage/Agent/Task: one admin row per call; never blocks |
| stop | ready only |
Fail directions: the turn fails open (any error prints nothing, exits 0, one line to hook-errors.log);
the delivery fails closed.
```

Settings file of the W1 project, hooks per event: SessionStart → AEF sidecar-autostart + `termlink sidecar hook
session-start`; PostToolUse → `termlink sidecar hook post-tool-use`; UserPromptSubmit → AEF `sidecar-inbox` + AEF
`sidecar-receiver-adapter`; Stop → AEF `sidecar-receiver-ready` + `termlink sidecar hook stop`; SessionEnd →
`termlink sidecar hook session-end`.
