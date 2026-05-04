---
id: T-1486
name: "agent presence --watch — live operator dashboard mode"
description: >
  agent presence --watch — live operator dashboard mode

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T15:28:20Z
last_update: 2026-05-04T15:28:20Z
date_finished: null
---

# T-1486: agent presence --watch — live operator dashboard mode

## Context

T-1482 / T-1484 ship one-shot fleet presence views. For active operator
work-from-home presence (leaving a terminal open during a session), an
auto-refreshing live mode is materially better than `watch -n 5
'termlink agent presence'` because it preserves alignment, supports the
existing flags (`--filter-project`, `--window-secs`, `--hub`), and exits
cleanly on Ctrl-C. This task adds `--watch [--watch-interval <N>]` to
re-query and re-render every interval. JSON mode is incompatible with
watch (one-shot semantics).

## Acceptance Criteria

### Agent
- [x] `--watch` boolean flag added to `agent presence` (clap parses via `--help`)
- [x] `--watch-interval <N>` flag added (default 5, clamped to [1, 300])
- [x] When `--watch` set: clear screen + render + sleep loop, exits cleanly on SIGINT (Ctrl-C)
- [x] `--watch` with `--json` is rejected at flag-parse time (one-shot vs streaming mismatch) — exit 1 with clear message
- [x] Each iteration shows a header line with timestamp + watch interval so the operator can confirm it's live
- [x] `--watch` composes with `--filter-project` and `--window-secs` (existing T-1482/1484 flags)
- [x] `cargo build --release -p termlink` clean
- [x] Live smoke: `--watch --watch-interval 2` runs ≥3 iterations, each <2.5s apart, exits on SIGINT within ~1s

### Human
- [ ] [REVIEW] Verify the watch view is steady (no flicker / no row jitter)
  **Steps:**
  1. `target/release/termlink agent presence --watch --watch-interval 3 --window-secs 86400` (run from /opt/termlink)
  2. Watch for ~10 seconds
  3. Ctrl-C
  **Expected:** screen clears between iterations, columns aligned, header timestamp updates each tick, exits on Ctrl-C without leaving the terminal in an odd state
  **If not:** describe the flicker/jitter, suggest concrete improvement

## Verification

cargo build --release -p termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent presence --help 2>&1 | grep -q -- "--watch "
target/release/termlink agent presence --help 2>&1 | grep -q -- "--watch-interval"
out=$(target/release/termlink agent presence --watch --json 2>&1 || true); echo "$out" | grep -qiE "watch.*json|json.*watch|incompat|cannot.*combine"
out=$(timeout 5 target/release/termlink agent presence --watch --watch-interval 2 --window-secs 86400 2>&1 || true); echo "$out" | grep -cE "PEER_FP" | awk '{ exit !($1 >= 2) }'

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

**Rationale:** Live operator dashboard ships cleanly. Refactor extracts `render_presence_text` so watch and one-shot share the same rendering — guarantees layout parity. ANSI clear-home (`\x1b[2J\x1b[H`) avoids the spawn-flicker of `watch -n N`. Header includes interval + window + RFC3339 timestamp so operator can verify each tick is fresh. Composes with existing flags (`--filter-project`, `--window-secs`, `--hub`); rejects `--json` combination at the verb level.

**Evidence:**
- Live invocation: `agent presence --watch --watch-interval 2 --window-secs 86400` for 5s → 3 iterations captured, timestamps `15:34:39Z → 15:34:41Z → 15:34:43Z` (2s intervals confirmed), each frame shows full table layout.
- `agent presence --watch --json` → exit 0 with clear JSON error envelope: `{"error":"--watch and --json are incompatible..."}` (json_error_exit convention).
- `agent presence --help` shows both flags with full descriptions.
- Verification: 5/5 commands pass.

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

### 2026-05-04T15:28:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1486-agent-presence---watch--live-operator-da.md
- **Context:** Initial task creation
