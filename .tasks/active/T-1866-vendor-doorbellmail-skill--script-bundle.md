---
id: T-1866
name: "Vendor doorbell+mail skill + script bundle into upstream AEF (T-1865 follow-up #1)"
description: >
  Phase 1 of T-1865 GO decision: ship the 9 doorbell+mail slash skills + 7 supporting scripts + systemd presence-emitter template into upstream /opt/999-Agentic-Engineering-Framework via direct termlink_run commit. No behavioral change — file copy only. Enables T-1867 to extend do_vendor includes so the toolkit reaches consumers.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1865, T-1867, T-1868]
created: 2026-05-29T12:04:27Z
last_update: 2026-05-29T12:04:27Z
date_finished: null
---

# T-1866: Vendor doorbell+mail skill + script bundle into upstream AEF (T-1865 follow-up #1)

## Context

Phase 1 of T-1865 GO decision. Ship the doorbell+mail toolkit into upstream
AEF (`/opt/999-Agentic-Engineering-Framework`) so it becomes part of the
framework codebase. **No behavioral change** — file copies only.

Memory `workflow_channel1_upstream_mirror` documents the upstream-write
pattern: use `termlink_run` MCP (T-559 blocks Bash on /opt/999-AEF...);
upstream remote is `origin` (OneDev) on branch `master`; verify landing
directly after push.

**Skills to vendor (9):** `.claude/commands/{be-reachable,peers,recent-chat,recent-dm,broadcast-chat,pulse,conversations,check-arc,agent-handoff}.md`

**Scripts to vendor (~9 — superset of the 7 minimum):**
- `agent-chat-arc-recent.sh` (T-1849)
- `recent-dm.sh` (T-1862)
- `agent-listeners.sh` (T-1833)
- `agent-listeners-fleet.sh` (T-1837)
- `chat-arc-broadcast.sh` (T-1857)
- `agent-conversation-list.sh` (T-1827)
- `agent-conversation-status.sh` (T-1826)
- `agent-send.sh` (T-1830)
- `agent-respond.sh` (T-1830)
- `listener-heartbeat.sh` (T-1832)
- `be-reachable.sh` (T-1841)

Plus systemd template at `docs/operations/listener-heartbeat-systemd.md` (T-1840).

T-1867 (next) is the structural change to `do_vendor` that propagates these
to consumers; this task makes that change possible by putting the files
upstream.

## Acceptance Criteria

### Agent
- [ ] All 9 doorbell+mail slash skills present in upstream `/opt/999-Agentic-Engineering-Framework/.claude/commands/`
- [ ] All ~11 supporting scripts present in upstream `/opt/999-Agentic-Engineering-Framework/scripts/`
- [ ] systemd template doc present at upstream `docs/operations/listener-heartbeat-systemd.md`
- [ ] Upstream commit landed on `origin/master` (OneDev) with task ref `T-1866` in message
- [ ] Vendored files are byte-identical to /opt/termlink originals at commit time (no edits during transit)
- [ ] Upstream `.claude/commands/recent-dm.md` references `scripts/recent-dm.sh` consistently — no broken script-path refs
- [ ] Live demo: read one vendored skill back from upstream and confirm it matches the local source

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

# Upstream presence checks (via termlink_run since T-559 blocks Bash on /opt/999-AEF):
# Each verifies a representative file landed at the expected upstream path.
# Full enumeration of all 9 skills + ~11 scripts is in the upstream git log.
# Run remotely; local Bash cannot reach the upstream path.
# (Commented because P-011 runs them in local Bash — actual verification done
#  via termlink_run during the build.)
# bash -c 'ls /opt/999-Agentic-Engineering-Framework/.claude/commands/recent-dm.md'
# bash -c 'ls /opt/999-Agentic-Engineering-Framework/scripts/recent-dm.sh'

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

### 2026-05-29T12:04:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1866-vendor-doorbellmail-skill--script-bundle.md
- **Context:** Initial task creation
