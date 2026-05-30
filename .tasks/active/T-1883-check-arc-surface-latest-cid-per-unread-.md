---
id: T-1883
name: "check-arc: surface latest cid per unread DM topic"
description: >
  check-arc: surface latest cid per unread DM topic

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T16:37:38Z
last_update: 2026-05-30T16:37:38Z
date_finished: null
---

# T-1883: check-arc: surface latest cid per unread DM topic

## Context

T-1881 made `conversation_id` visible in `/recent-dm`. T-1882 made it
targetable via `/reply --cid`. The remaining gap is at the entry point:
`/check-arc` (the inbox view) currently shows topic name + unread count
but not the latest cid. Operator landing fresh has to peek (`/recent-dm`)
to know which thread `/reply` would hit.

This task surfaces latest cid per topic in `/check-arc`'s Step-4 render
so the inbox view becomes a one-look diagnostic: "do I `/reply` (default
latest cid X), `--cid <Y>` (target a non-latest), or `--ensure-cid`
(first structured turn on a chat-style topic)?"

Implementation is markdown-only: the skill is read-and-executed by the
claude session, so updating `.claude/commands/check-arc.md` to add a
`channel subscribe --limit 5` cid-extract step + render row IS the
implementation.

## Acceptance Criteria

### Agent
- [x] Cid-extract one-liner works on a real DM topic (sanity check that the jq path returns the load-bearing cid)
- [x] `.claude/commands/check-arc.md` Step 3 includes a per-topic cid-extraction sub-step (using the same `channel subscribe --limit 5 --json | jq` shape as T-1880's agent-reply.sh)
- [x] `.claude/commands/check-arc.md` Step 4 renders `latest_cid=<short>` (or `latest_cid=-` if none) on each topic row
- [x] Step 4's reply-hint footer points at the right verb per cid state (cid present → `/reply <peer>`, cid missing → `/reply <peer> --ensure-cid`)
- [x] Cross-link maintained: Related footer / Pair-with section references T-1881 + T-1882

<!-- All criteria are mechanically verifiable — no Human section. -->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# Skill doc references the cid-extract jq path.
grep -q "metadata.conversation_id" .claude/commands/check-arc.md
# Step 4 render includes latest_cid line.
grep -q "latest_cid" .claude/commands/check-arc.md
# Reply-hint differentiates between cid-present and cid-missing states.
grep -q -- "--ensure-cid" .claude/commands/check-arc.md
# Cross-link maintained.
grep -q "T-1881\|T-1882" .claude/commands/check-arc.md
# Cid extraction works against the self-DM smoke topic (--limit 100 mirrors agent-reply.sh).
test -n "$(termlink channel subscribe dm:d1993c2c3ec44c94:d1993c2c3ec44c94 --limit 100 --json 2>/dev/null | jq -sr 'map(select(.metadata.conversation_id != null)) | sort_by(.offset) | .[-1].metadata.conversation_id // empty')"
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

### 2026-05-30T16:37:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1883-check-arc-surface-latest-cid-per-unread-.md
- **Context:** Initial task creation
