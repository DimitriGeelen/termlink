---
id: T-1492
name: "agent recent <peer> — show last N posts from a peer on chat-arc"
description: >
  agent recent <peer> — show last N posts from a peer on chat-arc

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T17:11:54Z
last_update: 2026-05-04T17:21:46Z
date_finished: 2026-05-04T17:21:46Z
---

# T-1492: agent recent <peer> — show last N posts from a peer on chat-arc

## Context

T-1481/1483/1487/1490 ship aggregation-style observability (presence,
who, ping). Operator can see WHO is active and WHAT projects/threads
they're working on, but not WHAT they're saying. This task adds
`agent recent <peer>` — a content-access verb that walks `agent-chat-arc`
filtered to a single peer and prints the last N posts with timestamp,
optional thread/project tags, and content snippet. Uses existing
primitives: `fetch_recent_chat_arc_msgs` (T-1481) for the walk,
`resolve_target_name_to_fp` (T-1483) for `<peer>` name resolution.
Closes the "I see alice posted on T-1485 — what did she actually say?"
gap without operator falling back to `events --topic agent-chat-arc | jq`.

## Acceptance Criteria

### Agent
- [x] New `agent recent` subcommand registered (clap parses via `--help`)
- [x] Positional `<TARGET>` (display name resolved locally) OR `--target-fp <hex>` (cross-host disambiguation); mutually exclusive; one required
- [x] `--n N` flag controls count (default 10, clamped to [1, 200])
- [x] `--window-secs N` flag bounds the walk (default 86400 / 1 day, clamped to [60, 604800])
- [x] `--thread <T-XXX>` filters by `metadata._thread`
- [x] `--project <name>` filters by `metadata.from_project`
- [x] `--hub <addr>` overrides default hub
- [x] `--json` outputs structured envelope: `{target, peer_fp, window_secs, n, posts: [...]}` with `filter_thread`/`filter_project` echoed when set
- [x] Pure helper `extract_recent_posts_for_peer(msgs, peer_fp, n, window_ms, now_ms, filter_thread, filter_project) -> Vec<RecentPost>`; testable
- [x] RecentPost has: ts_ms, msg_type, content (string, possibly truncated), thread, project
- [x] Sort: chronological asc (oldest at top — natural reading order)
- [x] Text mode: each post on its own block: `[ts_relative] msg_type=<type> [thread=<T>] [project=<p>]\n  <content trimmed to ≤200 chars>` ((empty) annotation when content is empty)
- [x] Filters out meta msg_types (reaction/edit/redaction/topic_metadata/receipt)
- [x] 7 unit tests: empty / target-filter / N-cap-keeps-most-recent / thread-filter / project-filter / meta-skipped+outside-window / content-truncation
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink --bin termlink recent_posts` passes (7/7)
- [x] Live smoke: `agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 3` returns 3 chronologically-ordered posts (msg_types: chat/status/star) — content empty because chat-arc on this fleet is heartbeat-only, but verb correctly extracts and orders

### Human
- [ ] [REVIEW] Verify the recent-post output is operator-readable
  **Steps:**
  1. `target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 5` (run from /opt/termlink)
  **Expected:** Output is scannable — each post clearly delimited, timestamp + content visible at a glance
  **If not:** suggest format changes (block separator, indent depth, content truncation length)

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
cargo test --release -p termlink --bin termlink recent_posts 2>&1 | grep -qE "test result: ok\. ([4-9]|[1-9][0-9])"
target/release/termlink agent recent --help 2>&1 | grep -q -- "--target-fp"
target/release/termlink agent recent --help 2>&1 | grep -q -- "--n "
out=$(target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 3 --json 2>&1); echo "$out" | python3 -c "import sys, json; d = json.load(sys.stdin); assert isinstance(d.get('posts'), list); assert d.get('n') == 3, d; assert d.get('peer_fp') == 'd1993c2c3ec44c94'"

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

**Rationale:** Closes the content-access gap on the observability stack. T-1481/1483/1487/1490/1491 tell you who's active and what they're working on; this verb shows what they've actually said. Pure helper extension — `extract_recent_posts_for_peer` is testable in isolation. Composes with thread/project filters. Backward-compat: brand-new verb, no existing surface to regress.

**Evidence:**
- Live: `agent recent --target-fp d1993c2c3ec44c94 --window-secs 3600 --n 3` → 3 posts in chronological asc (chat/status/star msg_types from heartbeats)
- Live JSON `--n 2`: envelope keys = `[n, peer_fp, posts, target, window_secs]`; `posts` is sorted-by-ts-asc array; `n=2` clamped value echoed back
- Unit tests: 7/7 `recent_posts_*` pass — covers empty / target-filter / N-cap / thread-filter / project-filter / meta-skipped / content-truncation
- Verification: 5/5 commands pass

**Note on live-content coverage:** This fleet's chat-arc is heartbeat-only (msg_type=chat with empty payload.text), so live posts show `(empty)` annotation. Verb correctly extracts content from `payload.text` when present (unit test `recent_posts_truncates_long_content_with_ellipsis` verifies); fleets with actual chat traffic will see real text. The "(empty)" annotation explicitly signals "no content" rather than dropping the post — operator can still see msg_type/thread/project metadata which IS populated on heartbeats.

## Decisions

### 2026-05-04 — Chronological ASC (oldest first) instead of DESC
- **Chose:** Sort posts oldest-first, newest-last.
- **Why:** Natural reading flow — operator pages down to see how a conversation evolved. Mirrors how `git log --reverse` and chat clients display threads.
- **Rejected:** Desc (newest first) — feels right for "give me the latest" but breaks the conversation-flow read; would need separate flag.

### 2026-05-04 — Content cap at 200 chars + ellipsis
- **Chose:** Truncate content to 200 chars in the helper, suffix `…`.
- **Why:** `agent recent` is a quick-glance verb, not a full-message reader. Keeps tables compact. JSON envelope honors the same cap so callers can't accidentally pull megabyte payloads.
- **Rejected:** No cap — could blow up output for long posts; operator can always fall back to `events --topic agent-chat-arc --since N | jq` for full content.

## Updates

### 2026-05-04T17:11:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1492-agent-recent-peer--show-last-n-posts-fro.md
- **Context:** Initial task creation

### 2026-05-04T17:21:46Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
