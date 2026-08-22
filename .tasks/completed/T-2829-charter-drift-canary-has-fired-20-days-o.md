---
id: T-2829
name: "Charter-drift canary has fired 20 days of false positives — it measures whatever termlink cron PATH finds, a Jul-31 binary"
description: >
  Charter-drift canary has fired 20 days of false positives — it measures whatever termlink cron PATH finds, a Jul-31 binary

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-22T11:33:44Z
last_update: 2026-08-22T11:38:08Z
date_finished: 2026-08-22T11:38:08Z
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

# T-2829: Charter-drift canary has fired 20 days of false positives — it measures whatever termlink cron PATH finds, a Jul-31 binary

## Context

The charter-drift canary (T-2483) appended a **40-tool FIRING entry every day from
2026-08-02 to 2026-08-22** — 20 consecutive days. CLAUDE.md meanwhile records the surface as
"214 live tools scanned, 0 off-charter". Both were reporting honestly about different
binaries.

`TERMLINK="${TERMLINK_BIN:-termlink}"` resolves bare via PATH. The crontab sets
`PATH=/usr/local/bin:/usr/bin:/bin`, which finds `/usr/local/bin/termlink` — a **Jul-31
build, 0.11.693**, predating the P4 deprecations, reporting all 40 social-analytics tools
`deprecated==false`. A developer's PATH finds `~/.cargo/bin/termlink` (0.11.1196), which
reports every one of them `deprecated==true`. So the check passed by hand and fired on cron,
daily, and nothing in its output said which binary had answered.

This cost more than noise: earlier this session I relayed the canary's "40 live tools drift
from the charter" to the operator as a headline finding and flagged it as the one to read
twice. It was false. The surface is clean.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Revised mid-task — a version floor was the wrong instrument and was rejected.** Both
      output paths now name the resolved binary path and version, so a firing entry is
      diagnosable on sight instead of ambiguous between drift and staleness
- [x] The rejection is reasoned in the script: deprecation here is monotonic (6 deprecated at
      0.11.693, 46 at 0.11.1196, none un-deprecated), so the 40 tools separating the builds
      ARE the firing set — the staleness signal and the drift verdict are the same names and
      no threshold over them separates the two. Compounding it, VERSION is stamped per-commit
      while the binary always trails it, so `>= VERSION` would refuse on every healthy run
- [x] Provenance is reported on BOTH verdicts, not just firing: a clean run from the wrong
      binary is the symmetric hazard (it would call a drifted surface healthy)
- [x] The crontab pins `TERMLINK_BIN` (or PATH) to the binary the project actually ships, so
      the daily run stops resolving to `/usr/local/bin/termlink`
- [x] Fixtures cover both verdicts carrying provenance, both JSON envelopes carrying
      `probe_binary`/`probe_version`, and the test hook working with no binary on PATH
- [x] Mutation-tested: removing the firing-path provenance line takes the suite 8/8 → 7/8,
      and restoring it returns 8/8 — the line is what carries the behaviour

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

# Provenance fixtures pass (8 assertions, both verdicts + both JSON envelopes + PL-213 hook).
bash tests/charter-drift-provenance-fixtures.sh
# The check itself is clean against the current binary.
bash scripts/check-charter-drift-freshness.sh --no-heartbeat
# Firing output names the binary — reproduce with the stale one still on this host.
out=$(TERMLINK_BIN=/usr/local/bin/termlink bash scripts/check-charter-drift-freshness.sh --no-heartbeat 2>&1 || true); echo "$out" | grep -q 'probed: /usr/local/bin/termlink'
# The crontab pins the binary so cron stops resolving to /usr/local/bin.
out=$(cat .context/cron/charter-drift-canary.crontab 2>&1); echo "$out" | grep -q '^TERMLINK_BIN='

## RCA

**Symptom:** The charter-drift canary appended a 40-tool FIRING entry daily for 20
consecutive days (2026-08-02 → 2026-08-22) while the tool surface was already clean. Run by
hand it reported healthy. Nobody could tell which was right from the output.

**Root cause:** The check resolves its binary as `${TERMLINK_BIN:-termlink}` — bare, via
PATH. Cron's `PATH=/usr/local/bin:/usr/bin:/bin` finds a Jul-31 build (0.11.693) that
predates the P4 deprecations and therefore reports all 40 social-analytics tools as live. A
developer's PATH finds `~/.cargo/bin/termlink` (0.11.1196), where all 40 are deprecated.

**Why structurally allowed:** The canary's output never named the binary it read. Its verdict
depends entirely on which build answers, and that dependency was invisible — so a firing
entry was ambiguous between "the surface drifted" and "you asked the wrong binary", with no
way to distinguish them short of manually diffing two catalogs. Twenty days of daily alarms
went unresolved because each one looked exactly like a real finding.

This is the same shape as the other defects found this session: **the check asserts a
property adjacent to the one it claims.** It claims "the live tool surface has drifted from
the charter" and actually measures "whichever binary happens to be first on this PATH has
these tools marked live."

**Prevention:** Both output paths, human and JSON, now carry the resolved binary path and
version. A firing entry is diagnosable on its third line. The crontab pins `TERMLINK_BIN` so
the daily run reads the binary the project builds.

**A version floor was tried first and rejected — worth recording, because it looked right.**
The obvious fix is "refuse if the binary is older than VERSION", mirroring preflight Check 4.
It fails twice. Deprecation here is monotonic — diffing the catalogs gives 6 deprecated at
0.11.693, 46 at 0.11.1196, and *nothing* un-deprecated — so the 40 tools separating the
builds are exactly the firing set: the staleness discriminator and the drift verdict are the
same 40 names, and no threshold over them separates a stale binary from a genuine
re-accretion. That is T-2415's lesson restated (a version floor is blind to a capability
question). Second, `VERSION` is stamped per-commit (0.11.1283) while the installed binary
always trails it (0.11.1196), so `>= VERSION` would refuse on every healthy run — a gate that
fires constantly is one that gets removed.

**Not fixed here:** the same bare-PATH resolution exists in sibling canaries that shell out to
`termlink`. This task pins and instruments the one with 20 days of evidence; a sweep of the
others is a separate piece of work.

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

### 2026-08-22T11:33:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2829-charter-drift-canary-has-fired-20-days-o.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-586d01bd
- **Timestamp:** 2026-08-22T11:38:09Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-22T11:38:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
