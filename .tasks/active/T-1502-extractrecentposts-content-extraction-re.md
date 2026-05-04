---
id: T-1502
name: "extract_recent_posts content extraction returns empty for chat-arc envelopes"
description: >
  Live observation while smoke-testing T-1501 (--grep filter): all 88 posts in a 24h window of agent-chat-arc render with empty content via cmd_agent_timeline / recent / on-thread. The render shows '(empty)' for every post even though metadata (msg_type, thread, project, peer_fp) is correctly extracted. This means agent recent / on-thread / timeline / overview have been silently rendering empty for real chat-arc traffic since T-1492 — content cap, grep, and reading flows are all moot until this is fixed. Expected fix: inspect actual envelope shape on the wire (likely payload structure differs from what extract_recent_posts assumes — text under 'content' or similar nested field), update the payload.text → payload (string) → payload.to_string() fallback chain to match real shape. Add unit test against a captured real envelope. RCA: how did 4 verbs ship without this being caught? content=empty was not regression-tested against live arc; unit tests use synthetic envelopes.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T22:02:14Z
last_update: 2026-05-04T22:02:14Z
date_finished: 2026-05-04T22:25:00Z
---

# T-1502: extract_recent_posts content extraction returns empty for chat-arc envelopes

## Context

Live envelope inspection (offset 265 on agent-chat-arc) shows the wire shape:
```json
{"msg_type":"note","metadata":{"from_project":"010-termlink","thread":"T-1500"},"payload_b64":"VC0xNTAw...","sender_id":"d1993c2c3ec44c94","ts":1777931136391}
```

Two bugs in `extract_recent_posts` (channel.rs):
1. **Content extraction**: looks for `payload.text` / `payload as &str` / `payload.to_string()` — but real envelopes carry content under `payload_b64` (base64-encoded UTF-8). All 88 posts in a 24h window render with empty content.
2. **Thread metadata key mismatch**: looks for `metadata._thread` — but real envelopes use `metadata.thread`. Thread filter silently never matches in production.

Fix: prepend `payload_b64` decode path to the content fallback chain; accept both `metadata.thread` AND `metadata._thread` keys (defensive — keep existing tests green).

RCA: 4 verbs (T-1492 recent, T-1493 on-thread, T-1495 overview, T-1500 timeline) shipped with empty-content rendering against the real arc; unit tests use synthetic envelopes that match the assumed shape, not the actual wire shape. Surfaced only when smoke-testing T-1501 --grep returned 0 matches against known-existing content.

## Acceptance Criteria

### Agent
- [x] `extract_recent_posts` decodes `payload_b64` (base64) to UTF-8 BEFORE falling back to `payload.text` / `payload as &str` / `payload.to_string()` chain
- [x] Invalid base64 / invalid UTF-8 falls through to existing chain (defensive — never panic)
- [x] Thread metadata extraction tries `metadata.thread` AND `metadata._thread` (existing tests use `_thread`; real wire uses `thread`)
- [x] All existing extract_recent_posts unit tests still pass (additive — no breaking signature change)
- [x] New unit test: payload_b64 envelope renders correctly-decoded content
- [x] New unit test: payload_b64 with invalid base64 falls through to fallback (returns empty, no panic)
- [x] New unit test: metadata.thread (without underscore) is recognized
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `agent timeline --window-secs 86400 --n 5` shows non-empty content for at least one note post (not "(empty)")
- [x] Live smoke: `agent timeline --window-secs 86400 --grep T-1500` returns at least one matching post (T-1500 was posted at offset 265 with mention of T-1500)
- [x] Live smoke: `agent on-thread T-1500 --window-secs 86400` returns posts (was returning empty under metadata._thread bug)

### Human
- [ ] [REVIEW] Verify the fixed reading verbs surface real chat-arc content
  **Steps:**
  1. `target/release/termlink agent timeline --window-secs 86400 --n 10` (run from /opt/termlink)
  2. `target/release/termlink agent recent --target-fp d1993c2c3ec44c94 --window-secs 86400 --n 5`
  3. `target/release/termlink agent on-thread T-1500 --window-secs 86400`
  **Expected:** Notes show their actual decoded text content (not "(empty)"). Status posts may still show empty (their payload_b64 is empty/missing). Thread filter on T-1500 returns matching posts.
  **If not:** capture an envelope via `channel subscribe agent-chat-arc --cursor N --limit 1 --json` and report the wire shape — the fix may need another field-name variant.

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
cargo test --release --bin termlink commands::channel::tests::recent_posts 2>&1 | tail -3 | grep -qE "test result: ok"
out=$(target/release/termlink agent timeline --window-secs 86400 --n 5 --json 2>&1); echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); contents=[p['content'] for p in d['posts'] if p.get('content')]; assert len(contents) > 0, f'no posts have content (got {len(d[\"posts\"])} total)'; print('OK')" 2>&1 | grep -q OK
out=$(target/release/termlink agent timeline --window-secs 86400 --n 50 --grep T-1500 --json 2>&1); echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); assert len(d['posts']) > 0, 'expected at least 1 post matching T-1500'; print('OK')" 2>&1 | grep -q OK

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

## Updates

### 2026-05-04T22:02:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1502-extractrecentposts-content-extraction-re.md
- **Context:** Initial task creation

### 2026-05-04T22:25:00Z — status-update [manual]
- **Change:** status: started-work → work-completed (G-054 workaround: fw task update flock-deadlocked)
- **Owner:** agent → human (partial-complete; Human REVIEW AC pending)
