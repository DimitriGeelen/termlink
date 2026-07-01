---
id: T-2299
name: "V6-S2 transport-select seam + reachability probe"
description: >
  V6-S2 transport-select seam + reachability probe

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:reliable-comms]
components: []
related_tasks: [T-2291, T-2296, T-2298]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-01T20:16:59Z
last_update: 2026-07-01T20:20:10Z
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

# T-2299: V6-S2 transport-select seam + reachability probe

## Context

arc-003 reliable-comms V6 (apex, T-2296) **slice S2** — the transport-select SEAM.
Design: `docs/plans/T-2296-v6-direct-transport-first-design.md` §S2. Builds on S1
(T-2298, journal, done). Adds a `--transport auto|direct|hub` flag + a reachability
probe to `scripts/agent-send.sh` (the existing routing brain) and surfaces the chosen
plan in the `--dry-run RESOLVED` line. **Scope is the SEAM + PLAN + probe only** —
the live try-direct/fall-back ORCHESTRATION is S4, and the direct-path confirm-source
change is S3. Default `--transport hub` preserves today's behavior exactly (zero
behavior change unless the flag is passed). Peer-free: testable via the existing
`LISTENERS_VERB` dry-run fixture seam (agent-send.sh:116-117) + loopback probe
(127.0.0.1:9100 up vs a closed port down).

Key code anchors: arg-parse agent-send.sh:72-90; fleet-resolve gives `peer_hub`
agent-send.sh:104-166; dry-run RESOLVED line agent-send.sh:205-207 (approx — verify
current line numbers, S1 may have shifted them). Probe primitive = `termlink remote
ping <addr>` (`cmd_remote_ping`, remote.rs:1071), bounded. Addr source = `peer_hub`
(T-2293 self-report); T-2297 hub-stamped `observed_addr` is a later hardening
follow-up (design §5 Q1) — S2 ships on self-report for the flat /24.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/agent-send.sh` accepts `--transport auto|direct|hub` (default `hub`); an invalid value exits 2 with a clear message; default `hub` reproduces today's behavior byte-for-byte (no send-path change) — flag parsed (agent-send.sh case `--transport`), validated up front (`must be auto|direct|hub`, T1 exit 2), `hub` prints no stderr plan line (T7 byte-for-byte)
- [x] A bounded reachability probe helper (wrapping `termlink remote ping <addr>`) computes `reachable=yes|no` for the resolved peer addr, with a test seam (e.g. `REMOTE_PING_VERB` / `TERMLINK_PROBE_TEST_*`) so both branches are assertable without a second host — `_probe_reachable` wraps `termlink remote ping` under `timeout` (`TERMLINK_PROBE_TIMEOUT`, default 5s); seams `REMOTE_PING_VERB` + real loopback; T3 yes (127.0.0.1:9100), T4 no (127.0.0.1:1)
- [x] `--dry-run` `RESOLVED` line is extended with `transport=<mode> direct_addr=<addr|local> reachable=<yes|no|skip>` reflecting the computed plan (probe only runs for `direct`/`auto`; `hub` prints `reachable=skip`) — T2 (hub→skip), T3 (direct→yes), T4 (auto→no), T5 (local degenerate→direct_addr=local reachable=skip)
- [x] The live (non-dry-run) send path is UNCHANGED in S2 — the flag records intent + prints the chosen-plan line to stderr for observability; the actual direct-vs-fallback branch is explicitly deferred to S4 (documented in `--help` and the ops doc) — T6 (live `direct`: POSTED on stdout unchanged + `transport-plan` line on stderr); hub_args routing untouched; documented in `usage()` + `docs/operations/agent-send-transport.md`
- [x] Tests prove: flag validation (bad value → exit 2), `hub`/`direct`/`auto` dry-run RESOLVED lines from canned presence (fixture seam), and probe reachable-vs-unreachable via loopback (127.0.0.1:9100 up, closed port down); existing agent-send tests A–G still pass (no regression) — `scripts/test-agent-send-transport.sh` 7/7 PASS; `scripts/test-agent-send.sh` A–G ALL PASS; `bash -n` clean

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
# (S2 commands — the transport-seam test file is created during the build.)
bash scripts/test-agent-send-transport.sh
bash scripts/test-agent-send.sh
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

### 2026-07-01 — S2 built as a clean seam; probe uses real loopback, not a mock

- **What changed:** The design (§S2) assumed a `TERMLINK_PROBE_TEST_*` force-seam
  might be needed to test both probe branches without a second host. It wasn't:
  `termlink remote ping 127.0.0.1:9100` succeeds against the live local hub and
  `127.0.0.1:1` (closed) fails, so real loopback exercises `reachable=yes|no`
  end-to-end. Kept `REMOTE_PING_VERB` as the command-override seam anyway (cheap,
  and useful for a future hermetic CI with no hub). The probe is bounded by
  `timeout ${TERMLINK_PROBE_TIMEOUT:-5}` so a wedged peer hub can't hang a send.
- **Plan impact:** The "local degenerate" case (peer on our own hub → `peer_hub`
  empty) wasn't called out in the design but falls out cleanly: `direct_addr=local`,
  `reachable=skip` — direct and hub coincide, nothing remote to probe. Encoded
  as a first-class row (T5), not an edge case.
- **AC1/AC4 tension resolved:** "records intent + prints a plan line" (AC4) vs
  "default hub byte-for-byte" (AC1) — resolved by emitting the stderr plan line
  ONLY for a non-default transport. Default `hub` path adds zero output and zero
  network calls; the interesting (`direct`/`auto`) path is the one that gets the
  observability line. T7 asserts the byte-for-byte silence.
- **Triggered:** No new sub-tasks. S3 (sidecar journaled-receipt), S4
  (try-direct/fall-back orchestration — the branch this seam enables), S5
  (firehose suppression) remain as planned; T-2297 (hub-attested source addr)
  still the hardening follow-up for the probe target.

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

### 2026-07-01T20:16:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2299-v6-s2-transport-select-seam--reachabilit.md
- **Context:** Initial task creation
