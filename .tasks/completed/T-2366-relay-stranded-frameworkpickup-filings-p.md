---
id: T-2366
name: "Relay stranded framework:pickup filings P-050/P-051 to AEF inbox"
description: >
  Drop the two card-redirect-couriered filings (P-050 fw-update dispatch order, P-051 unsolicited bd-init/help-init) that fell through the framework:pickup->AEF bridge (offsets 75/76, posted 2026-06-27, bridge dormant until 2026-07-05 resumed past them) directly into AEF .context/pickup/inbox/ via termlink_run. G-067 (offset 74) SKIPPED: already fixed upstream by AEF T-2297/T-2298.

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
created: 2026-07-05T19:46:02Z
last_update: 2026-07-05T20:01:54Z
date_finished: 2026-07-05T20:01:54Z
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

# T-2366: Relay stranded framework:pickup filings P-050/P-051 to AEF inbox

## Context

Two card-redirect-couriered bug-reports (`P-050` fw-update dispatch-order,
`P-051` unsolicited `bd init` / `--help` auto-init) were posted to the
`framework:pickup` bus topic (offsets 75/76, 2026-06-27) with pickup_ids
`TL-COURIER-cardredirect-P050` / `...-P051`. The framework:pickup→AEF inbox
bridge was dormant 2026-06-10→2026-07-05 and, on resuming, consumed only the
newer offsets 77/78 (P-048/P-049) — leaving 75/76 stranded (absent from AEF's
`.context/pickup/{inbox,processed}` + `dedup.log`). This task delivers them
directly into AEF's `.context/pickup/inbox/` via `termlink_run` (T-559 blocks
Bash on AEF paths; MCP run is the sanctioned channel).

**G-067 (offset 74) deliberately SKIPPED** — already fixed in AEF master by
T-2297 (`06041f9bc`, "6.7min→132s") + T-2298 (`503058289`, "132s→65s");
`agents/audit/audit.sh` carries the batched block. Relaying a resolved bug
would create a duplicate task. This is the same fix I re-vendored in T-2365.

## Acceptance Criteria

### Agent
- [x] P-050 + P-051 envelopes decoded from the bus payloads, YAML-valid, `pickup_id`/`type` preserved verbatim (attribution to card-redirect intact)
- [x] Both envelopes written into AEF `/opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/` via `termlink_run`
- [x] Delivery verified: AEF per-minute `fw pickup process` cron drained both to `processed/` within ~10s (dedup.log 2026-07-05T19:55:01/02Z; inbox emptied) — confirmed by direct `ls`/`grep`
- [x] G-067 relay skip documented with upstream-fix evidence (T-2297/T-2298 commit shas) — no G-067 envelope written

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

# Envelopes are YAML-valid with attribution preserved
python3 -c "import yaml; d=yaml.safe_load(open('/tmp/claude-0/-opt-termlink/d013faa8-8a10-4419-a98d-ff5c8648174f/scratchpad/P-050-cardredirect-bug-report.yaml')); assert d['pickup_id']=='TL-COURIER-cardredirect-P050' and 'card-redirect' in d['source']['project']"
python3 -c "import yaml; d=yaml.safe_load(open('/tmp/claude-0/-opt-termlink/d013faa8-8a10-4419-a98d-ff5c8648174f/scratchpad/P-051-cardredirect-bug-report.yaml')); assert d['pickup_id']=='TL-COURIER-cardredirect-P051' and 'card-redirect' in d['source']['project']"

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

### 2026-07-05T19:46:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2366-relay-stranded-frameworkpickup-filings-p.md
- **Context:** Initial task creation

### 2026-07-05T19:49:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-05T19:55Z — relay delivered
- **Action:** Dropped P-050 + P-051 card-redirect envelopes (decoded verbatim from
  framework:pickup offsets 75/76) into AEF `.context/pickup/inbox/` via `termlink_run`.
- **Delivery:** AEF's own per-minute drain cron
  (`/etc/cron.d/agentic-audit-999-...`, `PROJECT_ROOT=AEF fw pickup process`, flock-guarded)
  consumed both within ~10s → `processed/P-050-cardredirect-bug-report.yaml` +
  `processed/P-051-cardredirect-bug-report.yaml`; `dedup.log` 2026-07-05T19:55:01/02Z;
  inbox emptied. Re-mirrored to the framework:pickup topic (idempotent by SHA).
- **Mechanism note:** confirms the inbox-drop path is the reliable relay channel —
  the *topic* consumer is cursor-based and skipped offsets 74/75/76 when it resumed
  past them (dormant 2026-06-10→2026-07-05), but the inbox drain re-injects them.
- **G-067 SKIPPED (offset 74):** already fixed in AEF master — T-2297 (`06041f9bc`,
  "6.7min→132s") + T-2298 (`503058289`, "132s→65s"); audit.sh carries the batched
  block. This is the same fix re-vendored into termlink under T-2365. Relaying a
  resolved bug would create a duplicate task — deliberately not written.

### 2026-07-05T19:57Z — relay made ACTIONABLE + AEF cron bug found
- **Finding (AEF bug):** the AEF pickup cron PROCESSED both envelopes but logged
  `WARN: fw not on PATH — cannot create task` for each — so files reached
  `processed/` + re-mirrored to the topic but NO actionable AEF task was created.
  Root cause: `lib/pickup.sh::pickup_create_inception` gates task creation on
  `command -v fw`; the AEF pickup cron runs with a minimal PATH that excludes
  `/opt/999-Agentic-Engineering-Framework/bin`, so bare `fw` is unresolved. This
  is a genuine AEF framework bug and a likely deeper root cause of G-063
  ("zero receipts / write-only sink") — cron-driven pickups silently fail to
  create tasks even after the envelope lands. Interactive/manual runs (fw on
  PATH) succeed, which is why prior hand-processed relays worked.
- **Completion:** replicated `pickup.sh`'s exact task-create call
  (`fw task create --type build --owner agent --horizon next --tags pickup,bug-report`,
  name `Pickup: <summary> (from card-redirect)`) via `termlink_run` with fw on PATH
  + `env -u CLAUDECODE`. No G-047 frontmatter injection needed (envelopes have empty
  `task_id`). Created **AEF T-100247** (P-050 dispatch-order) + **AEF T-100248**
  (P-051 bd-init/help-init), both `status: captured, type: build, owner: agent,
  horizon: next, tags [pickup, bug-report]` — identical to what the pipeline intended.
- **Follow-up (for user):** the `fw not on PATH` pickup cron bug warrants its own
  AEF fix (use resolved `$FW_BIN` not bare `fw` in `pickup_create_inception`) —
  surfaced to the user, not auto-filed (same bug would swallow the filing).

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1ee2c21b
- **Timestamp:** 2026-07-05T20:01:55Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-05T20:01:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
