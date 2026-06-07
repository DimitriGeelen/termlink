---
id: T-2015
name: "fw upgrade CLAUDE.md clobber — propagate template-merge hardening upstream (PL-124 / G-055)"
description: >
  fw upgrade step 1/10 (CLAUDE.md governance section refresh) replaces project-specific inline customizations with template content every run. Fired on /opt/termlink today losing 18 lines including the .agentic-framework/bin/fw path rule, /be-reachable opt-in, and the entire conversation-arc skill table. PL-124 documented this for >7 days; G-055 codifies the gap. Upstream fix needed: template-merge that preserves project-specific blocks, or operator-acceptable opt-out per CLAUDE.md section.

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
created: 2026-06-06T11:14:02Z
last_update: 2026-06-06T16:38:33Z
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

# T-2015: fw upgrade CLAUDE.md clobber — propagate template-merge hardening upstream (PL-124 / G-055)

## Context

Observed live on 2026-06-06 at root@.107 in /opt/termlink during a `fw upgrade --force-downgrade` run (steps 1-4 of 10 completed before separate refusal at 4c). Step 1 ("CLAUDE.md governance sections") wrote a fresh template-derived CLAUDE.md, backed up the prior content to `CLAUDE.md.bak`, and printed a warning:

```
17 line(s) in CLAUDE.md.bak are absent from the new CLAUDE.md.
These may be project-specific inline customizations the template merge cannot preserve.
```

The framework warning is correct but the behavior is silently destructive. Restoring required `cp CLAUDE.md.bak CLAUDE.md`; otherwise 18 lines of TermLink-specific operating guidance would have been lost permanently after the next session's audit prune. PL-124 was registered against T-1447 documenting this exact pattern. G-055 codifies it as a structural gap.

This task is the TermLink-side tracker. The fix lands upstream in `agentic-engineering-framework`. Sibling to T-2014 (which propagated the fork-bomb fix upstream and now lives as T-2099).

## Acceptance Criteria

### Agent
- [x] RCA captured in `## RCA` block below
- [x] Framework-agent prompt artifact written to `docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md` for operator copy-paste
- [ ] After upstream fix lands in vendored `.agentic-framework/lib/upgrade.sh`, re-run `fw upgrade` and confirm CLAUDE.md is left intact (or only the framework-template block within it is updated, leaving project-specific blocks unchanged)

### Human
- [ ] [REVIEW] Framework-agent prompt at `docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md` is operator-ready
  **Steps:**
  1. Open the file
  2. Read top-to-bottom as if you knew nothing about PL-124 or G-055
  3. Verify it contains: symptom, repro, file:line root cause, recommended fix shape (selective merge vs sentinel-section markers vs opt-out)
  **Expected:** Self-contained prompt — no follow-up clarifying questions needed from framework-agent
  **If not:** Note what's missing and revise the artifact, not the upstream fix

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

test -f docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md
grep -q 'PL-124\|G-055' docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md
grep -q 'Root cause' .tasks/active/T-2015-fw-upgrade-claudemd-clobber--propagate-t.md

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

**Symptom:** Every `fw upgrade` run (step 1/10 — "CLAUDE.md governance sections") writes a fresh template-derived CLAUDE.md, replacing what was previously committed. The framework backs up the prior content to `CLAUDE.md.bak` and prints a warning listing N lost lines, but takes no further action. If the operator/agent does not manually restore those lines, they vanish from version control after the next commit prune. Today's run lost 18 lines including the `.agentic-framework/bin/fw` vendored-path rule, the `/be-reachable` opt-in, and the entire conversation-arc skill table (`/agent-handoff`, `/reply`, `/check-arc`, `/check-outbox`, `/recent-dm`, `/peers`, `/recent-chat`, `/broadcast-chat`, `/pulse`, `/conversations` — 10 rows of operator-facing entry points).

**Root cause:** The template-merge logic for CLAUDE.md's governance section in `lib/upgrade.sh` step 1 is whole-section replacement, not block-aware merge. The framework template has no concept of "project-extends-here" boundaries within the governance section, and the consumer's CLAUDE.md has no opt-out mechanism per section. Result: the only way to retain project-specific governance is to NEVER let `fw upgrade` run that step, which conflicts with the framework's goal of getting consumers onto the latest governance rules.

**Why structurally allowed:** PL-124 was registered against T-1447. PL-022 (T-1069) documented the broader "fw upgrade clobbers local patches" pattern. The framework warned about the gap (step 1 prints the lost-line count) but did not block — printing a warning is not prevention, just observation. G-055 codifies the gap. No test exercises "fw upgrade preserves project-specific CLAUDE.md sections". The structural state has persisted for 7+ days across multiple consumers (T-1447 hit it ≥2 times within a week, then today's incident on /opt/termlink — the third documented occurrence in the same lineage).

**Prevention:**
1. **Primary** — sentinel markers in CLAUDE.md template. Wrap the framework-managed governance text with HTML comment markers like `<!-- fw-upgrade:governance-start -->` and `<!-- fw-upgrade:governance-end -->`. Step 1 of `fw upgrade` only rewrites content BETWEEN the markers. Project-specific lines added inside the marker block are still lost — but lines OUTSIDE the markers (the common case for TermLink) are preserved automatically.
2. **Secondary** — per-section opt-out via `.framework.yaml`. Operator declares `governance_section_managed: false` (default true) to skip step 1 entirely. Operator who knows their CLAUDE.md is hand-curated turns the auto-refresh off.
3. **Tertiary** — block on lost lines unless `--accept-clobber` is passed. The warning becomes a refusal. Operator runs `fw upgrade --accept-clobber` to acknowledge.
4. **Test** — `tests/e2e/upgrade-test.sh` adds a regression: seed a CLAUDE.md with `# PROJECT_MARKER` line outside the framework's governance section, run `fw upgrade`, assert the marker still exists.

Refer to T-1447 (PL-124 origin) and T-1069 (PL-022) for prior diagnoses. T-2014 / T-2099 lineage is the analog for the fork-bomb path.

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

### 2026-06-06T15:12Z — pickup envelope delivered to framework:pickup [agent autonomous, focus=T-2015]

Operator nudge: "why not send a pickup note?" — taken. Posted structured `pickup-bug-report` envelope to `framework:pickup` topic on local hub.

**Delivery:**
- Topic: `framework:pickup`
- msg_type: `pickup-bug-report`
- pickup_id: `termlink-G-055-T-2015-2026-06-06`
- Offset: **34**
- ts: `1780752479898` (2026-06-06T15:12Z)
- Payload: full RCA + 4 prevention options + ref to `docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md`

**Queue health flag (G-019 escalation candidate):** `channel.info framework:pickup` returns `count: 36, receipts: []` — 36 envelopes accumulated since topic creation, ZERO receipts. The most recent prior pickup (G-082) sat ~31 days unread. Pickups are reaching the durable channel correctly, but no active consumer is acking. The "framework-agent reads framework:pickup" assumption embedded in T-1899 Q2/A7 may need re-confirmation. **NOT my job to fix on this turn** — but flag for operator decision: either spin up a pickup-observer at /opt/999-AEF, or the receipts:[] gap becomes a separate concern (sibling to G-061 framework-blindness-to-bus-bridges).

Sibling T-2016 pickup posted at offset 35 in the same dispatch window.

### 2026-06-06T15:08Z — fresh occurrence #4 during operator-triggered `fw upgrade` [agent autonomous, focus=T-2015]

Operator typed `fw upgrade` on `/opt/termlink` (root@.107). Pre-upgrade state captured (md5 = `916e6d22ad94775b0f306e90c9f6fd90`, 72414 bytes, 1186 lines). Step 1/10 fired the clobber.

**Damage list (16 line-deletions, 1 modification, net -4036 bytes, same line count):**

| Lost | What it was |
|---|---|
| `.agentic-framework/bin/fw` vendored-path rule (modified down to template's generic "Use `bin/fw` not `fw`") | Per-project memory pin from feedback_fw_path_consumer |
| DEFER `revisit_at` protocol (T-1451 / G-053) | Entire inception-defer revisit-mechanism rule |
| 11 slash-command table rows | `/agent-handoff`, `/reply`, `/check-arc`, `/check-outbox`, `/recent-dm`, `/be-reachable`, `/peers`, `/recent-chat`, `/broadcast-chat`, `/pulse`, `/conversations` |
| `/be-reachable` session-start opt-in step | Step 7 of Session Start Protocol |
| `/be-reachable` session-end stop step | Step 8 of Session End Protocol |

**Restored via** `cp /tmp/claudemd-pre-upgrade.bak CLAUDE.md` (post-restore md5 verified = pre-upgrade md5). The framework's `CLAUDE.md.bak` would also have worked — the framework backup behavior is correct, but the warning is non-blocking.

**T-2014 status by comparison:** auto-clone fork-bomb fix CONFIRMED LANDED — `fw upgrade` cleanly executed `Bare-from-consumer detected — auto-cloning upstream` → handoff → no infinite loop. Closing the T-2014 lineage is possible (T-2099 upstream fix verified live).

**T-2015 still active:** the template-merge clobber is still in upstream `lib/upgrade.sh` step 1. Framework-agent prompt at `docs/reports/T-2015-fw-upgrade-claudemd-clobber-framework-prompt.md` is ready for operator paste.

**Step 4c shim refusal:** `REFUSED  /root/.local/bin/fw resolves into a framework repo (/root/.agentic-framework/bin/..)` — this is a SAFETY guard, not a bug. Global shim correctly points to `/root/.agentic-framework/bin/fw` (the framework repo's own bin/fw). Refusal prevented self-overwrite. No action needed.

### 2026-06-06T11:14:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2015-fw-upgrade-claudemd-clobber--propagate-t.md
- **Context:** Initial task creation
