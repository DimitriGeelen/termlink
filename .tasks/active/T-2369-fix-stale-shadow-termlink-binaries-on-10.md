---
id: T-2369
name: "Fix stale shadow termlink binaries on .107 + agent-presence retention-reset (clears substrate-preflight/unconfirmed-delivery/topic-growth canaries)"
description: >
  Fix stale shadow termlink binaries on .107 + agent-presence retention-reset (clears substrate-preflight/unconfirmed-delivery/topic-growth canaries)

status: started-work
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
created: 2026-07-05T21:46:26Z
last_update: 2026-07-05T21:46:26Z
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

# T-2369: Fix stale shadow termlink binaries on .107 + agent-presence retention-reset (clears substrate-preflight/unconfirmed-delivery/topic-growth canaries)

## Context

Three PATH-shadowed `termlink` binaries exist on .107: `/root/.cargo/bin`
(0.11.324, current), `/root/.local/bin` (0.9.33, stale), `/usr/local/bin`
(0.9.1542, stale). Interactive shells resolve `.cargo/bin` first (fine), but
cron's minimal PATH resolves `/usr/local/bin` first → the stale 0.9.x binary.
This single root cause fires TWO daily canaries: substrate-preflight (WARNs on
the 0.9.1542 shadow) and unconfirmed-delivery (0.9.x has no `channel
awaiting-ack` verb → exit 2 → FALSE firing; the current binary reads the
tracker fine, pending=0). Separately, agent-presence on the local hub has
grown to ~30.8k records with retention=forever, firing the topic-growth canary
(documented reset: `set-retention latest-per-cv-key` + `sweep`). All three are
agent-actionable on this host — not operator-gated.

## Acceptance Criteria

### Agent
- [x] No 0.9.x `termlink` shadow remains on PATH (`which -a termlink` + `--version` on each shows only 0.11.x). Stale copies backed up as `*.bak`. (Residual substrate-preflight WARN of "binary < project VERSION" is expected dev-tree version-lag, not the shadow bug — the installed 0.11.x understands all current verbs.)
- [x] Current-PATH `termlink channel awaiting-ack --json` returns a readable tracker (no exit=2); unconfirmed-delivery canary healthy on re-run under cron PATH
- [x] agent-presence on the local hub is bounded (count=4, retention=latest_per_cv_key); topic-growth canary healthy on re-run under cron PATH — the prior 30804/forever reading was the stale 0.9.x binary misreading the topic
- [x] substrate-preflight binary check no longer resolves a 0.9.x binary (now 0.11.x; only benign version-lag WARN remains)

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
# No 0.9.x binary remains on PATH:
! which -a termlink | while read b; do "$b" --version; done | grep -q '0\.9\.'
# awaiting-ack tracker is readable via current-PATH binary (no exit=2):
termlink channel awaiting-ack --json > /dev/null

## RCA

**Symptom:** substrate-preflight canary WARNs "termlink 0.9.1542 older than
project VERSION"; unconfirmed-delivery canary fires 7× "could not read
awaiting-ack tracker (exit=2)". Both fired daily despite the local hub running
a current 0.11.x binary and the awaiting-ack tracker being healthy (pending=0).

**Root cause:** Multiple `termlink` binaries on PATH. Cargo installs to
`~/.cargo/bin` (kept current, 0.11.324) but two older installs persist at
`/usr/local/bin` (0.9.1542) and `/root/.local/bin` (0.9.33). Cron jobs run with
a minimal PATH (`/usr/local/bin:/usr/bin:/bin`) that lacks `~/.cargo/bin`, so
they resolve the stale `/usr/local/bin/termlink`. The 0.9.x binary predates the
`channel awaiting-ack` verb (arc-003 RC3b, T-2287) → exits 2 → unconfirmed
canary false-fires.

**Why structurally allowed:** Binary upgrades installed the new artifact to
`~/.cargo/bin` without removing/updating prior-location shadows; nothing
reconciled the multiple install locations. The substrate-preflight binary check
(T-2181) surfaces the WARN but its remediation (`install to ~/.cargo/bin`) fixes
the *already-current* copy, not the *stale shadow* the cron actually resolves —
so following the hint verbatim would not clear the canary.

**Prevention:** Upgraded all three PATH locations to the same current artifact
so cron resolves current regardless of PATH order (stale copies backed up as
`*.bak`). Longer-term catch is already partly present: substrate-preflight
Check 4 flags the stale binary; the gap is that it should check *every* PATH
entry, not just the first-resolved one. Logged as a follow-up note; the
resolved-binary-vs-shadow distinction is captured in the
[[reference_stale_shadow_binaries]] memory.

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

### 2026-07-05T21:46:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2369-fix-stale-shadow-termlink-binaries-on-10.md
- **Context:** Initial task creation
