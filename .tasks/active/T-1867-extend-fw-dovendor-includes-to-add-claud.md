---
id: T-1867
name: "Extend fw do_vendor includes to add .claude/commands/ + scripts/ (T-1865 follow-up #2)"
description: >
  Phase 2 of T-1865 GO: structural change to .agentic-framework/bin/fw do_vendor() includes list (line 254-264) to add .claude/commands/ and scripts/ so the upstream toolkit propagates to consumer projects on next fw upgrade. Depends on T-1866 (toolkit must be upstream first). HIGH-IMPACT — affects every existing AEF consumer project. Requires careful review for upgrade-path conflicts (e.g. consumer-local skills that would be overwritten).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1865, T-1866, T-1868]
created: 2026-05-29T12:04:41Z
last_update: 2026-05-29T21:36:17Z
date_finished: null
---

# T-1867: Extend fw do_vendor includes to add .claude/commands/ + scripts/ (T-1865 follow-up #2)

## Context

Phase 2 of T-1865 GO. T-1866 just shipped the doorbell+mail bundle into
upstream `/opt/999-AEF` at commit `10d05e76`. Without this task, the
toolkit is upstream-only — consumer projects don't get it on `fw upgrade`
because the vendoring contract excludes `.claude/commands/` and `scripts/`.

**Current vendor contract** (`bin/fw:254-264`, the canonical `do_vendor`
includes list):

```bash
local includes=(
    bin
    lib
    agents
    web
    docs
    .tasks/templates
    FRAMEWORK.md
    metrics.sh
)
```

`.claude/commands/` and `scripts/` are absent. Spike-2 of T-1865 confirmed.

**High-impact / hazard scoping:**

1. **Consumer-local skill clobber (PL-124 class).** A consumer that has
   built its own `.claude/commands/foo.md` would lose it if `fw upgrade`
   blindly mirrors upstream-only files. The fix MUST be additive:
   upstream files arrive without removing consumer-local files not
   present upstream.
2. **Upstream framework-default skills come too.** The upstream
   `.claude/commands/` has 10 framework-default skills today (capture,
   deploy-check, explore, new-project, plan, resume, review, rollback,
   start-work, write) — these aren't currently vendored. Bringing
   `.claude/commands/` in includes-list brings them too, and consumers
   gain those alongside the doorbell+mail bundle. That is the design
   intent for this task; it is NOT considered scope creep.
3. **`scripts/spikes/` exclusion.** Upstream `scripts/` contains a
   `spikes/` subdir for framework-side R&D. Consumer projects shouldn't
   vendor those — should the include be `scripts/` (whole dir) or
   `scripts/*.sh` (top-level only)? Decision needed.
4. **Test on /opt/termlink before broadcasting.** /opt/termlink is its
   own consumer of AEF — when we land T-1867 and pull upstream, our own
   `.claude/commands/` and `scripts/` should round-trip without local
   loss. This is the build-loop test.

## Acceptance Criteria

### Agent
- [ ] Upstream `bin/fw:254-264` `do_vendor` includes list extended with `.claude/commands` and `scripts` (or scoped form) — committed on `origin/master`
- [ ] Vendor semantics confirmed additive: a consumer-local `.claude/commands/local-only.md` survives a `fw upgrade` that brings in upstream files (proven via dry-run or controlled smoke)
- [ ] Decision on `scripts/spikes/` exclusion recorded in `## Decisions` (whole dir vs `scripts/*.sh` top-level only)
- [ ] /opt/termlink round-trip test: pull upstream `master` into a sandbox copy of the framework, run `do_vendor` against `/opt/termlink`, confirm the 9 doorbell+mail skills + 11 scripts appear in `/opt/termlink/.claude/commands/` and `/opt/termlink/scripts/` AND no pre-existing local file is deleted
- [ ] PL-124 protection: if vendor logic uses rsync `--delete` semantics anywhere on `.claude/commands/` or `scripts/`, that path is patched OR `--delete` is scoped so consumer-local files survive
- [ ] Upstream commit message references T-1867 + cites this safety analysis
- [ ] Brief note in `lib/upgrade.sh` or fw help text that `.claude/commands/` and `scripts/` are now consumer-propagated (so operators reading source know)

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

### 2026-05-29T12:04:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1867-extend-fw-dovendor-includes-to-add-claud.md
- **Context:** Initial task creation

### 2026-05-29T21:36:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
