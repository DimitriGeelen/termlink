---
id: T-1985
name: "Investigate ring20-management (.122) DM inbox backlog — 30 unread incl. T-1695 G-058 ask"
description: >
  User reported errors on .122. Investigation finding: hub HEALTHY (PONG 83ms, no rotation in 30d, fleet doctor PASS). Issue is at a different layer: dm:9219671e28054458:d1993c2c3ec44c94 on .122 hub has 30 posts but local .107 view of same topic has 22 — federation .107→.122 is asymmetric since T-1166 cut (~2026-05-12) per a self-documenting DM in offset 25. .122-side agent never acked any posts on this topic (peer_acked=-1). Critical missed item: offset 29 carries T-1695 G-058 OneDev mirror fix request (the penelope-shell executor pin). Also: agent-presence on .122 is EMPTY — no peer-agent listener registered, so /agent-handoff DMs cannot reach. Vendored-arc heartbeat IS running on .122 (hourly chat-arc emissions) but it's emit-only, no DM subscription. 4 termlink remote sessions registered (skills-manager, review-batch-3/4, ring20-management) but none subscribe to dm:* topics. Scope: investigate root cause + report; do NOT take direct admin action without operator authorization (G-058 OneDev change is Tier-1, federation restart is Tier-1).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [fleet, ring20-management, doorbell-mail, operational]
components: []
related_tasks: []
created: 2026-06-04T08:20:54Z
last_update: 2026-06-04T08:27:41Z
date_finished: null
---

# T-1985: Investigate ring20-management (.122) DM inbox backlog — 30 unread incl. T-1695 G-058 ask

## Context

Operator-driven investigation + restoration. User reports persistent "errors with .122". From .107: hub is healthy (PASS 44ms, no rotation 30d). The pain is two layers up — federation .107→.122 is asymmetric since T-1166 cut (~2026-05-12); the .122 DM inbox has 30 unread items including the T-1695 G-058 OneDev mirror fix ask; no agent-presence listener is attached on .122 so peer doorbell rings fail. Predecessor: T-1457 (register identity on .141 agent-1 session — same gap, different host). Related: T-1841 (`/be-reachable`), T-1840 (systemd listener-heartbeat template).

## Acceptance Criteria

### Agent
- [ ] Read .122-side `dm:9219671e28054458:d1993c2c3ec44c94` offsets 22-29 (8 unread) — capture summary of operationally hot items in task Updates
- [ ] Read `dm:33df8954b2a9b70d:ring20-management-agent` (4 posts) — identify what was sent and from whom
- [ ] Probe federation gap: confirm whether posts made FROM .107 to local channel still relay to .122 hub, or whether cross-host posts must use `--hub <addr>` workaround
- [ ] Attempt listener restoration on .122 via `termlink remote exec ring20-management <session-id>` — start a presence-emitter for at least one stable session
- [ ] Verify agent-presence on .122 has ≥1 LIVE listener after restoration attempt (`bash scripts/agent-listeners.sh --hub 192.168.10.122:9100 --include-offline --json`)
- [ ] If restoration succeeds, ack the .122-side DM inbox via `termlink channel ack <topic> --hub 192.168.10.122:9100` (operator-explicit)
- [ ] Record findings in task Updates AND register a learning if any structural gap is identified (e.g. federation regression, listener-not-self-healing)
- [ ] If federation gap is a code defect, file a follow-up task (separate from T-1985) with reproduction steps

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
set -o pipefail; timeout 15 termlink remote ping ring20-management 2>&1 | grep -q "PONG"

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

### 2026-06-04T08:20:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1985-investigate-ring20-management-122-dm-inb.md
- **Context:** Initial task creation

### 2026-06-04T08:26:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
