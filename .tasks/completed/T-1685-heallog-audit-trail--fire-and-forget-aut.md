---
id: T-1685
name: "heal.log audit trail — fire-and-forget auto-heal needs a record"
description: >
  heal.log audit trail — fire-and-forget auto-heal needs a record

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-18T06:24:51Z
last_update: 2026-05-18T06:28:17Z
date_finished: 2026-05-18T06:28:17Z
---

# T-1685: heal.log audit trail — fire-and-forget auto-heal needs a record

## Context

`--auto-heal` is fire-and-forget: subprocesses are spawned via
`Command::spawn()` and never reaped. T-1671's `rotation.log` captures
state transitions, but it doesn't record the heal action itself —
operator running `--watch --auto-heal` overnight has no answer to
"what did you do?". Even the one-shot mode has no record of which
hubs fired and which were skipped after the terminal scrolls.

Add `~/.termlink/heal.log` (NDJSON, symmetric to `rotation.log`) and
append one line per heal action. Schema:
- ts: RFC3339
- hub: profile name
- mode: "watch" | "one-shot"
- trigger: "cert-drift" | "auth-mismatch"
- action: "fired" | "skipped-no-anchor" | "dry-run"
- bootstrap_from: declared anchor string or null (for skipped)

Best-effort write — failures go to stderr, never block the heal or
the watch loop. Same shape and discipline as `append_rotation_log`.

No reader command in this scope — operator runs `cat ~/.termlink/heal.log | jq`.
A future task may add `fleet history --include-heals` once usage is
established.

## Acceptance Criteria

### Agent
- [x] `append_heal_log(hub, mode, trigger, action, bootstrap_from)` helper added immediately above `append_rotation_log` with matching shape + best-effort error handling
- [x] Single-shot path: fire/dry-run/skip-no-anchor all append one entry (verified live)
- [x] Watch path: same three cases all append entries via the transition handler; rotation.log calls untouched
- [x] Write failures emit to stderr (`heal.log write failed (...)`) without crashing
- [x] `cargo check -p termlink` passes clean
- [x] Smoke (dry-run with anchor): `{"action":"dry-run","bootstrap_from":"file:/tmp/fake-anchor","hub":"ring20-dashboard","mode":"one-shot","trigger":"cert-drift","ts":"2026-05-18T06:27:03Z"}`
- [x] Smoke (skip-no-anchor): `{"action":"skipped-no-anchor","bootstrap_from":null,"hub":"ring20-dashboard","mode":"one-shot","trigger":"cert-drift","ts":"2026-05-18T06:27:29Z"}`
- [x] State restored after smoke (known_hubs + hubs.toml + heal.log)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo check -p termlink 2>&1 | tail -5

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

### 2026-05-18T06:24:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1685-heallog-audit-trail--fire-and-forget-aut.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-5d98f77f
- **Timestamp:** 2026-05-18T06:28:26Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-18T06:28:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
