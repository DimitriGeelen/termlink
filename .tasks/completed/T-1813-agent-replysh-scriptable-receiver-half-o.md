---
id: T-1813
name: "agent-reply.sh scriptable receiver half of doorbell+mail loop (ack + optional reply)"
description: >
  agent-reply.sh scriptable receiver half of doorbell+mail loop (ack + optional reply)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-25T21:49:25Z
last_update: 2026-05-25T21:53:01Z
date_finished: 2026-05-25T21:53:01Z
---

# T-1813: agent-reply.sh scriptable receiver half of doorbell+mail loop (ack + optional reply)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
> **Outcome: REDUNDANT-ON-DISCOVERY.** The scriptable receiver half already
> exists — `scripts/agent-respond.sh` (T-1805) does receipt-ack + optional
> `--reply`, and `scripts/test-agent-respond.sh` already pairs the real
> agent-send.sh with the real agent-respond.sh end-to-end. No new script was
> warranted. I caught this only after building a duplicate; ACs rewritten to
> the real (cleanup) deliverable.
- [x] Verified `scripts/agent-respond.sh` (T-1805) already provides the receiver half: receipt ack (auto `up_to` = highest offset on the cid) + optional `--reply` turn, with `--topic`/`--peer-fp` resolution
- [x] Verified `scripts/test-agent-respond.sh` already covers the end-to-end round-trip with the REAL agent-send.sh (Path A receipt delivery, Path B no-responder, Path V arg validation)
- [x] Removed the redundant `scripts/agent-reply.sh` + `scripts/test-agent-reply.sh` I started before discovering T-1805 — no duplicate left in the tree
- [x] Confirmed the existing receiver test is green after cleanup (`test-agent-respond: ALL PASS`)

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
#
# The redundant agent-reply.sh was removed; the real receiver half is
# agent-respond.sh (T-1805). Prove it + the round-trip still work, and that no
# duplicate remains.
bash scripts/test-agent-respond.sh
test ! -f scripts/agent-reply.sh

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

### 2026-05-25 — delete the duplicate instead of shipping a second receiver
- **Chose:** Remove `agent-reply.sh`/`test-agent-reply.sh`; keep the existing `agent-respond.sh` (T-1805) as the one receiver verb.
- **Why:** Two scripts doing the same receipt-ack + optional-reply, composing the same primitives, would split the surface and confuse callers (and the recipe doc). One verb, one test. The marginal extras my version had (`--json`, exit-3-on-no-turn) don't justify a parallel script or churn on a tested, shipped one.
- **Rejected:** (a) Merge `--json` into agent-respond.sh — scope creep on a working script, not asked for. (b) Add a `--await-reply`×`--reply` e2e path to test-agent-respond.sh — marginal: `--await-reply` is already proven in test-agent-send.sh Path D, and agent-respond's reply post is wire-identical to that inline post.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-25T21:49:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1813-agent-replysh-scriptable-receiver-half-o.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-35f1d176
- **Timestamp:** 2026-05-25T21:53:07Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#1 (Agent)** — Verified `scripts/agent-respond.sh` (T-1805) already provides the receiver half: receipt ack (auto `up_to` = highest offset on the cid) + optional `--reply` turn, with `--topic`/`--peer-fp` resolution
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/agent-respond.sh in: Verified `scripts/agent-respond.sh` (T-1805) already provides the receiver half: receipt ack (auto `up_to` = highest offset on the cid) + optional `--`
- **AC#3 (Agent)** — Removed the redundant `scripts/agent-reply.sh` + `scripts/test-agent-reply.sh` I started before discovering T-1805 — no duplicate left in the tree
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/test-agent-reply.sh in: Removed the redundant `scripts/agent-reply.sh` + `scripts/test-agent-reply.sh` I started before discovering T-1805 — no duplicate left in the tree`

### 2026-05-25T21:53:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
