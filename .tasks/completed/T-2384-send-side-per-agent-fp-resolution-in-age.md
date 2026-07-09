---
id: T-2384
name: "Send-side per-agent fp resolution in agent contact — resolve recipient per-agent identity fp, not shared host fp (comms loud-contract; fixes silent DM mis-route on shared hosts, T-2380 #4)"
description: >
  cmd_agent_contact local Ok(reg) branch (agent.rs:1099) returns reg.metadata.identity_fingerprint verbatim = shared host fp on co-resident hosts, so name-based DMs land on the wrong dm topic and the right agent never wakes. Route the target name through the agent identity --resolve precedence (mirror be-reachable.sh:284-292) instead.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/agent.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-09T09:29:02Z
last_update: 2026-07-09T11:13:29Z
date_finished: 2026-07-09T11:13:29Z
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

# T-2384: Send-side per-agent fp resolution in agent contact — resolve recipient per-agent identity fp, not shared host fp (comms loud-contract; fixes silent DM mis-route on shared hosts, T-2380 #4)

## Context

T-2380 GO, link #4 (the addressing prerequisite). `cmd_agent_contact`
(`crates/termlink-cli/src/commands/agent.rs:1099`, the local `Ok(reg)` branch)
resolves a bare peer NAME to `reg.metadata.identity_fingerprint` — the local
session registration's fp, which on a shared host is the **shared host fp**
(PL-166/236), NOT the per-agent identity the recipient's be-reachable dm-rail
waker subscribes on. Result: name-based `agent contact <name>` lands on the wrong
`dm:<a>:<b>` topic and the right agent never push-wakes. The remote path
(`resolve_contact_via_fleet`, line 1078-1082) already uses the **presence-advertised**
fp — the correct per-agent identity. Fix: make the local path resolve the same
per-agent fp the recipient's waker listens on. Observed live in T-2381 (host fp
`d1993c2c` vs correct per-agent `dcd44820`).

### Identity model — CONFIRMED (2026-07-09, ready to implement)

Traced end-to-end:
- **Q1 presence-advertised fp** = the `agent-presence` heartbeat envelope's
  top-level **`sender_id`** (signing identity). `resolve_contact_via_fleet` →
  `resolve_agent_presence` reads it: `agent.rs:770,778,785`; parser contract
  `crates/termlink-session/src/fleet_presence.rs:16,44-48,131-135,165`.
- **Q2 heartbeat SIGN fp** = per-agent. `scripts/listener-heartbeat.sh:147`
  exports `TERMLINK_AGENT_ID` before emit (:167); `be-reachable.sh:234,244`
  feeds it → signed with `~/.termlink/identities/<agent_id>.key`.
- **Q3 waker SUBSCRIBE fp** = per-agent. `be-reachable.sh:290-292`
  `self_fp=$(agent identity --resolve --json | jq -r .fingerprint)` (with
  `TERMLINK_AGENT_ID` set) → `:299 --self-fp`; pushwaker dm-rail subscribes on it
  `be-reachable-pushwaker.sh:179-180`, matches `addressee==self_fp` (:88-90).
- **Q4 registration metadata fp** = HOST key. `agent.rs:1099`
  `reg.metadata.identity_fingerprint` set at
  `crates/termlink-session/src/registration.rs:291`
  `load_identity_fingerprint_best_effort()` — runs in the `termlink register`
  process env WITHOUT `TERMLINK_AGENT_ID` (that export happens later, only in
  `be-reachable.sh start`, a separate process) → falls through to
  `$HOME/.termlink/identity.key` (host key, registration.rs:49,64-74).

**CRUX (confirmed): Q1 == Q2 == Q3 = per-agent fp; Q4 = host fp.** Presence-fp and
waker-subscribe-fp are equal *by construction* (same env, same key resolver) — they
cannot drift. So addressing the DM to the presence fp hits the waker's subscribe
topic; addressing to Q4 (host fp) silently misses the dm rail (only the
`inbox.queued`/`--inbox-id <agent_id>` rail, which is fp-independent, still fires).

### Fix spec (next session — one edit)

In `cmd_agent_contact` (`agent.rs`), the `Ok(reg)` branch (~1099): before using
`reg.metadata.identity_fingerprint`, consult presence for this `target_name` and
**prefer the presence-advertised fp** when found (reuse the same
`resolve_agent_presence`/`resolve_contact_via_fleet` source the `Err`/remote path
uses — ideally scope the read to the LOCAL hub first for latency, since a local
`Ok(reg)` peer is co-resident). Fall back to `reg.metadata.identity_fingerprint`
only when presence has no live entry (don't hard-fail — a registered-but-not-
be-reachable peer should still resolve, just via the host fp as today). Extract a
small pure precedence helper `prefer_presence_fp(presence_fp: Option<..>,
reg_fp: Option<..>) -> ..` and unit-test: presence-fp preferred when both present
& differ; reg-fp used when presence absent; error when both absent. Keep
`--target-fp` path (line 1060-1070) untouched.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Identity model confirmed (see RCA): presence-advertised fp (Q1) == waker-subscribe fp (Q3) = per-agent resolved key; registration metadata fp (Q4) = host key. Fix premise holds.
- [x] Fix: name-resolved local peer's target fp resolves to the per-agent presence-advertised identity (the fp the waker listens on), not the shared host registration fp — reusing the presence source the remote path already uses. (`Ok(reg)` branch now calls `resolve_contact_via_fleet` and `prefer_presence_fp(presence, reg)`.)
- [x] Verified: a name-based `agent contact <name>` to a co-resident armed agent computes the SAME `dm:<a>:<b>` topic the recipient's be-reachable dm rail subscribes to (so it push-wakes). Manual repro on .107: A/B dry-run — TREATMENT `framework-agent-systemd` (presence heartbeat under per-agent key `3bba15e681b3a078`) → `peer_fp=3bba15e681b3a078`, `topic=dm:3bba15e681b3a078:d1993c2c3ec44c94` (the waker's subscribe topic); CONTROL `email-archive` (co-resident, no presence) → `peer_fp=d1993c2c3ec44c94` (host fp fallback). Both share reg fp = host key `d1993c2c`; only the live presence heartbeat flips the resolution.
- [x] No regression: single-identity hosts (registration fp == per-agent fp) unchanged; the explicit `--target-fp` path unchanged (lines 1060-1070, not touched); a peer with no presence entry still falls back to registration metadata (control dry-run above returned the reg/host fp, not a hard failure).
- [x] `cargo build --release -p termlink` succeeds (9m41s, exit 0); unit test `contact_tests::prefer_presence_fp_precedence` covers the precedence helper (presence-fp preferred when both differ; reg-fp fallback when presence absent; equal-fp no-op; both-absent → None). Test: `1 passed; 0 failed`.

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

# T-2384: the precedence-helper unit test (bin unit-test binary — `-p termlink`
# alone runs only the integration binaries; the module tests live in the bin).
# Capture-then-grep per L-387 (single pipe, no SIGPIPE).
out=$(cargo test --release -p termlink --bin termlink prefer_presence_fp 2>&1); echo "$out" | grep -q "1 passed"

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

**Symptom:** A name-based `agent contact <name>` to a co-resident agent on a
shared host wrote the doorbell DM to a topic the recipient's push-waker was NOT
subscribed on — so the message landed durably (`offset N`) but never woke the
recipient. On .107 all co-resident sessions share one host identity key
(`d1993c2c`, PL-166), so the computed topic degenerated to a self-topic
(`dm:d1993c2c...:d1993c2c...`) instead of the recipient's per-agent rail.

**Root cause:** `cmd_agent_contact`'s local-session branch (`Ok(reg)`) addressed
`reg.metadata.identity_fingerprint` — the fingerprint captured at *registration*.
The register process runs without `TERMLINK_AGENT_ID`, so
`load_identity_fingerprint_best_effort()` falls back to the **host** key. But the
recipient's push-waker (`/be-reachable`'s dm rail) subscribes on its **per-agent**
key — the same key its `agent-presence` heartbeat advertises as `sender_id`. The
send side read one identity source (registration metadata = host key) while the
receive side listened on another (presence = per-agent key); on a shared host they
diverge, so the two never met. (The remote/`Err` branch already resolved via
presence — only the local-session branch had the bug.)

**Why structurally allowed:** the two identity sources were never reconciled at the
addressing seam. Registration metadata and presence heartbeats are populated by
different code paths (register subprocess vs. heartbeat under `TERMLINK_AGENT_ID`);
nothing asserted "the fp you *address* must equal the fp the recipient *subscribes*
on." Single-identity hosts masked it (both fps equal), so it only manifested on
shared hosts — and silently, because the write always succeeded.

**Prevention:** (1) the fix makes presence the authoritative send-side source for
locally-registered peers, reconciling the seam; (2) `prefer_presence_fp` is a pure,
unit-tested precedence helper so the rule can't silently regress; (3) this is the
addressing link of the T-2380 loud-delivery-contract arc — T-2385's preflight will
additionally verify `waker_running` end-to-end so a broken rail *fails loud* rather
than returning a silent `offset N`.

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

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-09T09:29:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2384-send-side-per-agent-fp-resolution-in-age.md
- **Context:** Initial task creation

### 2026-07-09T09:40:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-02eb78f4
- **Timestamp:** 2026-07-09T11:19:37Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-09T11:13:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
