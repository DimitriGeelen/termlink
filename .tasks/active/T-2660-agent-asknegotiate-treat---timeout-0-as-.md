---
id: T-2660
name: "agent ask/negotiate treat --timeout 0 as instant-fail while listen+wire treat 0 as forever"
description: >
  timeout-0 footgun divergence between ask/negotiate and listen

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T20:34:27Z
last_update: 2026-08-12T20:34:27Z
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

# T-2660: agent ask/negotiate treat --timeout 0 as instant-fail while listen+wire treat 0 as forever

## Context

Verified in code (round-12 footgun hunt, 2026-08-12). `cmd_agent_ask` (agent.rs)
encodes `timeout_secs: if timeout > 0 { Some(timeout) } else { None }` (agent.rs:71)
— i.e. `--timeout 0` tells the PEER the request is **open-ended** — but locally sets
`timeout_dur = Duration::from_secs(0)` (agent.rs:140) so the very first loop
iteration hits `remaining.is_zero()` (agent.rs:145-146) and **bails instantly** with
the timeout message. `cmd_agent_negotiate` has the identical shape (agent.rs:445 wire
encoding, 468 `timeout_dur`, 616 elapsed check). The sibling `agent listen`
documents + honors `0 = listen forever` (cli.rs:4519). So a user carrying the
documented `listen` "0 = forever" convention to `ask`/`negotiate` gets an instant,
surprising failure — while the peer believes the request is open-ended.

**NOT a mechanical fix — a design decision with 3 defensible resolutions (this is
why it is FILED, not autobuilt):**
1. **Make `0 = forever` locally** (loop indefinitely, matching the wire `None` it
   already sends + the listen convention). Aligns local with the already-shipped
   wire semantic. Risk: an unbounded CLI hang with no other exit.
2. **Reject `0` loudly** with an actionable error ("`--timeout 0` is not valid for
   `ask`; pass a positive value, or use `agent listen` for open-ended waiting").
   No hang, no wire-contract change — but still contradicts line 71's `None`.
3. **Fix the wire encoding** so `0` maps to a bounded value (or is validated at the
   clap layer), making local + wire consistent the other direction.
Note: `ask` defaults to **30** (not 0) and does NOT document `0 = forever`, so this
is genuinely ambiguous — a human/owner should pick the resolution.

## Acceptance Criteria

### Agent
- [ ] A resolution is chosen (1/2/3 above) — record it in `## Decisions` with rationale before touching code
- [ ] `cmd_agent_ask` and `cmd_agent_negotiate` handle `--timeout 0` consistently with the chosen resolution AND with each other AND with the wire encoding at agent.rs:71/445
- [ ] Behavior is unit-testable in-process (no hub) — e.g. a pure helper mapping `timeout: u64 -> Option<Duration>` (or a validation result) that the loop consumes; test both `0` and a positive value; prove load-bearing via temp-revert
- [ ] If resolution 1 (forever) is chosen, confirm there is a non-timeout exit path (Ctrl-C / peer response) so it cannot hang a script silently
- [ ] `cargo build -p termlink` clean

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

**Symptom:** `termlink agent ask <peer> <action> --timeout 0` fails instantly with a
timeout error, even though the peer is told the request is open-ended.

**Root cause:** asymmetric handling of `timeout == 0` — the wire encoding
(agent.rs:71) treats it as "no timeout / forever" (`None`), but the local poll loop
feeds it into `Duration::from_secs(0)` and bails on the first `remaining.is_zero()`.

**Why structurally allowed:** three sibling commands (`ask`, `negotiate`, `listen`)
each independently chose a `0` semantic; there is no shared timeout-parsing helper
enforcing one convention, and no test pins the `0` case. `listen` documents
`0 = forever`; `ask`/`negotiate` silently diverge.

**Prevention:** a shared `parse_agent_timeout(u64) -> Option<Duration>` (or a
validated enum) consumed by all three commands makes the `0` semantic single-sourced
+ unit-testable; add a test asserting the `0` mapping.

**Filed not built:** the resolution is a UX/product design decision (3 options with
different hang/consistency trade-offs) — must not be picked autonomously per the
autonomous-mode authority boundary.

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

### 2026-08-12T20:34:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2660-agent-asknegotiate-treat---timeout-0-as-.md
- **Context:** Initial task creation
