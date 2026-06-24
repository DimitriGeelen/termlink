---
id: T-1814
name: "Remove event.broadcast fallback from framework bus bridges (T-1166 cut-unblocker)"
description: >
  Remove event.broadcast fallback from framework bus bridges (T-1166 cut-unblocker)

status: work-completed
workflow_type: refactor
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-25T22:03:32Z
last_update: 2026-05-25T22:08:42Z
date_finished: 2026-05-25T22:08:42Z
---

# T-1814: Remove event.broadcast fallback from framework bus bridges (T-1166 cut-unblocker)

## Context

T-1166's cut-readiness gate ("zero attributable legacy calls in the 7-day
window") has lingered "almost ready" for weeks. Root cause found 2026-05-26:
the framework's two bus bridges — `lib/pickup-channel-bridge.sh` (posts
`framework:pickup`) and `lib/publish-learning-to-bus.sh` (posts
`channel:learnings`) — both fall back to `termlink event broadcast` when their
`channel.post` attempt fails. On `.122` (ring20-management) channel.post fails
(old binary lacking `--ensure-topic` and/or the topic missing after a hub
restart), so every pickup there emits a legacy `event.broadcast` to the .107
hub. Audit proof: `event.broadcast` calls from peer 192.168.10.122 are tagged
`topic=framework:pickup` (last 2026-05-22T11:46Z) — and the bridge fallback is
the ONLY framework code that posts event.broadcast to that topic. Because .122
pickups happen every few days, each one resets the 7-day clean window before it
can elapse. Removing the fallback (it points at a primitive being retired by
T-1166) breaks that cycle. The bridges are non-fatal "pure enhancement" code
(T-1214), so a channel.post failure correctly degrades to a logged no-op.
The `--ensure-topic` path (T-1443+) already covers the topic-loss case the
fallback was protecting against. Must land upstream (channel-1) — vendored
patches are clobbered by `fw upgrade` (PL-022).

## Acceptance Criteria

### Agent
- [x] `event.broadcast` fallback removed from `lib/pickup-channel-bridge.sh`: a
  channel.post failure logs a skip and `exit 0` with NO `termlink event
  broadcast` invocation. `bash -n` clean; channel.post path (incl
  `--ensure-topic` probe) retained. **Verified:** `grep -c 'termlink event
  broadcast'` = 0; golden-path functional test posted via=channel.post to the
  live .107 hub framework:pickup topic.
- [x] `event.broadcast` fallback removed from `lib/publish-learning-to-bus.sh`:
  same treatment, same retention of the channel.post path. **Verified:**
  `grep -c 'termlink event broadcast'` = 0; `bash -n` clean.
- [x] Both edits land upstream on `/opt/999-Agentic-Engineering-Framework`
  `origin/master` via channel-1 (survives `fw upgrade` re-vendoring, PL-022).
  **Verified on remote:** commit `f87f8e97` is ancestor of `origin/master`
  (local == origin); `git show origin/master:lib/pickup-channel-bridge.sh |
  grep -c 'termlink event broadcast'` = 0 (and 0 for the learning bridge);
  `bash -n` clean on both remote blobs; `termlink channel post` retained
  (5 occurrences each).
- [x] T-1166 `## Updates` records the root cause + this fix as the cut-unblocker
  (2026-05-26 entry: "CUT-BLOCKER ROOT CAUSE FOUND + FIXED (T-1814)").

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
bash -n .agentic-framework/lib/pickup-channel-bridge.sh
bash -n .agentic-framework/lib/publish-learning-to-bus.sh
! grep -n 'termlink event broadcast' .agentic-framework/lib/pickup-channel-bridge.sh
! grep -n 'termlink event broadcast' .agentic-framework/lib/publish-learning-to-bus.sh
grep -q 'termlink channel post' .agentic-framework/lib/pickup-channel-bridge.sh
grep -q 'termlink channel post' .agentic-framework/lib/publish-learning-to-bus.sh

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

### 2026-05-26 — How to stop the bridge's legacy emission
- **Chose:** Remove the `event.broadcast` fallback outright; degrade to a logged no-op on channel.post failure.
- **Why:** The fallback points at a primitive termlink is actively retiring (T-1166), so it is dead-weight-trending-to-harmful. It was the lone live emitter resetting the cut's clean-window gate. The bridges are explicitly non-fatal "pure enhancement" code (T-1214), so no-op-on-failure is the designed degradation. `--ensure-topic` (T-1443+) already covers the missing-topic case the fallback was protecting against. CLAUDE.md: delete unused/harmful code rather than gate it behind a flag.
- **Rejected:** (a) Add an opt-in env flag to preserve the fallback — rejected per "no feature flags / backwards-compat shims when you can just change the code"; no fleet hub lacks channel.post (.122 HAS it — it just fails). (b) Deploy the staged binary to .122 to make channel.post succeed — complementary operator action (T-1438) but doesn't fix the framework-wide latent landmine, and .122 is flaky/auth-gated. (c) Probe hub.capabilities to gate the fallback on `legacy_primitives` — more code, more risk, same end state.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-25T22:03:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1814-remove-eventbroadcast-fallback-from-fram.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-6f70e505
- **Timestamp:** 2026-05-25T22:08:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-05-25T22:08:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
