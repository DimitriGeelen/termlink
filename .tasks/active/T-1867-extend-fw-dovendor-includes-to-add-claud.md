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
last_update: 2026-05-29T21:37:53Z
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
- [ ] Upstream `lib/templates/skills/` directory created and populated with the 9 doorbell+mail skill `.md` files (be-reachable, peers, recent-chat, recent-dm, broadcast-chat, pulse, conversations, check-arc, agent-handoff)
- [ ] Upstream `lib/templates/scripts/` directory created and populated with the 11 supporting script `.sh` files (agent-chat-arc-recent, recent-dm, agent-listeners, agent-listeners-fleet, chat-arc-broadcast, agent-conversation-list, agent-conversation-status, agent-send, agent-respond, listener-heartbeat, be-reachable) — chmod 755 in source
- [ ] `lib/upgrade.sh` extended with a loop (placed next to the existing resume.md block at ~1060-1090) that iterates `lib/templates/skills/*.md` and `lib/templates/scripts/*.sh`, applies compare-and-update-with-backup to project-root `.claude/commands/` and `scripts/`, and preserves script execute bit
- [ ] Drift-and-backup semantics confirmed: a consumer-edited skill triggers `.bak` then update (same as resume.md flow)
- [ ] dry-run output lists each propagated file under `WOULD UPDATE` / `WOULD CREATE`
- [ ] /opt/termlink round-trip test: run upstream's `lib/upgrade.sh` flow against /opt/termlink, confirm: (a) the 11 scripts arrive at `/opt/termlink/scripts/` executable, (b) consumer-local script (e.g. /opt/termlink/scripts/chat-arc-multicast.sh) NOT in the propagated set survives untouched, (c) the 9 skills arrive at `/opt/termlink/.claude/commands/`, (d) consumer-local skill (e.g. /opt/termlink/.claude/commands/heartbeat.md) survives
- [ ] Upstream commit message references T-1867 + names the corrected pattern
- [ ] `fw upgrade --help` text updated to mention "doorbell+mail toolkit propagation" alongside resume.md

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

### 2026-05-29 — original premise invalidated by destination-path discovery

- **What changed:** Original plan said "extend `bin/fw:254-264` `do_vendor`
  includes list with `.claude/commands` + `scripts`". Investigation while
  starting build revealed `bin/fw:221` sets `local dest="$target/.agentic-framework"`
  — so anything in the `includes` array is copied INTO the vendored
  `.agentic-framework/` subdir, NOT to the consumer's project root where
  claude-code reads from. Adding `.claude/commands` to `includes` would
  put files at `<consumer>/.agentic-framework/.claude/commands/`, invisible
  to claude-code.
- **Plan impact:** The stated AC slate ("extend includes list") cannot
  achieve the goal. T-1865 spike-2 confirmed the includes list excluded
  these paths, but did not verify destination semantics. False positive.
- **Triggered:** Need to reshape T-1867 around the actually-viable pattern.
  The existing project-root propagation path for `resume.md` lives in
  `lib/upgrade.sh:1063-1090` and uses `lib/templates/resume-md.md` as
  source, project-root `.claude/commands/resume.md` as target, with `.bak`
  backup on drift. That's the operative pattern.

### 2026-05-29 — additional --delete hazard noted (independent of path issue)

- **What changed:** `bin/fw:321` uses `rsync -a --delete --delete-excluded`
  per include. Even if a hypothetical project-root-targeted include
  worked, naive add would clobber consumer-local files. PL-124-class
  hazard. Mitigation: either additive copy semantics (Option B) or
  per-file pattern (Option C — which is what resume.md already does).
- **Plan impact:** Option C (per-file via lib/templates + lib/upgrade.sh)
  has the side-benefit of being inherently additive — it only touches
  the specific files it knows about, never sweeps a directory.

## Decisions

### 2026-05-29 — corrected approach: lib/templates loop, not do_vendor includes

- **Chose:** Stage the 9 doorbell+mail skills + 11 scripts under
  `lib/templates/skills/*.md` and `lib/templates/scripts/*.sh` in upstream
  AEF, then add ONE loop in `lib/upgrade.sh` (next to the existing
  resume.md block at lines 1060-1090) that iterates them and applies the
  same compare-and-update-with-backup pattern. One loop, 20 files.
- **Why:** This is the only pattern that puts files where claude-code
  reads from (consumer project root). It also inherits the resume.md
  block's drift-detection + `.bak` semantics, which means consumer-local
  edits to a vendored skill survive across upgrades. PL-124-safe by
  construction. No new vendor contract architecture.
- **Rejected — Option A (do_vendor includes naïve add):** invalid because
  destination is `.agentic-framework/`, not project root. Discovery
  during build (see Evolution above).
- **Rejected — Option B (additive copy in do_vendor):** does not solve
  the project-root targeting problem. Even with `--delete` skipped,
  files land in `.agentic-framework/`, invisible to claude-code.
- **Rejected — Option C-per-file (20 hand-coded branches in upgrade.sh):**
  works but copy-paste tax. Replaced by Option C-looped (above).
- **Rejected — Option D (new "additive-includes" contract on do_vendor):**
  bigger change, conceptually attractive, but premature for one toolkit.
  Revisit if a third project-root-targeting subsystem appears.

### 2026-05-29 — scripts/spikes/ handling

- **Chose:** Don't include `scripts/spikes/` in vendoring at all. Only
  the 11 specific named `*.sh` scripts go through the lib/templates loop.
- **Why:** `scripts/spikes/` (T-1736-* / T-1740-* / etc Python metrics)
  is framework-side R&D, not consumer-bound. The per-file loop naturally
  excludes it by enumerating only the doorbell+mail subset.
- **Rejected — vendor whole `scripts/` dir with exclude pattern:** would
  require do_vendor's directory-mode (which also has the destination-path
  problem). Per-file loop sidesteps both.

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
