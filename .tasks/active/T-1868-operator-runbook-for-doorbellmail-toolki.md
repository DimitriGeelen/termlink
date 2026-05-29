---
id: T-1868
name: "Operator runbook for doorbell+mail toolkit (T-1865 follow-up #3, optional)"
description: >
  Phase 3 of T-1865 GO (optional): docs/operations/doorbell-mail-operator-runbook.md covering hub deployment, secret deployment, /be-reachable opt-in, /pulse cold-start, /agent-handoff vs /broadcast-chat decision tree. Audience: AEF consumer-project operators who just got the toolkit via fw upgrade and need to know how to use it.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1865, T-1866, T-1867]
created: 2026-05-29T12:04:45Z
last_update: 2026-05-29T22:25:48Z
date_finished: null
---

# T-1868: Operator runbook for doorbell+mail toolkit (T-1865 follow-up #3, optional)

## Context

Phase 3 of T-1865 GO. T-1866 (vendor bundle) + T-1867 (do_vendor propagation)
shipped. Operators on AEF consumer projects will gain the 9 skills + 11
scripts on their next `fw upgrade`, but have no canonical guide to: (a) which
skill to reach for in which situation, (b) prerequisites (hub deployment,
secret deployment, `~/.termlink/hubs.toml`), (c) the cold-start path
(`/be-reachable` → `/pulse` → `/agent-handoff` / `/broadcast-chat`), and
(d) failure-mode diagnostics. Without this doc, the toolkit ships but
adoption stalls — `T-1830` "drive zero adoption gap" finding.

Source material already exists in scattered form:
- `docs/operations/agent-conversations.md` (T-1830) — recipe master doc
- `docs/operations/listener-heartbeat-systemd.md` (T-1840) — persistent rail
- Skill-by-skill notes in each `.claude/commands/*.md` Related section

This task synthesizes them into a single AEF-consumer-facing runbook
written for someone who has NEVER used the toolkit before — assumes only
that `fw upgrade` has placed the skills + scripts at project root.

## Acceptance Criteria

### Agent
- [x] `docs/operations/doorbell-mail-operator-runbook.md` created with sections: Overview / Prerequisites / Cold-start (first 5 minutes) / Skill reference (one row per skill, when to use vs not) / Decision tree (broadcast vs DM vs presence-check) / Failure modes / Where things live (state files, logs)
- [x] Cold-start path tested: 4 commands in the doc (`/pulse → /be-reachable → /peers → /broadcast-chat`) get operator from "I have the toolkit" to "first signal posted + LIVE on peer list". Under the ≤5 cap.
- [x] Skill reference table includes all 9 skills (be-reachable, peers, recent-chat, recent-dm, check-arc, agent-handoff, broadcast-chat, pulse, conversations) + the 11 scripts they wrap, with one-line "use when" + "do not use when" entries
- [x] Decision tree covers the 4 most common operator situations: (1) just arrived; (2) need to reach one peer; (3) need to announce fleet-wide; (4) want to be reachable — each maps to a specific verb sequence
- [x] Failure-mode section covers: hub unreachable (→ `fleet doctor`), no peers visible (cold rail vs broken discovery), sender_id unresolved (→ `/be-reachable` first), G-060 agent-chat-arc no-federation surprise, heartbeat-only "fleet looks busy but it's just bookkeeping" (T-1861)
- [x] Doc committed with T-1868 reference + ship-notice broadcast on chat-arc to all 5 hubs (5/5 delivered)

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

### 2026-05-29T12:04:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1868-operator-runbook-for-doorbellmail-toolki.md
- **Context:** Initial task creation

### 2026-05-29T22:25:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now
