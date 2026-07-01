---
id: T-2301
name: "V6-S4 try-direct/fall-back orchestration"
description: >
  V6-S4 try-direct/fall-back orchestration

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:reliable-comms]
components: []
related_tasks: [T-2291, T-2296, T-2298, T-2299, T-2300]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-01T21:15:50Z
last_update: 2026-07-01T21:49:04Z
date_finished: 2026-07-01T21:49:04Z
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

# T-2301: V6-S4 try-direct/fall-back orchestration

## Context

arc-003 reliable-comms V6 (apex, T-2296) **slice S4** — the try-direct /
fall-back-to-hub orchestration. This is the slice the S1–S3 rails were built for.
Design: `docs/plans/T-2296-v6-direct-transport-first-design.md` **§2-S4** + §4
(decision logic) + §3 (ladder). Read that §S4 first — it is the detailed spec;
these ACs are the acceptance surface, not a re-spec.

**What ships:** `agent-send.sh` gains the actual transport BRANCH that S2 only
laid the seam for. `--transport auto` becomes the default and, per §4:

```
probe = remote ping <peer_addr>   (S2's bounded _probe_reachable)
if reachable → DIRECT:   post to peer's own hub; confirm via mechanism A
                          (the S3 sidecar stage=delivered receipt); doorbell
                          becomes OPTIONAL on this path (sidecar acks, no LLM turn)
else (auto)  → FALLBACK:  LOUD `agent-send: FALLBACK host <addr> unreachable →
                          hub store-and-forward` line, then the existing hub leg
                          (local store-and-forward + mechanism B frontier confirm
                          via the --await-ack / offline-queue path). NEVER silent.
--transport direct = direct only; FAIL loud if host unreachable (no fallback).
--transport hub    = force today's hub leg unchanged (escape hatch / back-compat).
```

**Prereqs already shipped:** S2 `_probe_reachable` + `--transport` flag + plan
computation (T-2299, agent-send.sh); S3 sidecar `stage=delivered` auto-receipt +
stage-aware DELIVERED poll (T-2300); S1 journal (T-2298). The offline queue
(`~/.termlink/outbound.sqlite`, T-2051) is the fallback store-and-forward
substrate. Mechanism B (`channel post --await-ack`) is the fallback confirm.

**Key code anchor:** the orchestration block to rewrite is
`agent-send.sh` §"2. Ring the doorbell + wait for a receipt" through the
DELIVERED/FAILED tail (roughly the `for (( ring... ))` loop down to `exit 3`) —
turn it into a transport-branch. The direct branch keeps the S3-style
mechanism-A poll; the fallback branch runs the hub leg. `hub_args` already targets
the right hub per T-2273; S4 adds the reachability GO/NO-GO + the loud fallback.

**Subtlety to resolve at build time (Evolution-worthy):** for a remote peer,
today's `hub` path already posts to `peer_hub`. The real direct-vs-fallback
difference is (a) the confirm SOURCE (A vs B) and (b) what happens when
`peer_hub` is UNREACHABLE — fallback must not just fail, it must enqueue to the
local offline queue for later flush. Verify how `channel post` + the offline
queue behave when the target hub is down (does the CLI auto-enqueue, or must
agent-send detect unreachable-and-enqueue?) before wiring the fallback leg.

**Scope boundary:** journal-authoritative + firehose suppression for dm: is
**S5** (the last slice). S4 does not move anything off the firehose.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `agent-send.sh` default transport flips to `auto`; `auto` runs the S2 reachability probe against the resolved peer addr and branches direct-vs-fallback per design §4. `--transport hub` still forces today's hub leg byte-for-byte (back-compat escape hatch); `--transport direct` does direct-only
- [x] DIRECT branch (peer reachable): posts to the peer's own hub and confirms via mechanism A (the S3 `stage=delivered` receipt); the doorbell-ring is optional on this path (a running sidecar acks without a woken interactive agent). DELIVERED line surfaces the stage (S3)
- [x] FALLBACK branch (`auto`, peer host unreachable): emits a LOUD `agent-send: FALLBACK host <addr> unreachable → hub store-and-forward` line (never a silent downgrade), then runs the hub leg — local store-and-forward (offline queue) + mechanism A/frontier confirm. `--transport direct` on an unreachable host FAILS loud instead of falling back. **(Confirm-source note: build resolved to mechanism A on the fallback leg too, not the design's mechanism B — see Evolution.)**
- [x] The single sender-API confirm contract is preserved (V3b): the caller still learns DELIVERED-or-FAILED loud; only the receipt SOURCE differs by transport (peer-hub sidecar on direct, local-hub+federation on fallback). Existing `agent-send.sh` A–G tests still pass (no observable-contract regression)
- [x] Tests prove both branches hub-independently (peer-free, loopback): loopback-up peer → DIRECT branch (DELIVERED via mechanism A); simulated-down peer (closed port / probe seam) → FALLBACK branch (loud FALLBACK line + DELIVERED via the hub leg); `--transport direct` on down host → loud FAIL; A–G + S1/S2/S3 test suites all still pass

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
# (S4 commands — the orchestration test file is created during the build.)
bash scripts/test-agent-send-orchestration.sh
bash scripts/test-agent-send.sh
bash scripts/test-agent-send-transport.sh
bash scripts/test-sidecar-auto-confirm.sh
bash -n scripts/agent-send.sh

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

### 2026-07-01 — fallback confirm is mechanism A, not the design's mechanism B
- **What changed:** The design (§S4, written before S3 shipped) specified the
  fallback leg confirm via **mechanism B** (`channel.receipts` frontier through
  `--await-ack`). By build time S3 (T-2300) had made the **journaled `stage=delivered`
  receipt envelope (mechanism A)** the durable store-and-forward confirm — and
  `agent-send.sh`'s whole confirm path already polls for a mechanism-A receipt.
  Wiring a parallel `--await-ack` invocation on the fallback leg would post the turn
  **twice** (once via the script's `channel post`, once inside `--await-ack`'s own
  post loop), breaking the script's load-bearing "post the turn exactly once up
  front" invariant. So the fallback reuses the SAME mechanism-A poll, retargeted to
  the local hub.
- **Plan impact:** AC3/AC4's "mechanism B on fallback" is superseded by "mechanism A
  on both transports; the receipt SOURCE differs (peer-hub sidecar on direct vs
  local-hub + DM federation on fallback)." AC4's "one confirm contract, receipt
  source differs by transport" holds — just with A/A, not A/B. This is exactly the
  spec-vs-build evolution the section exists to capture (T-1717).
- **Triggered:** No new sub-task. Documented in `docs/operations/agent-send-transport.md`
  §S4 and in the AC3 note above.

### 2026-07-01 — the real direct-vs-fallback difference is the POST TARGET, surfaced by the offline-queue topology
- **What changed:** Investigating "how does `channel post` + the offline queue
  behave when the target hub is down" (the task's flagged build-time subtlety)
  revealed the crux: **TCP cross-hub posts bypass the offline queue and hard-fail on
  an unreachable hub** (`crates/termlink-cli/src/commands/channel.rs:575` T-1385 —
  `BusClient` is Unix-only; only the local unix-socket path yields
  `PostOutcome::Queued`). So "hub store-and-forward" on fallback **cannot** mean
  "post to the down peer_hub" (that just dies). It must mean **clear `hub_args` and
  post to the LOCAL hub** — the genuine queue-backed store — and let DM federation
  carry it to the peer. The honest per-branch difference collapses to: *which hub
  the turn is posted to* (peer's vs local) + the LOUD fallback announcement + skip
  the (pointless) doorbell when the host is down.
- **Plan impact:** Confirms the design's "local store-and-forward" wording but pins
  down the mechanism (clear `hub_args`, not post-to-peer-then-queue). The `hub`
  escape hatch keeps the old hard-fail-at-post behavior for a down remote (proven by
  orchestration test O4, rc=2) — a useful contrast that makes the fallback value
  explicit.
- **Triggered:** No new sub-task. TOCTOU note: `auto` branches on the PROBE, so a
  peer that dies between probe and post (narrow ≤3s window) is a loud die on the
  direct leg rather than a fallback — acceptable for this slice; a probe-plus-post-
  failure fallback could be an S5-era hardening if it ever bites.

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

### 2026-07-01T21:15:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2301-v6-s4-try-directfall-back-orchestration.md
- **Context:** Initial task creation

### 2026-07-01T21:49:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
