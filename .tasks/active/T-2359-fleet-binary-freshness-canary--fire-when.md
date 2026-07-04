---
id: T-2359
name: "Fleet binary-freshness canary — fire when a hub serves below its declared version floor (G-069 prevention)"
description: >
  Fleet binary-freshness canary — fire when a hub serves below its declared version floor (G-069 prevention)

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
created: 2026-07-04T22:36:47Z
last_update: 2026-07-04T22:36:47Z
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

# T-2359: Fleet binary-freshness canary — fire when a hub serves below its declared version floor (G-069 prevention)

## Context

G-069: fleet hubs ran stale/deleted-exe binaries for weeks with nothing firing —
.122 served a pre-arc-004 feature set for ~13 days while the push-transport arc
was recorded closed=shipped, and .107 itself was 26 hub-side commits stale.
`fleet doctor` prints per-hub `hub_version` + a `fleet_versions` histogram but
nothing FIRES on it. Preflight Check 5 (T-2184) covers only the LOCAL hub.
This task builds the fleet-tier canary named in G-069's mitigation_candidate:
`scripts/check-fleet-binary-freshness.sh`, same skeleton as
`check-mirror-freshness.sh`, gated on a **declared per-hub version floor**
(config file), NOT on cross-hub skew — ring20-dashboard serves 0.11.806 from a
foreign build lineage, so patch-number comparison across hubs is structurally
unsound (both directions). Unreachable hubs are informational, not firing
(PL-219 expected-transient class; `fleet doctor`/`fleet status` already
surface down hubs). Bumping the floor when hub-side rails ship is the
operator's declaration that "shipped" must mean "capability-live" — the canary
then names lagging hubs daily until they restart.

## Acceptance Criteria

### Agent
- [x] `scripts/check-fleet-binary-freshness.sh` exists: reads `fleet doctor --json`, compares each reachable hub's `hub_version` against its declared floor from `.context/cron/fleet-version-floors.conf` (format: `<hub-name> <min-version>`, `*` default row optional, `-` = exempt/foreign-lineage), FIRES (exit 1) when any floored hub serves below its floor; exit 0 healthy, exit 2 tooling/hubs-unreadable
- [x] Version compare is numeric per segment (major.minor.patch), not lexicographic — `0.11.296 < 0.11.324` and `0.9.1591 < 0.11.2` both classify correctly
- [x] Unreachable/down hubs and hubs with no declared floor (incl. `-` exempt) are informational lines, never firing (PL-219); unknown `hub_version` on a reachable floored hub DOES fire (a hub too old to report its version is the staleness class itself)
- [x] Heartbeat file `.context/working/.fleet-binary-canary.heartbeat` touched every run before network calls (T-1723 meta-canary convention, `--no-heartbeat` opt-out); firing entries append to `.context/working/.fleet-binary-canary.log` framed `=== <ts> ===\n<output>\n---` (empty log = healthy, /canaries auto-discovers)
- [x] `--quiet` (cron mode: output only on firing) and `--json` (`{ok, firing[], hubs[]}` envelope) flags work; `TERMLINK_FLEET_FRESHNESS_TEST_JSON=<file>` feeds canned fleet-doctor JSON for hub-independent verification (PL-213 test-hook convention)
- [x] Test script `scripts/test-check-fleet-binary-freshness.sh` passes: covers below-floor firing, at/above-floor healthy, unreachable-hub non-firing, exempt-hub non-firing, unknown-version firing, numeric-compare edge (0.9.1591 vs 0.11.2)
- [x] Crontab `.context/cron/fleet-binary-canary.crontab` created AND installed to `/etc/cron.d/` (pre-push audit gate); floors file seeded with current expectations: local-test + workstation-107-public at 0.11.324 (T-2355 rails), ring20-management at 0.11.324 (fires until their restart picks up the walk-deadline fix — that firing is the G-069 signal working), ring20-dashboard `-` (foreign lineage), laptop-141 `-` (expected-transient, PL-219)
- [x] Live run against the real fleet exits 1 naming ring20-management (0.11.296 < 0.11.324) and no other firing hub
- [x] CLAUDE.md canary section documents the new canary (empty-log convention, floors file semantics, operator action on firing); G-069 gains a mitigation_progress entry

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

test -x scripts/check-fleet-binary-freshness.sh
bash scripts/test-check-fleet-binary-freshness.sh > /tmp/.t2359-test.out 2>&1 && grep -q "ALL PASS" /tmp/.t2359-test.out
test -f .context/cron/fleet-binary-canary.crontab
test -f /etc/cron.d/termlink-fleet-binary-canary
grep -q "fleet-binary-canary" /opt/termlink/CLAUDE.md
out=$(python3 -c "import yaml; d=yaml.safe_load(open('.context/project/concerns.yaml')); c=[x for x in d['concerns'] if x['id']=='G-069'][0]; print(c.get('mitigation_progress',''))"); echo "$out" | grep -q "T-2359"

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

### 2026-07-05 — firing gate: declared per-hub floor vs cross-hub skew
- **Chose:** declared per-hub version floors in a canary-owned config file (`.context/cron/fleet-version-floors.conf`), with `-` exemption for foreign lineages
- **Why:** patch numbers are commits-since-tag and are NOT comparable across build lineages — ring20-dashboard serves 0.11.806 from its own fork, numerically "newest" while lacking our commits; skew detection would false-positive on it forever AND miss same-lineage lag if the stale hub happened to be numerically close. A floor is also the semantic G-069 wants: bumping it when rails ship is the operator declaring "shipped must mean capability-live".
- **Rejected:** (a) cross-hub skew threshold (unsound across lineages, both directions); (b) compare-to-local-VERSION for all hubs (fires permanently on hubs we don't deploy to); (c) hubs.toml extra keys for floors (risk of tripping the binary's TOML parser — canary-owned file is decoupled)

### 2026-07-05 — unknown hub_version on a floored reachable hub fires
- **Chose:** treat missing `hub_version` as firing, not informational
- **Why:** a hub too old to report its version IS the staleness class the canary exists for; classifying it informational would let the worst offenders hide
- **Rejected:** informational (hides pre-version-field binaries); exit-2 tooling error (it is a data finding, not a tooling failure)

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

### 2026-07-04T22:36:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2359-fleet-binary-freshness-canary--fire-when.md
- **Context:** Initial task creation

### 2026-07-05T00:45:00Z — shipped [agent]
- **Action:** Canary + floors config + 14-case test suite (ALL PASS) + crontab installed to /etc/cron.d/termlink-fleet-binary-canary (06:03 UTC + meta-canary line) + CLAUDE.md eighth-canary section + G-069 mitigation_progress
- **Live proof:** ad-hoc run exits 1 naming exactly ring20-management (0.11.296 < floor 0.11.324 — the T-2355 walk-deadline rails .122 still lacks); /canaries auto-discovers and classifies FIRING with the signal line surfaced
- **Context:** G-069 what_remains (1) closed; (2) arc-closure capability-live check + (3) deploy restart-epilogue remain open under the concern
