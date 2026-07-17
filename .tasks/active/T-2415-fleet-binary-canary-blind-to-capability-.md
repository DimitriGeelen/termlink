---
id: T-2415
name: "Fleet-binary canary blind to capability-staleness on exempt hubs (.121 cannot serve cv_keys)"
description: >
  Add a lineage-independent CAPABILITY probe to the fleet-binary canary so a hub that cannot serve doorbell-prerequisite RPCs (cv_keys) FIRES, even when version-floor-exempt. .121 is doorbell-incapable and nothing fires.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-17T11:15:22Z
last_update: 2026-07-17T11:15:22Z
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

# T-2415: Fleet-binary canary blind to capability-staleness on exempt hubs (.121 cannot serve cv_keys)

## Context

**Found 2026-07-17 while completing the whole-fleet doorbell (T-2409).** `.121`
(ring20-dashboard) is structurally incapable of joining the fleet doorbell, and NOTHING
in the framework fires about it. The fleet-binary canary reports
`healthy — all floored reachable hubs at/above floor`.

**The evidence (reproducible, ~10s):**

```
termlink fleet doctor                        # .121 => [PASS] connected in 42ms (0.11.806)
termlink channel cv-keys agent-presence --hub 192.168.10.121:9100
  => Error: JSON-RPC error -32001: Missing 'target' in params      # <-- cannot serve cv_keys
termlink channel cv-keys agent-presence --hub 192.168.10.122:9100
  => topic=agent-presence count=2                                   # <-- healthy peer, same call
```

`.121` cannot serve `channel.cv_keys` (T-2103 cv_index). Without cv_index, every
agent-presence read walks the full backlog and times out (`rc=124` from
`agent-listeners-fleet`), so `.121` is invisible to `/peers`, `find-idle`, and the
`agent contact` reachability preflight. This is the SAME class T-2390/T-2391/T-2392
closed client-side (PL-250) — but here the defect is the HUB binary, not the reader.

**Why nothing fires — and why the existing instrument cannot help.**
`.context/cron/fleet-version-floors.conf` has `ring20-dashboard -` (exempt). The
exemption is CORRECT on its own terms: CLAUDE.md §fleet-binary canary warns that
`.121`'s `0.11.806` is a `git describe` tag-epoch artifact and that patch numbers are
NOT comparable across build lineages/tag epochs — T-2377 concluded `.121` is very likely
a ~1050-commit-STALE build of our OWN mainline, not a fork. So a version FLOOR is
structurally the wrong instrument: it cannot distinguish "newer" from "older lineage",
which is exactly why the hub was exempted, which is exactly why the framework went blind.

**The insight: probe CAPABILITY, not version.** "Does this hub answer `channel.cv_keys`?"
is lineage-independent, tag-epoch-independent, and decisive — it needs no floor, no
version comparison, and no knowledge of the build's provenance. It answers the question
the operator actually has ("can this hub carry the doorbell?") rather than a proxy
("is its integer bigger?"). Capability probing settles what version comparison provably
cannot.

**G-019 framing:** the symptom (`.121` unreachable) has been visible for weeks and read
as "infra/unreachable, informational" (PL-219). The real state is "hub is UP, auth PASSES
in 42ms, but the binary predates the doorbell's discovery prerequisite" — a different and
actionable condition that no canary expresses. Fixing `.121` itself needs a foothold +
binary upgrade (no remote-exec session exists there — operator reach). This task is the
DETECTION half only: make the framework SAY it.

## Acceptance Criteria

### Agent
- [ ] Canary probes a doorbell-prerequisite CAPABILITY (`channel.cv_keys`) per reachable hub, independent of version floors
- [ ] A hub that is version-floor-EXEMPT still FIRES when it cannot serve the capability (the `.121` case)
- [ ] A hub that serves the capability does NOT fire (the `.122` case — verify against both live hubs)
- [ ] Unreachable hubs stay informational, never firing (PL-219 convention preserved)
- [ ] Firing output names the failed capability + the remediation (binary upgrade on that host), not just "stale"
- [ ] Existing version-floor behaviour is unchanged (no regression in `tests/`-covered floor logic)
- [ ] Test hook feeds canned probe results so the check is verifiable hub-independently (PL-213 convention)

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

### 2026-07-17T11:15:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2415-fleet-binary-canary-blind-to-capability-.md
- **Context:** Initial task creation
