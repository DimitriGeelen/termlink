---
id: T-1834
name: "agent-send.sh --to <agent-id> auto-discover (T-1830 sub-build c) + heartbeat pty_session"
description: >
  agent-send.sh --to <agent-id> auto-discover (T-1830 sub-build c) + heartbeat pty_session

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-28T13:01:01Z
last_update: 2026-05-28T13:06:46Z
date_finished: 2026-05-28T13:06:46Z
---

# T-1834: agent-send.sh --to <agent-id> auto-discover (T-1830 sub-build c) + heartbeat pty_session

## Context

T-1830 GO sub-build (c) — wires the agent-send.sh send path to the T-1832 heartbeat + T-1833 discovery primitives. Today agent-send requires `--to-session NAME` + `--topic dm:...` or `--peer-fp <fp>` — meaning the sender must already know the peer's PTY-session name AND their dm-topic. With auto-discover, the sender says `--to <agent-id>` and the script resolves both via agent-listeners. Schema extension: heartbeat declares optional `pty_session` in metadata; discovery surfaces it; auto-discover consumes it.

This completes the T-1830 trio (a/b/c). After this, an operator on host X can run `bash scripts/agent-send.sh --to penelope --message "hi"` without needing to know penelope's session or topic — provided penelope is heartbeating.

## Acceptance Criteria

### Agent
- [x] `scripts/listener-heartbeat.sh` extended: new optional `--pty-session NAME` flag. When provided, posts `metadata.pty_session=NAME`. When omitted, the field is absent from the envelope (not empty string)
- [x] `scripts/test-listener-heartbeat.sh` extended: new test verifying `--pty-session foo --once` produces `metadata.pty_session=="foo"` in the posted envelope; existing test confirms omitting the flag means the field is absent
- [x] `scripts/agent-listeners.sh` extended: `pty_session` field surfaces in the JSON listener entry (omitted if absent in source envelope); not surfaced in text mode (table width is tight)
- [x] `scripts/test-agent-listeners.sh` extended: new test verifying pty_session round-trips through discovery
- [x] `scripts/agent-send.sh` extended: new `--to AGENT_ID` flag (mutually exclusive with `--to-session`+`--topic`/`--peer-fp`). When `--to` is given, the script invokes `agent-listeners.sh --filter-agent-id AGENT_ID --json` to resolve:
    - to_session ← metadata.pty_session (REQUIRED — error if missing/empty)
    - topic      ← first item in metadata.listen_topics that starts with `dm:`; if no dm:* item, error with hint
- [x] `scripts/agent-send.sh` adds `--dry-run` flag: when set with `--to`, resolves and prints `RESOLVED: to_session=... topic=... agent_id=... status=...` to stdout and exits 0 WITHOUT posting or injecting (test seam)
- [x] Auto-discover error paths exit 2 with specific messages: (a) agent-id not found in listeners → "no listener with agent_id=X", (b) listener present but status OFFLINE → "agent X is OFFLINE", (c) listener present but pty_session missing → "agent X heartbeat does not declare pty_session", (d) listener present but no dm:* listen_topic → "agent X has no dm:* listen_topic"
- [x] `scripts/test-agent-send-auto-discover.sh` exists: T1 --to + --to-session both given → exit 2 (mutex); T2 --to <unknown> → exit 2 with not-found message; T3 --to <agent> with pty_session and dm:* listen_topic + --dry-run → exit 0 with RESOLVED line; T4 --to <agent> with no pty_session → exit 2; T5 --to <agent> with no dm:* listen_topic → exit 2
- [x] All test suites pass: listener-heartbeat (7), agent-listeners (8), agent-send-auto-discover (5)
- [x] Live verification: spawn heartbeat with `--pty-session ANY --listen-topic dm:foo:bar --agent-id T-1834-soak`; run `bash scripts/agent-send.sh --to T-1834-soak --message hi --dry-run` and confirm RESOLVED line shows the expected to_session + topic

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
bash scripts/test-listener-heartbeat.sh
bash scripts/test-agent-listeners.sh
bash scripts/test-agent-send-auto-discover.sh
bash scripts/agent-send.sh --help >/dev/null

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

### 2026-05-28T13:01:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1834-agent-sendsh---to-agent-id-auto-discover.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-107f1783
- **Timestamp:** 2026-05-28T13:07:28Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 3

**Per-AC findings:**

- **AC#1 (Agent)** — `scripts/listener-heartbeat.sh` extended: new optional `--pty-session NAME` flag. When provided, posts `metadata.pty_session=NAME`. When omitted, the field is absent from the envelope (not empty strin
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/listener-heartbeat.sh in: `scripts/listener-heartbeat.sh` extended: new optional `--pty-session NAME` flag. When provided, posts `metadata.pty_session=NAME`. When omitted, the `
- **AC#3 (Agent)** — `scripts/agent-listeners.sh` extended: `pty_session` field surfaces in the JSON listener entry (omitted if absent in source envelope); not surfaced in text mode (table width is tight)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/agent-listeners.sh in: `scripts/agent-listeners.sh` extended: `pty_session` field surfaces in the JSON listener entry (omitted if absent in source envelope); not surfaced in`

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 12
     - evidence: `bash scripts/agent-send.sh --help >/dev/null`

### 2026-05-28T13:06:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
