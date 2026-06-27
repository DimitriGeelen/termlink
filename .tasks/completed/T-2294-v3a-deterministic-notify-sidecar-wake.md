---
id: T-2294
name: "V3a: deterministic notify (sidecar wake)"
description: >
  RC3a fix. No-LLM sidecar listener (AEF ADR §5): remote-write -> local flag/KV + heartbeat timestamp; turn-based agent cooperatively polls local flag at its yield points; stale timestamped-delta => deaf => stop before acting (self-check-ears); sender missing-ack => retry. The flag is a file/KV, not a keystroke; determinism = the timestamp (absent fresh delta IS the signal). Replaces the preemptive PTY doorbell (T-1800, miss-mid-turn = T-2285 gap). Homes: kv, agent-presence, T-2051 offline-queue. ACs: idle turn-based agent woken deterministically on new mail; stale-delta self-check halts an agent whose listener died; NO preemptive mid-turn injection; missing-ack triggers retry.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:reliable-comms]
components: []
related_tasks: [T-2291, T-2292, T-2293]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-27T17:06:22Z
last_update: 2026-06-27T20:32:45Z
date_finished: 2026-06-27T20:32:45Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2294: V3a: deterministic notify (sidecar wake)

## Context

Arc-003 (reliable-comms) slice. RC3a from the T-2291 inception RCA: the recipient has no deterministic way to learn a message arrived. The operator-specified mechanism is a no-LLM sidecar listener (AEF ADR §5): remote-write → local flag/KV + heartbeat timestamp; the turn-based agent polls the local flag at yield points; a stale timestamped-delta means the listener is deaf, so the agent self-checks and halts before acting. Replaces the preemptive PTY doorbell (T-1800, miss-mid-turn = the T-2285 gap). Depends on T-2293 (discovery). Design trail: `docs/reports/T-2291-cross-agent-comms-inception.md`; `docs/architecture/parallel-execution-substrate.md` §5.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] An idle turn-based agent is woken deterministically on new mail by polling the local flag/KV at a yield point (no LLM in the sidecar)
      — `notify-check.sh` returns exit 10 (MAIL) when the sidecar's flag shows pending>0. Proven LIVE: real DM (unread=2) → `notify-sidecar.sh` real-probe → flag `pending=2` → check exit 10. Test T8/T12.
- [x] The notify carries a heartbeat timestamp; a stale timestamped-delta is detectable as "listener is deaf"
      — sidecar writes `<agent>.heartbeat` (epoch-ms) every cycle; `notify-check.sh` computes the delta. Test T13/T14.
- [x] Self-check-ears: an agent whose listener died halts before acting on a stale flag rather than proceeding blind
      — `notify-check.sh` exits 3 (DEAF) + "HALT" when heartbeat is stale (>--deaf-after) OR missing. Test T10/T11.
- [x] No preemptive mid-turn injection occurs (the flag is a file/KV, read cooperatively — not a keystroke)
      — flag is a file under `~/.termlink/notify/`; zero `inject`/pty/`--enter` in either script (greppable structural guard in Verification).
- [x] A missing ack from the recipient triggers a sender retry
      — reuses the shipped T-2286 `channel post --await-ack --retry`. Proven LIVE: 2 attempts to a non-acking peer → loud non-zero exit ("did not ack after 2 attempt(s); awaiting-ack row retained for retry/recovery").

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# V3a deterministic notify sidecar — hub-independent test suite (14 cases).
bash scripts/test-notify-sidecar.sh
# AC4 structural guard: no preemptive injection in the notify path.
! grep -qE "\binject\b|pty[_-]|command\.inject|--enter" scripts/notify-sidecar.sh scripts/notify-check.sh
# Both scripts must be executable.
test -x scripts/notify-sidecar.sh && test -x scripts/notify-check.sh

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

### 2026-06-27 — V3a is script-tier, not a new CLI subcommand
- **What changed:** The surface map confirmed every substrate home is shipped
  (the local flag dir, agent-presence liveness, `channel unread` for mail). The
  novel part is just the flag+heartbeat protocol + the self-check verdict — which
  is a pair of shell scripts, not Rust. The Explore agent's "add `KvAction::Watch`
  CLI + extend ack_retry.rs" suggestion was over-scope.
- **Plan impact:** No new CLI/crate code. Deliverable = `notify-sidecar.sh` +
  `notify-check.sh` + `test-notify-sidecar.sh` + ops doc. AC5 reuses the shipped
  T-2286 `channel post --await-ack --retry` verbatim.
- **Triggered:** Decision "file not kv" and "no kv-watch" (see Decisions).

### 2026-06-27 — the V1/V2 binary was never actually deployed (build footgun)
- **What changed:** During the AC5/AC1 live proofs the installed `termlink`
  (0.11.78) signed every post as the SHARED host fp `d1993c2c…` regardless of
  `TERMLINK_AGENT_ID` — i.e. V1 per-agent identity was not engaging. Root cause:
  `cargo build --release -p termlink-cli` errors (`package … did not match any
  packages`; the package is named `termlink`, not `termlink-cli`), and a
  `| tail` pipe masked cargo's non-zero exit (PIPESTATUS), so the "fresh build"
  silently no-op'd and a stale pre-V1 binary got installed. The V1/V2 SOURCE was
  correct and committed all along.
- **Plan impact:** Built the correct package (`-p termlink`, 9m53s), installed
  0.11.95 (= VERSION), and re-verified V1 LIVE: agent-A `4ce77aa5`, agent-B
  `869bfcc6`, shared `d1993c2c` — all distinct, per-agent keys created under
  `~/.termlink/identities/`. V1 and V2 are now genuinely deployed, not just
  source-complete. Note: `/preflight` Check 4 (binary-vs-VERSION) is the existing
  guard that would have caught this (0.11.78 < 0.11.95).
- **Triggered:** Build/install lesson worth a learning (wrong `-p` name + pipe-
  masked exit). Candidate follow-up: a `make install`/`fw` target that builds
  `-p termlink` and checks PIPESTATUS so the footgun can't recur.

### 2026-06-27 — sidecar probe jq bug + inherited cold-DM off-by-one
- **What changed:** First AC1 live proof returned `pending=0` for a genuinely
  unread DM. Two distinct causes: (1) a real bug — `channel list --json` returns
  `{"topics":[…]}` (object) but the probe used `.[]?.name` which errors on the
  object form and silently yielded zero topics; fixed to `(.topics // .)[]?.name`.
  (2) inherited primitive behaviour — `channel unread --sender X` with no receipt
  frontier defaults to the first offset and counts *past* it, so the very first
  message in a cold DM is not counted (the same property `/check-arc` has).
- **Plan impact:** jq fixed + re-proven (real DM unread=2 → MAIL exit 10). The
  off-by-one is documented as an inherited property, NOT patched here — fixing it
  in `channel unread` once would benefit both surfaces; out of scope for V3a.
- **Triggered:** Doc § "Known property: cold-DM off-by-one".

## Decisions

### 2026-06-27 — local FILE for the flag, not `termlink kv`
- **Chose:** flag + heartbeat as plain files under `~/.termlink/notify/`.
- **Why:** `kv` is session-scoped, in-memory, and hub-mediated (per-session
  HashMap, lost on session exit, needs the hub to read). The self-check must stay
  trustworthy precisely when the hub is down — that is exactly when an agent most
  needs to learn its listener went deaf. A local file read has no hub dependency
  and mirrors the offline-queue's `~/.termlink/` path discipline. Writes are
  atomic (temp+rename) so a yield-point read never sees a half-written flag.
- **Rejected:** session-KV (hub dependency defeats the antifragile property).

### 2026-06-27 — no `kv watch` / event stream; cooperative poll only
- **Chose:** the agent polls the local flag at its own yield points.
- **Why:** the §5 design is explicitly "neither bus-polling nor preemptive." A
  watch/event stream is a preemptive push — the exact thing the PTY-doorbell
  miss-gap (T-2285) taught us to avoid. Determinism comes from the heartbeat
  timestamp (absence of a fresh delta IS the signal), not from a live stream.
- **Rejected:** adding `KvAction::Watch` consuming the hub's `kv.change` event
  (Explore agent's suggestion) — over-scope and against the cooperative model.

### 2026-06-27 — AC5 reuses T-2286, no new retry code
- **Chose:** missing-ack→retry = `channel post --await-ack --retry` verbatim.
- **Why:** T-2286 already ships durable await-ack-with-retry (SQLite tracker,
  exactly-once via T-2049 dedupe). Rebuilding it would duplicate a load-bearing
  primitive. The recipient-side auto-ack that satisfies it is V3b's job (T-2295);
  the sidecar is its natural home.
- **Rejected:** a bespoke V3a retry loop.

### 2026-06-27 — PTY doorbell stays until V6
- **Chose:** add the sidecar as the deterministic *alternative*; leave
  `agent-send.sh`/`agent-respond.sh` doorbell in place.
- **Why:** ripping out the load-bearing doorbell mid-arc is risky; the send-path
  restructure (flag-drop-only + per-conversation journals) is V6's (T-2296) scope.
  V3a delivers the opt-in deterministic mechanism today without destabilising the
  current RECEIVE path.
- **Rejected:** replacing the doorbell ring in `agent-send.sh` now (defer to V6).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-27T17:06:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2294-v3a-deterministic-notify-sidecar-wake.md
- **Context:** Initial task creation

### 2026-06-27T17:57:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-27T20:32:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
