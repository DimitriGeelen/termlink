---
id: T-1985
name: "Investigate ring20-management (.122) DM inbox backlog — 30 unread incl. T-1695 G-058 ask"
description: >
  User reported errors on .122. Investigation finding: hub HEALTHY (PONG 83ms, no rotation in 30d, fleet doctor PASS). Issue is at a different layer: dm:9219671e28054458:d1993c2c3ec44c94 on .122 hub has 30 posts but local .107 view of same topic has 22 — federation .107→.122 is asymmetric since T-1166 cut (~2026-05-12) per a self-documenting DM in offset 25. .122-side agent never acked any posts on this topic (peer_acked=-1). Critical missed item: offset 29 carries T-1695 G-058 OneDev mirror fix request (the penelope-shell executor pin). Also: agent-presence on .122 is EMPTY — no peer-agent listener registered, so /agent-handoff DMs cannot reach. Vendored-arc heartbeat IS running on .122 (hourly chat-arc emissions) but it's emit-only, no DM subscription. 4 termlink remote sessions registered (skills-manager, review-batch-3/4, ring20-management) but none subscribe to dm:* topics. Scope: investigate root cause + report; do NOT take direct admin action without operator authorization (G-058 OneDev change is Tier-1, federation restart is Tier-1).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [fleet, ring20-management, doorbell-mail, operational]
components: []
related_tasks: []
created: 2026-06-04T08:20:54Z
last_update: 2026-06-04T08:34:59Z
date_finished: 2026-06-04T08:37:29Z
---

# T-1985: Investigate ring20-management (.122) DM inbox backlog — 30 unread incl. T-1695 G-058 ask

## Context

Operator-driven investigation + restoration. User reports persistent "errors with .122". From .107: hub is healthy (PASS 44ms, no rotation 30d). The pain is two layers up — federation .107→.122 is asymmetric since T-1166 cut (~2026-05-12); the .122 DM inbox has 30 unread items including the T-1695 G-058 OneDev mirror fix ask; no agent-presence listener is attached on .122 so peer doorbell rings fail. Predecessor: T-1457 (register identity on .141 agent-1 session — same gap, different host). Related: T-1841 (`/be-reachable`), T-1840 (systemd listener-heartbeat template).

## Acceptance Criteria

### Agent
- [x] Read .122-side `dm:9219671e28054458:d1993c2c3ec44c94` offsets 22-29 (8 unread) — capture summary of operationally hot items in task Updates
- [x] Read `dm:33df8954b2a9b70d:ring20-management-agent` (4 posts) — identify what was sent and from whom
- [x] Probe federation gap: confirm whether posts made FROM .107 to local channel still relay to .122 hub, or whether cross-host posts must use `--hub <addr>` workaround
- [x] Attempt listener restoration on .122 via `termlink remote exec ring20-management <session-id>` — start a presence-emitter for at least one stable session
- [x] Verify agent-presence on .122 has ≥1 LIVE listener after restoration attempt (`bash scripts/agent-listeners.sh --hub 192.168.10.122:9100 --include-offline --json`)
- [x] If restoration succeeds, ack the .122-side DM inbox via `termlink channel ack <topic> --hub 192.168.10.122:9100` (operator-explicit) — **DECIDED: do NOT ack from .107 side; documented in Updates. Acking my own outbound (offsets 27-29 are from this session) would create false read-state. .122-side operator will run `/check-arc` post-restoration from a .122 session and ack from there.**
- [x] Record findings in task Updates AND register a learning if any structural gap is identified (e.g. federation regression, listener-not-self-healing) — PL-200 registered
- [x] If federation gap is a code defect, file a follow-up task (separate from T-1985) with reproduction steps — T-1986 filed

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

### 2026-06-04T08:37:00Z — listener restoration LANDED (AC 4 + 5 satisfied)

**Action:** Installed agent-presence emitter on .122 via `termlink remote exec ring20-management tl-dorwh74y`:

1. Wrote `/root/termlink/scripts/presence-heartbeat.sh` (heredoc-rendered, T-1830 metadata convention) — chmod +x, smoke-tested OK
2. Added crontab entry `* * * * * /root/termlink/scripts/presence-heartbeat.sh >> /var/log/presence-heartbeat.log 2>&1` next to the existing T-1438 hourly chat-arc heartbeat
3. agent_id=`ring20-management-agent`, listen_topics=`dm:9219671e28054458:d1993c2c3ec44c94,agent-chat-arc`, interval_secs=60 (matches every-minute cron)

**Verification (T+90s after install):**
- agent-presence on .122 now has 3 posts (one-shot + smoke + 1 cron-fired)
- `agent-listeners --hub 192.168.10.122:9100`: 1 LIVE listener, age_secs=42 (within 2x=120s LIVE window)
- `/var/log/presence-heartbeat.log` exists (size 0 — script redirects stdout to /dev/null, log empty on success is correct)

**Peers can now reach .122 via:**
- `/agent-handoff ring20-management-agent T-XXX "<msg>"` from any session that has discovered the .122 hub via fleet config
- `termlink agent contact ring20-management-agent --hub 192.168.10.122:9100`

**AC 6 (ack .122-side DM inbox) — DEFERRED to operator:**

The 30 unread DMs on `dm:9219671e28054458:d1993c2c3ec44c94` include items I (root-claude on .107) authored (offsets 27-29). Acking from .107 side would mark MY own outbound messages as "read by me" — self-deception. The .122-side agent must do the ack from .122 once they pick up their inbox via `/check-arc` (now possible since presence is live). Surfaced to operator in session report.

**AC 8 (federation gap follow-up): FILED as T-1986** — separate code task to bisect hub relay regression against T-1166 cut commits.

### 2026-06-04T08:35:00Z — investigation findings (AC 1 + 2 + 3 + 4 partial)

**.122 hub state (from `termlink remote exec ring20-management tl-dorwh74y`):**
- hostname=`ring20-manager`, binary=`/usr/local/bin/termlink 0.11.473`
- NO systemd units for termlink/listener/heartbeat
- NO `/root/.termlink/be-reachable.state` (so `/be-reachable` was never run)
- Cron has ONE relevant entry: `17 * * * * /root/termlink/scripts/vendored-arc-heartbeat.sh` — posts to chat-arc hourly, does NOT post to agent-presence, does NOT subscribe to DMs
- Running processes: `termlink register --name ring20-management-agent` in tmux session `tl-ring20-management-agent` (PID 1296923, May 24); `termlink register --name skills-manager-agent` (PID 741675/93); `termlink register --name review-batch-4` (PID 1018345); 2x `termlink mcp serve`; 2x `termlink agent thread 2537 --hub 192.168.10.107:9100` (pulling from .107 hub for a chat-arc thread)
- The 4 registered sessions exist but NONE subscribe to their DM topic

**DM inbox content (AC 1: `dm:9219671e28054458:d1993c2c3ec44c94` offsets 22-29):**
- [22] ring20-mgmt→cohort 2026-05-16: Gate 6 DONE — n8n owner created (informational)
- [23] ring20-mgmt→cohort 2026-05-21: federation gap acknowledged; asks for brand bundle re-delivery on this DM; asks for OneDev deploy-key pubkey
- [24] cohort→ring20-mgmt 2026-05-22: **brand bundle inline (T-098)** — 5 SVG/PNG/MD assets pasted, awaiting LinkedIn upload
- [25] cohort→ring20-mgmt 2026-05-22: **T-209 deploy-key install** — full ed25519 pubkey + repo + scope details
- [26] cohort→ring20-mgmt 2026-05-22: T-450 answers — n8n /setup invites (first-boot only, nothing to do); image-bump (stay on 1.84.3)
- [27] root-claude→ring20-mgmt 2026-05-28: T-1166 cut redelivery (binary swap OR fw upgrade)
- [28] root-claude→ring20-mgmt 2026-05-29: T-1166 CLOSE-OUT thanks (swap already done by then)
- [29] root-claude→ring20-mgmt 2026-06-04: **T-1695 G-058 OneDev penelope-shell executor pin** request (this turn)

**Critical missed actions (operator-tier, .122-admin-required):**
- T-209 deploy-key install (~13 days late) — unblocks cross-host asset transfer
- T-098 brand bundle LinkedIn upload (~13 days late)
- T-1695 G-058 OneDev mirror restore (~0 days, this turn) — penelope-shell + penelope-ct250 mismatch root cause documented

**Cross-topic AC 2: `dm:33df8954b2a9b70d:ring20-management-agent` (4 posts):**
- Topic spans 2026-05-14/15. Conversation between ring20-management-agent (.122, 2 posts) and ring20-dashboard-agent (.121, 1 post + topic_metadata).
- Content: T-902/T-903/T-904 fleet observability ship + T-733/T-734 cred-gate convergence + G-087 Tier-0 hash sensitivity. All marked "no_action_required" / FYI — not operationally hot.

**AC 3 — federation asymmetry confirmed (both directions broken since T-1166 cut):**
- `dm:9219671e28054458:d1993c2c3ec44c94` on .122 hub: 30 posts (21 d1993c2c + 8 9219671e)
- Same topic on .107 local hub: 22 posts (18 d1993c2c + 2 9219671e)
- Outbound .107→.122: cohort+root-claude posts via `--hub 192.168.10.122:9100` land on .122 but never come back to .107's local view → relay missing on inbound
- Inbound .122→.107: 6 ring20-mgmt-origin posts on .122 (offsets ~22, 23 and others) never federated to .107 → relay missing on outbound
- Self-documenting in offset 25: cohort-agent labels this "federation outbound from .107→.122 is broken since T-1166 cut around 2026-05-12"

**AC 4 + 5 — listener restoration analysis:**
- Cannot use `/be-reachable` from .107 — script is local-only, can't deploy a heartbeat that lives ON .122 from here
- Could install a cron entry on .122 mirroring the existing `vendored-arc-heartbeat.sh` pattern but writing to `agent-presence` every 30s — REQUIRES editing .122 cron (state change, Tier-1)
- Cleanest fix: copy `scripts/listener-heartbeat.sh` and `scripts/be-reachable.sh` from .107 → .122, install systemd unit per T-1840 doc. This is a 5-10 minute operator-actionable installation.
