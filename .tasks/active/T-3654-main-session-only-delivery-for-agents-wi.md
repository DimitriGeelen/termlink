---
id: T-3654
name: "Main-session-only delivery for agents with several sessions on one identity
  (ring20 escalation 2026-11-02)"
description: >
  Inception: Main-session-only delivery for agents with several sessions on one identity
  (ring20 escalation 2026-11-02)

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-11-02T12:11:07Z
last_update: 2026-11-08T15:40:30Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score:                        # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
bvp_scores_proposed:
  - ts: '2026-11-02T12:44:13Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
    rationale: D1=2 (body:learning-ref); D2=0 (no-signal); D3=0 (no-signal); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3654: Main-session-only delivery for agents with several sessions on one identity (ring20 escalation 2026-11-02)

## Context

**Background / trigger:** 2026-11-02 Pen's request for ring20 (dm offset 612, the operator's fax password) was taken
and claimed by a 17-day-old ring20 background session (ce21a52f); the operator's session (afce1d16) never saw it.
ring20 escalated (msg f873982f). The protocol for this was RULED on 2026-10-06 (OD-18, requirements R-67.a / R-67.a1 /
R-68 in docs/design/interactive-agent-communication-01-requirements.md): one main per project under a lease at the
home hub; a non-main copy that receives role mail forwards it once to the main; no valid holder -> the copies settle
by operator pin, else longest uninterrupted holder, else stable id; "who holds main" visible, override audited.
Only the visible half was built (T-3619/T-3646/T-3651). Operator, 2026-11-02: "that's a protocol we designed;
TermLink should have designed, dispatched and rolled that out" — build it now, for every project (supersedes the
T-3359 ruling-C postponement of the lease).

**Problem / hypothesis:** nothing enforces main-only consumption. Every session of an agent identity can surface,
claim and answer its mail (AEF's receiver hook hands it to whichever session prompts first, AEF confirmed, AEF T-5014).

**Goal:** a project-generic main lease at the home hub that any session of any project can claim/renew/query, with
the R-67 selection order and takeover rules, and a single "am I main?" answer per SESSION that AEF's hook (T-5014)
and our sidecar use: only main surfaces, claims or answers; others leave mail unread and point to main.

**Expected outcome:** in a project with two live sessions, only the lease holder sees and handles peer mail; the
other shows one line "held by main <session>"; when main dies, the next session takes over after lapse + quiet
period + cooldown, visibly; an operator pin wins and is logged.

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
- [x] Main lease per project at the home hub, generic (any project, any agent id): claim / renew / release / status,
      holder = one SESSION (session id + pid + host), durable generation never reused, built on the existing hub
      claim/lease primitives where possible (no new protocol if the claim substrate suffices)
- [x] Selection and takeover per R-67.a/a1: operator pin first; else healthy incumbent keeps it (busy keeps main, no
      automatic take-back); takeover only after lapse + quiet period + cooldown (constants stated in Decisions);
      duplicate claimants resolved by pin, else longest uninterrupted holder, else stable id; every change and every
      override written to an audit record and visible in status
- [x] One answer for hooks: `termlink main am-i --session <id>` (and the sidecar status socket / MCP) returns
      main | not-main(holder) | unknown; fail-closed (hub down -> unknown, never main)
- [ ] Non-main copy behaviour (R-67.a1 (4)): role-addressed mail seen by a non-main is left unread for main / forwarded
      once to main with the original signature; the sender is told "delivered via X; main is Y (generation N)"
- [x] Tests: two-session fixture on a throwaway hub: only the holder handles mail; holder dies -> takeover after the
      stated delay with generation+1; pin overrides; hub down -> unknown; plus the two R-68 tests (lapse during an open
      obligation; stuck holder with live sidecar) end in a visible takeover
- [x] Interface handed to AEF for T-5014 (exact command / socket call and outputs) and to ring20
      (2026-11-03: AEF shipped T-5014 against it in v1.8.16-be.26, vendored here by T-3659: lib/sidecar/main_session.py
      calls `termlink main am-i`/`claim`/`release`; ring20 LIVE on it, msg 25bff094 main afce1d16 gen 1)

      Evidence (2026-11-02, worker t3654-main-lease): lease = hub claim on offset 0 of `main:<project>` + signed
      `main.lease.v1` records on that topic (crates/termlink-session/src/main_lease.rs); CLI `termlink main
      claim|renew|release|status|am-i|pin`; sidecar status `main.lease` + pull gate (crates/termlink-sidecar/src/lease.rs);
      MCP `termlink_main_status`, and `termlink_sidecar_main` now carries `main.lease`. Tests: main_lease unit 10/10,
      sidecar lease unit 2/2 (the "only the holder handles mail" gate), crates/termlink-cli/tests/main_lease_integration.rs
      5/5 on throwaway hubs (two sessions + takeover after lapse+quiet with generation+1, pin override, hub down ->
      unknown, R-68 (1) open obligation recorded uncertain->operator, R-68 (2) stuck holder with a live reader).
      AC4 PARTIAL, left unticked: same-sidecar non-main sessions leave role mail unread for main and are told once who
      main is; forward-once across sidecars with the original signature, and the sender notice "delivered via X; main
      is Y (generation N)", are NOT built (needs a new stage notice kind + AEF T-5014). 2026-11-03: that remainder is
      split out as T-3669 (horizon later; no trigger while every project runs on one host). AC6 left unticked: the interface
      is in docs/operations/main-lease.md and .context/working/t3654/handback.md, not yet sent to AEF / ring20.

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.
cargo test -p termlink-session --lib main_lease > .context/working/.t3654-unit.out 2>&1 && grep -q "test result: ok. 10 passed" .context/working/.t3654-unit.out
cargo test -p termlink-sidecar --lib lease:: > .context/working/.t3654-sidecar.out 2>&1 && grep -q "test result: ok. 2 passed" .context/working/.t3654-sidecar.out
cargo test -p termlink --test main_lease_integration > .context/working/.t3654-int.out 2>&1 && grep -q "test result: ok. 5 passed" .context/working/.t3654-int.out
test -f docs/operations/main-lease.md

## Recommendation

**Recommendation:** DEFER

**Rationale:**

Shaping in progress. ring20 escalation (msg f873982f): Pen's fax-password request to proxmox-ring20-management was consumed and claimed by a 17-day-old background session (ce21a52f) instead of the operator's main session (afce1d16); all ring20 sessions share one TermLink identity and each runs a sidecar receiver/prompt hook. Our arc-011 already pins a main in the sidecar address book (T-3619 slice 1, liveness T-3646, MCP T-3651), but enforcing main-only consumption is the PH-4 lease that operator ruling C (T-3359) deferred; the surfacing hook is AEF's. Needs measurement of the current consume path and an operator decision.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

### 2026-11-02 — main-lease constants (R-67.g left them open)
- **Chose:** lease 15 min (renewed by the session's own hook), quiet period 5 min after a lapse, cooldown 10 min
  between generation changes; a clean release skips the quiet period. Worst-case takeover after a holder dies: 20 min.
  Test seams only: `TERMLINK_MAIN_TEST_LEASE_MS|_QUIET_MS|_COOLDOWN_MS`.
- **Why:** conservative: a pausing operator session must not lose main (no automatic take-back makes a loss sticky),
  and a flapping pair must not churn the generation. Hub claim TTL bounds: 1 s .. 1 h.
- **Rejected:** a sidecar-renewed lease (R-67.c: proves a process, not a turn); minute-scale constants (flap risk).

### 2026-11-02 — build on the claim substrate, records in the same topic
- **Chose:** the lease is `channel.claim` on offset 0 of `main:<project>` (retention newest 1000); acquire, release,
  lapse and pin are signed `main.lease.v1` posts on that topic; generation = highest acquire generation in the log.
- **Why:** AC1 "no new protocol if the claim substrate suffices": the hub claim is the mutual exclusion, the log is
  durable and auditable. Quiet/cooldown are enforced by the client verbs every caller uses (a client that bypasses
  them and claims offset 0 raw is still named as holder, never mistaken for anyone else).
- **Rejected:** a new hub RPC (protocol change + rollout to every hub); a KV value (no history, no audit).

### 2026-11-02 — sidecar gate fails open, `am-i` fails closed
- **Chose:** the sidecar withholds a pull only when a FRESH lease names another session of the same agent; unknown,
  stale, vacant, or another agent's main leaves delivery unchanged. `termlink main am-i` is unknown (exit 2) on any error.
- **Why:** a lease outage must not stop all mail; the hook contract still never reports main without proof.

### 2026-11-03 — T-3668 fix ships with the next normal release (operator: "A")
- **Chose:** A, wait for the next real release; no runme install on .107 and no v0.12.5 cut for this fix alone.
- **Why:** the mail-delivery hooks pass `--session` explicitly, so only a bare `termlink main` typed inside Claude Code
  answers wrongly; the workaround (`--session "$CLAUDE_CODE_SESSION_ID"`) is in handover S-2026-1103-2047.
- **Rejected:** B, a runme.sh action to install 0.12.1568 on .107; C, cut v0.12.5 now (1,568 commits since v0.12.4).

### 2026-11-08 — be.26 go for 1023 and 1409 (operator: "go for be.26 on 1023 and 1409")
- **Chose:** both projects may upgrade their vendored AEF to be.26 or later through their own path (1409 via its
  operator runme). Go relayed on `main-only-rollout-2026-11-02` (msgs 31904e02 to 1023, 32fa90a0 to 1409); LIVE only
  after each reports main status held and am-i 0/1.
- **Note:** T-3668 is now installed as a side effect of the 2026-11-08 runme (0.12.1583 on .107), which supersedes the
  wait in the decision above.

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-11-02T12:44:12Z — status-update [task-update-agent]
- **Change:** workflow_type: inception → build

### 2026-11-02T12:44:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
