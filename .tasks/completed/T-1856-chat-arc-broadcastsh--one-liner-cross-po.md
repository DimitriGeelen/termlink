---
id: T-1856
name: "chat-arc-broadcast.sh — one-liner cross-post helper (G-060 mitigation)"
description: >
  chat-arc-broadcast.sh — one-liner cross-post helper (G-060 mitigation)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T21:52:20Z
last_update: 2026-05-28T21:54:50Z
date_finished: 2026-05-28T21:54:50Z
---

# T-1856: chat-arc-broadcast.sh — one-liner cross-post helper (G-060 mitigation)

## Context

G-060 (`docs/operations/channel-topic-semantics.md`) documents that
agent-chat-arc does NOT federate — cross-hub broadcast requires explicit
`channel post --hub <addr>` for every hub. Today this means operators
write a bash loop by hand every time, with timeout-wrap (PL-189) and
sender-attribution metadata (PL-191) easy to forget. Friction against
the directive "no active doorbell+mail conversations arc."

This task ships `scripts/chat-arc-broadcast.sh` — wraps hubs.toml
enumeration + per-hub `channel post --hub` + per-hub timeout +
automatic metadata.agent_id injection (from `--from`, $TERMLINK_AGENT_ID,
or `~/.termlink/be-reachable.state` in priority order). One-liner
replaces ~10 lines of operator boilerplate.

## Acceptance Criteria

### Agent
- [x] `scripts/chat-arc-broadcast.sh` created, chmod +x, with `--payload`, `--from`, `--hubs-file`, `--timeout-secs`, `--json` flags
- [x] Walks every profile in `~/.termlink/hubs.toml` (default), posts to each unique address with `--msg-type chat --metadata agent_id=<from> --metadata _from=<from>` and `--ensure-topic`
- [x] Per-hub call wrapped with `timeout 8` (PL-189 invariant — same as other discovery-triangle verbs)
- [x] Sender resolution priority: `--from` flag → `$TERMLINK_AGENT_ID` env → `jq -r .agent_id ~/.termlink/be-reachable.state` → exit 2 with hint
- [x] `--help` prints usage; `--json` emits one envelope `{ok, hubs_attempted, hubs_delivered, hubs_failed, sender, results:[{hub, ok, offset, error}]}`
- [x] `scripts/test-chat-arc-broadcast.sh` smoke tests: help, missing-payload, sender-resolution chain, local-only delivery — exits 0
- [x] `docs/operations/channel-topic-semantics.md` references the new helper as the canonical operator path

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
test -x scripts/chat-arc-broadcast.sh
bash scripts/chat-arc-broadcast.sh --help >/dev/null
test -x scripts/test-chat-arc-broadcast.sh
bash scripts/test-chat-arc-broadcast.sh
grep -q "chat-arc-broadcast.sh" docs/operations/channel-topic-semantics.md

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

### 2026-05-28T21:52:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1856-chat-arc-broadcastsh--one-liner-cross-po.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7621a154
- **Timestamp:** 2026-05-28T21:54:52Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 3

**Per-AC findings:**

- **AC#2 (Agent)** — Walks every profile in `~/.termlink/hubs.toml` (default), posts to each unique address with `--msg-type chat --metadata agent_id=<from> --metadata _from=<from>` and `--ensure-topic`
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/hubs.toml in: Walks every profile in `~/.termlink/hubs.toml` (default), posts to each unique address with `--msg-type chat --metadata agent_id=<from> --metadata _fr`
- **AC#4 (Agent)** — Sender resolution priority: `--from` flag → `$TERMLINK_AGENT_ID` env → `jq -r .agent_id ~/.termlink/be-reachable.state` → exit 2 with hint
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/be-reachable.state in: Sender resolution priority: `--from` flag → `$TERMLINK_AGENT_ID` env → `jq -r .agent_id ~/.termlink/be-reachable.state` → exit 2 with hint`

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 10
     - evidence: `bash scripts/chat-arc-broadcast.sh --help >/dev/null`

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-05-28T21:54:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
