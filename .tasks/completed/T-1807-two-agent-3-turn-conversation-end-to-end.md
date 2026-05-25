---
id: T-1807
name: "Two-agent >=3-turn conversation end-to-end validation (T-1800 S-5/build #4)"
description: >
  T-1800 build #4 / spike S-5. Validate the full runtime loop live: two real persistent claude listeners (per build #3 recipe) hold a >=3-turn structured conversation using agent-send.sh (T-1804) + the pickup-respond ritual (build #2), with heartbeats. Confirms determinism (every turn acked) and A-4 (content via channel.* envelopes, PTY never scraped). Needs live infra; lower priority until #2/#3 land.

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T17:35:16Z
last_update: 2026-05-25T20:17:39Z
date_finished: 2026-05-25T20:17:39Z
---

# T-1807: Two-agent >=3-turn conversation end-to-end validation (T-1800 S-5/build #4)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
<!-- Scope evolved during materialization (see Evolution): the live two-real-claude
     soak hit two reproduced blockers (G-a, G-b) and is split to T-1810 (blocked on
     T-1809). This task now certifies the deterministic multi-turn transport+ritual
     validation, which confirms 2 of the 3 stated properties (determinism, A-4). -->
- [x] A ≥3-turn conversation is conducted end-to-end over the doorbell+mail loop on a single `conversation_id`: each round `agent-send.sh` (post turn + doorbell + receipt wait) pairs with `agent-respond.sh` (receipt + reply). All three turns reported DELIVERED with FRESH per-turn receipts (offsets 1→4→7, not stale). Transcript captured.
- [x] **Determinism confirmed** — every sender turn (offsets 0/3/6) has a matching receipt whose `up_to` (0/3/6) equals that turn's offset; no silent drop, PL-011 satisfied per turn (incorporates the T-1808 offset-aware fix).
- [x] **A-4 confirmed** — delivery detected purely from `channel.*` receipt envelopes; the PTY was never scraped (the doorbell `inject` targeted a non-existent session, non-fatal, and contributed nothing to detection).
- [x] The full transcript (6 turns + 3 receipts on one conversation_id) is captured via `channel subscribe --json` and saved under `docs/reports/T-1807-doorbell-mail-loop-validation.md`.
- [x] Live two-real-claude blockers reproduced and documented (G-a: `--dangerously-skip-permissions` refused under root → allowlist required; G-b: `/check-arc` doorbell doesn't signal respond mode); recipe updated; follow-ups filed (T-1809 respond-mode signal, T-1810 live soak).

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
test -f docs/reports/T-1807-doorbell-mail-loop-validation.md
bash scripts/test-agent-respond.sh
bash scripts/test-agent-send.sh

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

### 2026-05-25 — live-claude soak split from deterministic transport validation
- **What changed:** Validating the loop with two REAL claude listeners hit two
  reproduced blockers: (G-a) `claude --dangerously-skip-permissions` is refused
  under root, so a hands-free listener needs `Bash(termlink:*)` allowlisted
  instead; (G-b) the `/check-arc` doorbell text doesn't tell a woken listener to
  enter respond mode vs read-only browse, so a live claude would read but never
  ack. Separately, building the multi-turn driver exposed the T-1808 bug
  (`agent-send.sh` matched any cid receipt, not one acking the current turn).
- **Plan impact:** The original "spawn governed claude listener" AC is not
  achievable hands-free on this root host without G-a/G-b fixes. Pivoted this
  task to certify the DETERMINISTIC multi-turn transport+ritual validation —
  which still confirms 2 of the 3 stated properties (determinism, A-4) — and
  split the live two-real-claude soak out.
- **Triggered:** Fixed T-1808 (offset-aware receipt, shipped). Filed T-1809
  (respond-mode doorbell signal) and T-1810 (live two-real-claude soak, blocked
  on T-1809). Updated `docs/operations/injectable-listener-spawn-recipe.md` with
  both constraints. Evidence in `docs/reports/T-1807-doorbell-mail-loop-validation.md`.

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

### 2026-05-25T17:35:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1807-two-agent-3-turn-conversation-end-to-end.md
- **Context:** Initial task creation

### 2026-05-25T20:06:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

## Reviewer Verdict (v1.4)

- **Scan ID:** R-c67923be
- **Timestamp:** 2026-05-25T20:17:45Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-25T20:17:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
