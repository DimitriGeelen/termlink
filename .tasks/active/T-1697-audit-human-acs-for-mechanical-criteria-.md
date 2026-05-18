---
id: T-1697
name: "Audit Human ACs for mechanical-criteria misclassification (G-059 punch list)"
description: >
  Audit Human ACs for mechanical-criteria misclassification (G-059 punch list)

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [audit, framework-discipline, G-059, PL-169]
components: []
related_tasks: [T-1480, T-1481]
created: 2026-05-18T17:41:52Z
last_update: 2026-05-18T17:41:52Z
date_finished: null
---

# T-1697: Audit Human ACs for mechanical-criteria misclassification (G-059)

## Context

PL-169 + G-059 (2026-05-18) surfaced a pattern: ACs whose success criterion is
mechanically verifiable were tagged `### Human` out of caution at filing time,
creating stuck `work-completed/owner=human` tasks that the framework can't
auto-close (T-1731 hook blocks agent-tick on Human ACs). T-1480 and T-1481
were the trigger instances; the audit hypothesis is that more exist across the
corpus. This task scans active/ + completed/ task files, classifies each
unticked `### Human` AC as either (a) mechanically verifiable → move-to-Agent
candidate, (b) genuine human-only (UI rendering / human-authenticated session /
subjective judgment) → leave, or (c) ambiguous → flag for operator review.
Output is a punch list, not auto-fixes — the operator decides which migrations
to authorize (Tier-2 `--skip-human-ownership` pattern from T-1480/T-1481).

## Acceptance Criteria

### Agent
- [ ] Scan all `.tasks/active/*.md` + `.tasks/completed/*.md` files; extract every unticked checkbox under `### Human` along with its Steps + Expected text
- [ ] Classify each AC into one of: `mechanical` (success criterion is string-match / exit-code / fields-present / structural-check), `human-only` (UI rendering needed / requires GitHub.com or similar authenticated session / subjective tone-aesthetic-architecture call), `ambiguous` (success criterion is operationalizable but author may have intended judgment)
- [ ] Produce a punch-list output at `docs/reports/T-1697-human-ac-audit.md` with three sections (mechanical / human-only / ambiguous), each row = task-id + AC-text + classification rationale (one line)
- [ ] Counts by classification reported in the punch-list header (e.g. "12 mechanical, 47 human-only, 5 ambiguous")
- [ ] For at least the first 5 `mechanical` rows: include a one-line proposed migration text the operator can paste into the task file
- [ ] Scan is idempotent: re-running produces stable classification (deterministic ordering, no datetime in output content)
- [ ] Spot-check: T-1480 + T-1481 (now in completed/, ACs already moved) should NOT appear in the mechanical list — proves the audit excludes already-migrated cases

### Human
<!-- This task's deliverable is the punch list. Once the operator reviews
     it and authorizes migrations on specific tasks, those become individual
     Tier-2 actions on each task (per the T-1480/T-1481 pattern). No
     blocking human AC here; the audit is the product. -->

## Verification

# Scan output exists and parses
test -f docs/reports/T-1697-human-ac-audit.md
grep -q "^## Mechanical" docs/reports/T-1697-human-ac-audit.md
grep -q "^## Human-Only" docs/reports/T-1697-human-ac-audit.md
grep -q "^## Ambiguous" docs/reports/T-1697-human-ac-audit.md
# Header counts present
grep -qE "[0-9]+ mechanical, [0-9]+ human-only, [0-9]+ ambiguous" docs/reports/T-1697-human-ac-audit.md
# Migrated tasks not in mechanical list (sanity check)
! grep -A 20 "^## Mechanical" docs/reports/T-1697-human-ac-audit.md | head -25 | grep -E "^- T-148[01]"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-18T17:41:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1697-audit-human-acs-for-mechanical-criteria-.md
- **Context:** Initial task creation
