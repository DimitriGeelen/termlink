---
id: T-1495
name: "agent overview — single-shot fleet digest combining presence + by-project + recent posts"
description: >
  agent overview — single-shot fleet digest combining presence + by-project + recent posts

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T17:46:11Z
last_update: 2026-05-04T17:52:36Z
date_finished: 2026-05-04T17:52:36Z
---

# T-1495: agent overview — single-shot fleet digest combining presence + by-project + recent posts

## Context

T-1481-1494 ship the fleet observability stack: `who`, `presence`,
`presence --by-project`, `recent`, `on-thread`, plus `--filter-project`,
`--thread`, `--watch`, `--top` flags. Together they answer specific
questions but require the operator to know which verb to invoke. This
task adds `agent overview` — a single-shot digest that fetches
chat-arc ONCE and renders three compact summaries:
top 5 peers (by posts), top 5 projects, last 5 posts (across fleet).
Useful as the first command of a session: "what's the fleet doing
right now?" without needing to chain three queries. Pure composition
verb — reuses existing pure helpers (`summarize_fleet_presence`,
`summarize_fleet_by_project`, `extract_recent_posts`).

## Acceptance Criteria

### Agent
- [x] New `agent overview` subcommand registered (clap parses via `--help`)
- [x] `--window-secs N` flag (default 3600 / 1h, clamped [60, 604800])
- [x] `--top N` flag (default 5, clamped [1, 50]) — applies to all 3 sections symmetrically
- [x] `--hub <addr>` overrides default hub
- [x] `--json` outputs envelope: `{window_secs, top, peers: [...], projects: [...], recent_posts: [...], total_peers, total_projects}` — three arrays each capped at `top` rows + total counts so caller can disambiguate "exactly N" vs "N truncated"
- [x] Single chat-arc fetch (one RPC round-trip); all three summaries computed from the same `msgs` slice
- [x] Text mode: 3 sections separated by blank lines, each with a `## Header` heading; ends with single footer line `# overview: window=Xs, top=N, total_peers=X, total_projects=Y`
- [x] Section 1 (Top Peers): PEER_FP / LAST_SEEN / POSTS / TOP_PROJECT (re-uses FleetPeerRow output)
- [x] Section 2 (Top Projects): PROJECT / POSTS / PEERS / TOP_PEER / LAST_SEEN (re-uses FleetProjectRow output)
- [x] Section 3 (Recent Posts): chronological asc, with peer_short / msg_type / thread / project labels per post + first-line content preview (capped at 100 chars + ellipsis if multi-line or truncated)
- [x] Empty fleet: "(no fleet activity in window)" — single line, exit 0
- [x] `cargo build --release -p termlink` clean
- [x] No new unit tests required (composition-only verb; underlying helpers covered by 15+6+11=32 existing tests)
- [x] Live smoke: `agent overview --window-secs 86400 --top 3` renders 3 sections (1 peer, 3 projects, 3 recent posts); `--json --top 2` returns valid envelope with peers/projects/recent_posts arrays + total counts

### Human
- [ ] [REVIEW] Verify the overview is operator-readable as a "first command" of a session
  **Steps:**
  1. `target/release/termlink agent overview --window-secs 86400` (run from /opt/termlink)
  **Expected:** Three clearly delimited sections; can scan in <5 seconds to know fleet state
  **If not:** suggest section ordering or layout changes

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

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent overview --help 2>&1 | grep -q -- "--top"
target/release/termlink agent overview --help 2>&1 | grep -q -- "--window-secs"
out=$(target/release/termlink agent overview --window-secs 86400 --top 3 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert isinstance(d.get('peers'), list); assert isinstance(d.get('projects'), list); assert isinstance(d.get('recent_posts'), list); assert d.get('top') == 3; assert d.get('window_secs') == 86400"
target/release/termlink agent overview --window-secs 86400 2>&1 | grep -qE "(Top Peers|## Top|peers active|projects active|posts? shown)"

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

## Recommendation

**Recommendation:** GO

**Rationale:** Pure composition verb — wraps three already-shipped pure helpers (`summarize_fleet_presence`, `summarize_fleet_by_project`, `extract_recent_posts`) on a single chat-arc fetch into one operator-friendly digest. Designed as a session's first command: tells you "fleet state right now" without chaining three queries. JSON envelope includes `total_peers` / `total_projects` so callers know what was clipped by `--top`. Empty-fleet single-line output keeps signal-to-noise high on quiet hubs.

**Evidence:**
- Live: `agent overview --window-secs 86400 --top 3` → 3 sections with `## Top Peers` / `## Top Projects` / `## Recent Posts` headings; footer summarizes window + top + total counts
- Live data: 1 peer (d1993c2c3ec44c94, 76 posts), 3 projects (010-termlink dominant at 33 posts), 3 recent posts (msg_types: star/status/status with empty content — fleet is heartbeat-only)
- Live JSON: envelope keys = `[peers, projects, recent_posts, top, total_peers, total_projects, window_secs]`; arrays correctly capped at top=2 (peers=1 since only 1 peer; projects=2; recent_posts=2)
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Single fetch + three views vs three separate fetches
- **Chose:** One `fetch_recent_chat_arc_msgs(2000)` call, three pure helpers compute their summaries from the same msgs slice.
- **Why:** Three RPC round-trips would triple latency and chat-arc network load for no benefit — all three summaries derive from the same data. Pure helpers don't share state, just msgs.
- **Rejected:** Three separate fetches via existing `fetch_*_via_chat_arc` wrappers — simpler call sites but worse latency and load.

### 2026-05-04 — Symmetric --top vs per-section flags
- **Chose:** Single `--top N` controls all three sections.
- **Why:** Operator wants "give me a digest" — calibrating per-section depth is overkill for a "first command of session" verb. If they want more depth, they reach for the specific verb (`agent presence`, `agent on-thread`).
- **Rejected:** `--top-peers N --top-projects M --top-posts P` — three flags, ambiguous default behavior, no clear win.

### 2026-05-04 — Content preview capped at 100 chars in overview vs 200 in `recent`/`on-thread`
- **Chose:** Stricter truncation in overview (100-char first-line preview, ellipsis if multi-line OR over 100 chars).
- **Why:** Overview is a digest — operator drills in via the specific verb for full content. 100 chars is a single-line scan limit.
- **Rejected:** Same 200-char cap as `recent` — wastes vertical space on a digest where each post should be 1-2 lines.

## Updates

### 2026-05-04T17:52:36Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)

### 2026-05-04T17:46:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1495-agent-overview--single-shot-fleet-digest.md
- **Context:** Initial task creation
