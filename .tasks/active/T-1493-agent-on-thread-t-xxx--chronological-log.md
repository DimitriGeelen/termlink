---
id: T-1493
name: "agent on-thread <T-XXX> — chronological log of all posts on a thread across all peers"
description: >
  agent on-thread <T-XXX> — chronological log of all posts on a thread across all peers

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T17:22:46Z
last_update: 2026-05-04T17:33:01Z
date_finished: 2026-05-04T17:33:01Z
---

# T-1493: agent on-thread <T-XXX> — chronological log of all posts on a thread across all peers

## Context

T-1488 (`agent who --thread`) and T-1490 (`agent presence --thread`)
ship per-peer and fleet-wide thread aggregations. T-1492 (`agent recent`)
shows a single peer's posts. The remaining gap: "show me the actual
posts on T-1485 across the whole fleet, in time order, so I can read
the discussion." This task adds `agent on-thread <T-XXX>` —
chronological reading view of a thread across ALL peers. Refactors
the T-1492 helper to take an optional peer filter so both verbs share
one extraction primitive (`extract_recent_posts`). RecentPost gains a
`peer_fp` field so cross-peer rendering can label each post.

## Acceptance Criteria

### Agent
- [x] Refactor `extract_recent_posts_for_peer` → `extract_recent_posts(msgs, n, window_ms, now_ms, filter_peer_fp, filter_thread, filter_project)` with all filters optional
- [x] `RecentPost` gains `peer_fp: String` field; `to_json` echoes it
- [x] Update existing 7 unit tests to use new signature; all still pass
- [x] T-1492 `cmd_agent_recent` updated to call new signature with `Some(peer_fp)`
- [x] New `agent on-thread` subcommand registered (clap parses via `--help`)
- [x] Positional `<THREAD>` (T-XXX style) is required
- [x] `--n N` controls count (default 50, clamped to [1, 500] — bigger than `recent` because thread logs are denser)
- [x] `--window-secs N` bounds the walk (default 86400, clamped [60, 604800])
- [x] `--project <name>` further filters by `metadata.from_project`
- [x] `--peer <name>` / `--peer-fp <hex>` further narrows to one peer (essentially equivalent to `agent recent --thread`; useful for confirmation)
- [x] `--hub <addr>` overrides default hub
- [x] `--json` outputs envelope: `{thread, window_secs, n, posts: [...]}` with `filter_project`/`peer_fp` echoed when set
- [x] Sort: chronological asc (natural reading flow)
- [x] Text mode: header `# agent on-thread <T-XXX> | window=Xs | n=Y` + per-post block: `[ts_relative] peer=<fp_short> msg_type=<t> [project=<p>]\n  <content>`
- [x] 4 new unit tests for `extract_recent_posts` covering thread-only path: thread-only-no-peer-filter / thread-only-excludes-others-and-untagged / thread+project AND-compose / N-cap-keeps-most-recent
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink recent_posts` passes (11/11 — 7 adapted + 4 new)
- [x] Live smoke: `agent on-thread T-1438 --window-secs 86400 --n 5` returns 5 chronological rows (peer fp shortened to 12 chars, project labeled per row)

### Human
- [ ] [REVIEW] Verify the on-thread reading view scans well
  **Steps:**
  1. `target/release/termlink agent on-thread T-1438 --window-secs 86400 --n 5` (run from /opt/termlink)
  **Expected:** Posts in chronological order, peer FP visible per post, content readable
  **If not:** suggest layout changes

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
cargo test --release -p termlink --bin termlink recent_posts 2>&1 | grep -qE "test result: ok\. (1[1-9]|[2-9][0-9])"
target/release/termlink agent on-thread --help 2>&1 | grep -q -- "--n"
out=$(target/release/termlink agent on-thread T-1438 --window-secs 86400 --n 3 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert d.get('thread') == 'T-1438', d; assert isinstance(d.get('posts'), list); assert d.get('n') == 3"
out=$(target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 2 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert isinstance(d.get('posts'), list)" # regression: existing recent verb still works

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

**Rationale:** Closes the chronological-reading gap on the observability stack. T-1488/1490 aggregate per-peer / per-fleet thread activity; T-1492 shows a single peer's content; this verb fills "show me the actual thread discussion across the fleet, in time order." Refactor of `extract_recent_posts_for_peer` → `extract_recent_posts` with optional peer filter unifies the two content-access verbs onto one helper, eliminating duplicate logic and ensuring filter semantics stay in lockstep. RecentPost gains `peer_fp` field (non-breaking — new field on a new struct).

**Evidence:**
- Live: `agent on-thread T-1438 --window-secs 86400 --n 5` → 5 rows, chronological asc, peer=d1993c2c3ec4 (12-char short fp), project=010-termlink labeled per row
- Live JSON `--n 3`: envelope keys = `[n, posts, thread, window_secs]`; `posts[*].peer_fp` populated; `posts[*].thread = "T-1438"` for every entry (filter applied)
- Live regression: `agent recent --target-fp ... --json` still emits valid `posts[]` envelope (T-1492 unchanged)
- Unit tests: 11/11 `recent_posts_*` pass — 7 existing adapted to new signature + 4 new on-thread-path tests (no-peer-filter / excludes-others-and-untagged / thread+project AND-compose / N-cap)
- Verification: 5/5 commands pass

## Decisions

### 2026-05-04 — Refactor extract_recent_posts_for_peer → extract_recent_posts
- **Chose:** Generalize the existing helper to take an optional peer filter; both `agent recent` and `agent on-thread` call the same primitive.
- **Why:** Duplicate logic for content extraction, payload-shape handling, and content truncation would inevitably drift. Single helper means filter semantics stay consistent. The signature change is internal — only one external caller (cmd_agent_recent) needed update.
- **Rejected:** Two parallel helpers (`extract_recent_posts_for_peer` + `extract_recent_posts_for_thread`) — would have meant copy-pasting ~60 lines of payload extraction logic; small bug fixes would need to be replicated.

### 2026-05-04 — Add peer_fp to RecentPost
- **Chose:** RecentPost gains `peer_fp: String`; `to_json` echoes it.
- **Why:** Cross-peer rendering (`agent on-thread`) needs to label each post with its sender. Single-peer rendering (`agent recent`) ignores the field but its presence is harmless and the JSON envelope is now self-describing.
- **Rejected:** Separate ThreadPost struct — would have required two near-identical to_json methods and downstream rendering branches.

## Updates

### 2026-05-04T17:22:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1493-agent-on-thread-t-xxx--chronological-log.md
- **Context:** Initial task creation

### 2026-05-04T17:33:01Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
