---
id: T-2532
name: "per-host live-session cap — no MAX_SESSIONS bound on the spawn path (orchestrator fork-bomb safety)"
description: >
  No MAX_SESSIONS cap exists on any of the 3 local session-spawn entry points (session.rs:246 register, execution.rs:312 spawn, tools.rs:11574 MCP termlink_spawn). A buggy orchestrator loop exhausts host PIDs/FDs/mem (1 process + forked shell + PTY FD pair + 1MiB scrollback per session). NOT remote-peer reachable (register_remote only stores metadata; no hub-side PTY spawn) — orchestrator-safety, not adversarial defense. Turnkey: needs a per-host-vs-per-caller policy decision (caller identity absent at spawn site → only per-host expressible) + default value + whether-to-cap-at-all call given local-only severity.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T14:14:51Z
last_update: 2026-08-04T14:14:51Z
date_finished: null
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

# T-2532: per-host live-session cap — no MAX_SESSIONS bound on the spawn path (orchestrator fork-bomb safety)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

Filed by the T-2468 purpose-review campaign (session-spawn reliability lens). An
adversarial hunter + in-code verification confirmed **no `MAX_SESSIONS` cap exists**
on any session-spawn path. Sessions are decentralized OS processes (not a hub-held
map): each is 1 `termlink register` process + a `fork()`ed child shell + a PTY
master/slave FD pair + a 1 MiB scrollback ring + ~2 tokio tasks (`pty.rs:106/120`,
`session.rs:246`, `scrollback.rs:14`). `Drop for PtySession` (`pty.rs:456`) SIGKILLs
the child, so cleanly-dropped sessions reap — but a loop holds them all live.

**Threat scope (honest):** NOT remote-peer reachable. No hub RPC spawns a PTY —
`session.register_remote` (`router.rs:749`) only stores metadata of a session running
elsewhere. The only vector is a LOCAL buggy/hostile orchestrator calling MCP
`termlink_spawn` (`tools.rs:11574`) or CLI `spawn` (`execution.rs:312`) in an unbounded
loop, exhausting host PIDs/FDs/memory — on a host the agent already controls (and
`remote_exec` Execute-scope is strictly more powerful). So this is **orchestrator
fork-bomb safety, not adversarial-peer defense** — lower severity than a remote DoS.

## Decisions

### OPEN — policy decisions required before build (why this is a turnkey, not an autonomous fix)
- **Cap at all?** Given the DoS is local/self-inflicted (no remote session-spawn RPC),
  is a cap warranted, or is documenting the footgun + gating the MCP tool enough?
- **Per-host vs per-caller?** Caller identity (T-1427 fingerprint / PermissionScope) is
  **absent at the spawn site** (spawn is not an authenticated hub RPC), so only a
  **per-host** total-session cap is naturally expressible. Per-caller would need
  threading identity into the register path. Human call.
- **Default value?** e.g. `TERMLINK_MAX_SESSIONS` default 256 — needs an operator's
  sense of realistic fleet session counts. Do NOT guess-and-ship.
- **Enforcement point.** No single `sessions.insert()` exists — three independent
  entry points. A cap must count live session files (`termlink_session::discovery::
  sessions_dir()/*.json`) at each `register` entry (a racy filesystem-count gate) or a
  shared helper must be introduced. Structural decision.

## Acceptance Criteria

### Agent
<!-- Fill after the OPEN policy decisions above are resolved by a human. Draft turnkey: -->
- [ ] A shared `enforce_session_cap()` helper counts live sessions (`sessions_dir()/*.json`, filtering dead PIDs) and refuses over `TERMLINK_MAX_SESSIONS` (env, default DECIDED) with a loud `TOO_MANY_SESSIONS` error
- [ ] The cap is enforced at all three register entry points (`session.rs:246`, via `execution.rs:312 spawn`, `tools.rs:11574 termlink_spawn`)
- [ ] A test spawns up to the cap, asserts the (cap+1)th refuses loudly, and asserts a dropped/dead session frees a slot (dead-PID filtering works)
- [ ] `cargo build` + `cargo test -p termlink-session` clean; docs note the remote-peer vector does NOT exist (orchestrator-safety framing)

### Human
- [ ] [REVIEW] Resolve the four OPEN policy decisions in `## Decisions` (cap-at-all / per-host-vs-per-caller / default value / enforcement structure) before the Agent ACs are actioned
      **Steps:**
      Read the Context + Decisions sections; decide each of the four questions.
      **Expected:** Each OPEN decision replaced with a chosen value + rationale.
      **If not:** Leave `horizon: later`; this is not urgent (local-only, low severity).

### Human — legacy
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

## Recommendation

**Recommendation:** DEFER — do not ship a cap until someone measures the real
live-session distribution on this fleet. Set a revisit trigger rather than guessing
a default.

**Rationale:** The finding is confirmed: there is no bound of any kind. But every
one of the four open questions reduces to one number nobody has measured — how many
sessions a legitimate orchestrator actually runs. A cap set too low refuses real
work with a loud error on a path that currently always succeeds, which is a
*worse* failure than the unbounded case it prevents, because it fires on correct
usage. The unbounded case fires only on a bug that has not been observed here. That
asymmetry says measure first, and the task's own filing agrees ("Do NOT
guess-and-ship").

**Evidence (measured 2026-08-27, this repo):**
- `grep -rn "MAX_SESSIONS" crates/` → **zero hits**. So do
  `TOO_MANY_SESSIONS`, `max_sessions`, `session_cap`. The absence is total, not
  partial — there is no cap to tune, anywhere.
- Spawn/register entry points found: `session.rs:225 cmd_register`,
  `execution.rs:394 cmd_spawn`, MCP `tools.rs:12489 termlink_spawn` — plus a
  **fourth the task does not name**: `session.rs:499 cmd_register_self`, which its
  own comment describes as "parity with cmd_register" (duplicated, not delegating).
  It is event-only, so it costs a process and a session file but no PTY FD pair —
  a different cost profile, still uncapped. Any cap scoped to the three named
  entry points leaves this one open.
- Actual live-session counts on this or any fleet host: **not measured**. The
  runtime dir is outside this project's boundary and I did not sample it.
- Remote reachability of the spawn path: I did not re-verify the task's
  `register_remote`-only claim; treat it as the filing agent's finding, not mine.

**What you are actually deciding.** Not "is the gap real" — it is. You are choosing
between three postures:

| Option | Cost |
|---|---|
| **Defer with a trigger** (recommended) | The gap stays open. Cheap, because the exposure is local-only and self-inflicted; a host that fork-bombs itself is a host you already control. |
| Ship a per-host cap now with a guessed default | You will pick a number with no data. If it is low, legitimate orchestration starts failing loudly; if it is high enough to be safe, it stops being a meaningful bound. |
| Close as knowingly-accepted, document the footgun | Honest and cheap, but discards the finding — including the fourth entry point above, which nothing else records. |

**A cheaper middle path worth considering.** The four questions all assume the
remedy is a cap. Emitting the current live-session count as telemetry costs no
policy decision at all, cannot refuse anyone's work, and produces exactly the
measurement that unblocks every one of the four. If you want progress without a
policy call, that is the move.

**Why I should not decide this alone.** "Default value needs an operator's sense of
realistic fleet session counts" is the task's own framing and it is correct. A
number I invent becomes a refusal that fires on your workload, not mine. The
per-host-vs-per-caller question is likewise structural: caller identity is genuinely
absent at the spawn site, so choosing per-host is choosing to accept that a single
misbehaving caller consumes the whole budget.

**If you defer:** set `revisit_at` plus `revisit_evidence_needed:` naming the
measurement — e.g. "observed peak live-session count across the fleet over 30 days"
— so the G-053 scan can surface it when that number exists. A bare defer with no
trigger is how this returns unchanged in six months.

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

### 2026-08-04T14:14:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2532-per-host-live-session-cap--no-maxsession.md
- **Context:** Initial task creation
