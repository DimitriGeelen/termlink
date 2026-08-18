---
id: T-2751
name: "install.sh artifact names drift from release.yml — 9th static check (herdr
  rank 20 real finding)"
description: >
  install.sh artifact names drift from release.yml — 9th static check (herdr rank
  20 real finding)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [scripts/check-release-artifact-drift.sh, 
      tests/release-artifact-drift-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T21:22:27Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-15T21:30:31Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:58Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2751: install.sh artifact names drift from release.yml — 9th static check (herdr rank 20 real finding)

## Context

Herdr backlog rank 20 proposed building a `curl | sh` user-level installer. **The premise is
false: `install.sh` already exists** (T-1134, repo root, 159 lines) — checksum-verifying,
multi-arch, no-sudo fallback with a `PREFIX=$HOME/.local` hint (install.sh:152) and a PATH
warning (install.sh:158). Every element of "herdr's shape" is already present; only the
default prefix differs, and that is a deliberate choice with a documented fallback. Rank 20
is closed as ALREADY-IMPLEMENTED, not built.

Investigating it surfaced a real and unguarded gap, which is what this task addresses.

**The artifact-name list exists as three hand-maintained copies:**
1. `install.sh:70-81` — case arms selecting which artifact to download
2. `.github/workflows/release.yml:229-235` — the assets actually published
3. `homebrew/Formula/termlink.rb` — download URLs for 4 of the 5

Nothing verifies they agree. This is precisely the shape T-2484 exists for (the charter
sentence as three copies with no transclusion), applied to release artifacts instead of prose.

**Why it matters more than an ordinary drift risk.** `install.sh` is the *first* option in
README Quick Start (README.md:91-92), described as "no toolchain required" — the path most
new users take. It has **zero** CI coverage. `.github/workflows/install-check.yml` guards
the *third* option (`cargo install --git`) and its own header calls that "the documented
installation path", which is now stale. So the guarded path is the least-used one, and the
unguarded one is piped into `sh` on a stranger's machine, where a rename surfaces as
`die "failed to download"` on their host rather than as a red build on ours.

Same class as T-2683 (static checks nothing ran) and T-2686 (`parity_topics` failing
undetected since 2026-08-12): the artifact exists, is believed to work, and nothing executes
or verifies it.

## Acceptance Criteria

### Agent
- [x] `scripts/check-release-artifact-drift.sh` exists, carries the `# guard-layer: source`
      marker, and joins the guard layer (appears in `run-guard-layer.sh --list`)
- [x] It parses the artifact set from `install.sh` case arms and from `release.yml`'s publish
      block, and FIRES (exit 1) on a mismatch in EITHER direction, naming which side is
      missing the name — install-offers-unpublished (user gets a 404 on their machine) and
      published-but-unreachable (a target built and shipped that the installer never selects)
      are distinct failures with distinct messages
- [x] The Homebrew formula is checked as a SUBSET, not an equality: every artifact the formula
      references must be published, but a published artifact the formula omits does NOT fire
      (the gnu variant is deliberately excluded per T-1135)
- [x] Exit codes follow the layer contract: 0 clean, 1 firing, 2 tooling error — and an
      unparseable/absent input is exit 2, never a clean bill (fail-closed)
- [x] `tests/release-artifact-drift-fixtures.sh` exists, is hermetic (fixture dirs via test
      seams, no network, no live release), and includes a PL-219 control asserting the check
      does NOT fire on the agreeing real tree
- [x] Load-bearing proof recorded in Decisions: a temporary rename on ONE side makes the check
      fire, restoring returns it to clean, and the tree is verified byte-identical afterwards
- [x] Current tree scans CLEAN (the three lists agree today) with the census stated in output
- [x] CLAUDE.md documents the check alongside the other eight static checks, stating its scope
      limit explicitly: it compares NAMES only, and does not verify that a published asset is
      downloadable or that the binary runs
- [x] Herdr backlog banner updated: rank 20 closed as ALREADY-IMPLEMENTED with the corrected
      premise recorded, so a future reader does not re-derive the false one

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

# The check scans the real tree clean (the three artifact lists agree today).
bash scripts/check-release-artifact-drift.sh

# Fixture suite passes (hermetic — no network, no live release).
bash tests/release-artifact-drift-fixtures.sh

# The check is a declared guard-layer member, not an orphan script (T-2683 class).
out=$(bash scripts/run-guard-layer.sh --list 2>&1); echo "$out" | grep -q "check-release-artifact-drift.sh"

# Whole guard layer still green with the new member included.
out=$(bash scripts/run-guard-layer.sh 2>&1); echo "$out" | grep -q "guard layer: PASS"

# JSON envelope parses and reports the census.
out=$(bash scripts/check-release-artifact-drift.sh --json 2>&1); echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert d['ok'] is True; assert d['published_count'] > 0"

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

### 2026-08-15 — rank 20 closed as ALREADY-IMPLEMENTED rather than built

- **Chose:** Do not build a `curl | sh` installer. Close rank 20 by correcting its premise,
  and build the guard the investigation actually surfaced.
- **Why:** `install.sh` has existed since T-1134 and already has every element the item
  describes — single binary, `uname` os/arch detection, checksum verification, writes no
  config, does not edit shell rc, and a no-sudo path hinting `PREFIX=$HOME/.local`
  (install.sh:152). The only delta is the default prefix (`/usr/local`), which is a
  deliberate choice with a documented fallback, not a missing capability. Building a second
  installer would have added a competing path to maintain and guarded nothing.
- **Rejected:** *Change the default prefix to `$HOME/.local`* — a real behaviour change to
  the published install path, affecting every existing documented invocation, to satisfy a
  backlog item's phrasing rather than an observed user problem. No evidence of anyone
  blocked by the current default was found; the sudo path and the hint both already work.
- **Rejected:** *Close rank 20 with no artifact* — defensible, but it would have left the
  finding (three unsynchronised copies, primary install path unguarded) undocumented and
  unguarded, and a future reader would re-derive the same false premise from the same entry.

### 2026-08-15 — bidirectional install↔release, but formula as subset only

- **Chose:** Check `install.sh` ↔ `release.yml` in **both** directions with distinct
  messages; check the Homebrew formula against `release.yml` in **one** direction only
  (every formula reference must be published; a published artifact the formula omits does
  not fire).
- **Why:** The two install↔release directions fail differently and only one is loud.
  Offered-but-unpublished gives the user a 404 on their own machine. Published-but-
  unreachable produces **no error anywhere** — we build and host a target the primary
  installer never selects — which is exactly the silent-wrong-answer shape Directive #2 is
  about, so collapsing them into one message would lose the more dangerous case. The
  formula's omission of the gnu `linux-x86_64` variant in favour of the musl static one is a
  decision already made (T-1135); checking that direction as equality would fire daily on it,
  which is how a guard teaches its operator to stop reading it (the T-2709 latch lesson).
- **Rejected:** *Three-way equality across all sources* — would false-fire on T-1135 forever.
- **Rejected:** *Parse any `termlink-*` token from the formula* — would ingest
  `Dir["termlink-*"].first`, a runtime glob, as if it were an artifact name. Fixture 8 pins
  this.

### 2026-08-15 — no allowlist, and an empty extraction is exit 2

- **Chose:** Ship without an allowlist, and treat a zero-length artifact set from either
  side as a tooling error rather than a clean census.
- **Why:** The eight sibling checks have allowlists because their firing sets contain sites
  that are genuinely correct but unprovable by grep. That is not true here: every mismatch is
  a real defect with a real one-line fix, so an allowlist could only ever be used to silence
  a genuine break. On the empty case — "0 names agree with 0 names" is vacuously true, so a
  parse that silently stopped matching would report green; this is the same failure T-2747
  refused when it declined to report a clean census over zero tools. Fixture 9 pins both
  directions.
- **Rejected:** *Warn-only on an empty set* — a warning in a check whose entire output
  contract is an exit code is a warning nobody reads.

### 2026-08-15 — load-bearing proof (demonstrated, then restored)

- **Chose:** Prove the check fires against the real tree, not only against fixtures.
- **Evidence:** Renaming one case arm in `install.sh`
  (`termlink-linux-aarch64` → `…-RENAMED`) fired the check **twice from a single edit** —
  `offered-but-unpublished` for the new name and `published-but-unreachable` for the
  orphaned one — each with its own message, and the formula (which still references a
  published name) correctly stayed silent. Restored from a pristine copy; `git diff --quiet
  install.sh` confirms byte-identical to HEAD, and the check returns to clean.
- **Why it matters:** fixtures alone can pass with a detector wired to fire unconditionally.
  Fixture 15 is the PL-219 control asserting the real tree scans clean; this revert is its
  complement, asserting the real tree *can* be made to fire.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-15T21:22:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2751-installsh-artifact-names-drift-from-rele.md
- **Context:** Initial task creation

### 2026-08-15T21:30:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
