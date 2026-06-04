---
id: T-1989
name: "PL-200 closure on .121 ring20-dashboard (T-1988 sibling)"
description: >
  T-1988 closed PL-200 on .141 + validated .122 end-to-end. Fleet listener walk shows ring20-dashboard (.121) still has zero presence-emitter entries — same PL-200 gap as .141. Install presence-heartbeat cron on .121 following the T-1988 PL-146-aware recipe. Verify ring20-dashboard-agent appears LIVE in fleet. Tags: pl-200, ring20-dashboard, doorbell-mail.

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: [pl-200, doorbell-mail, ring20]
components: []
related_tasks: []
created: 2026-06-04T20:15:20Z
last_update: 2026-06-04T20:16:25Z
date_finished: null
---

# T-1989: PL-200 closure on .121 ring20-dashboard (T-1988 sibling)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Probe .121: hub runs as root (UID 0), TERMLINK_RUNTIME_DIR=/var/lib/termlink (post T-1294/T-1296), bin /usr/local/bin/termlink, scripts at /root/termlink/scripts/, self-fp 33df8954b2a9b70d, hostname dashboard-agent
- [x] presence-heartbeat.sh installed at /root/termlink/scripts/presence-heartbeat.sh — uses 3-tier runtime_dir fallback chain (try /tmp/termlink-$(id -u)/hub.sock → /var/lib/termlink → $HOME/.termlink/runtime), agent_id=ring20-dashboard-agent, listen_topics=dm:33df8954b2a9b70d:*,agent-chat-arc
- [x] Cron entry `* * * * * /root/termlink/scripts/presence-heartbeat.sh >> /var/log/presence-heartbeat.log 2>&1` added (idempotent: grep -qF check before append)
- [x] ring20-dashboard-agent LIVE on .121 hub within ≤90s: `bash scripts/agent-listeners.sh --hub 192.168.10.121:9100 --include-offline --json` returns LIVE/age=2s
- [x] PL-200 learning updated with .121 fleet status entry; .141 memory note left unchanged (already current from T-1988)

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
# Verify ring20-dashboard-agent LIVE on .121 hub
set -o pipefail; timeout 15 bash scripts/agent-listeners.sh --hub 192.168.10.121:9100 --include-offline --json 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print('OK' if any(l.get('agent_id')=='ring20-dashboard-agent' and l.get('status')=='LIVE' for l in d.get('listeners',[])) else 'MISSING')" | grep -q "^OK$"

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

### 2026-06-04T20:15:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1989-pl-200-closure-on-121-ring20-dashboard-t.md
- **Context:** Initial task creation

### 2026-06-04T20:18Z — .121 install + recipe generalized
- **Probe:** hub runs as root, TERMLINK_RUNTIME_DIR=/var/lib/termlink (post T-1296), socket at /var/lib/termlink/hub.sock, default /tmp/termlink-0/hub.sock absent.
- **Recipe:** generalized T-1988 fallback into a 3-tier chain (`default → /var/lib/termlink → $HOME/.termlink/runtime`) so the same script template works on both root-hubs with migrated runtime (e.g. .122, .121) AND non-root hubs (e.g. .141 dimitri). Sim-cron posted offset=0 first try, no detour needed.
- **Outcome:** ring20-dashboard-agent LIVE, age=2s, ~90s after cron install. PL-200 fleet coverage: 3/3 ring20-family + sibling hosts (.122, .141, .121) all LIVE. PL-200 learning updated with snapshot + recipe note.
- **Files touched on .121:** /root/termlink/scripts/presence-heartbeat.sh + root crontab append (T-1989 comment + cron line).
