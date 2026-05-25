---
id: T-1803
name: "Watchtower shared-launcher: deterministic per-project port + triple self-validation (T-1802 follow-up)"
description: >
  On a multi-project host (.107 runs termlink :3003 + 050-email-archive :3001 + others), bin/watchtower.sh can advertise/drift onto a port another project owns, and the watchtower.{pid,port,url} triple can be internally inconsistent (pid=real instance, url/port=neighbor). Harden: refuse to kill a foreign port holder; validate that watchtower.url/port resolve to THIS project's PROJECT_ROOT (cross-check served instance) before trusting them; surface a doctor warning on drift. Lives in vendored .agentic-framework — propagate upstream via channel-1.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T15:40:43Z
last_update: 2026-05-25T21:25:38Z
date_finished: 2026-05-25T21:25:38Z
---

# T-1803: Watchtower shared-launcher: deterministic per-project port + triple self-validation (T-1802 follow-up)

## Context

On a multi-project host (.107 runs termlink :3003 + 050-email-archive :3001 +
others), `bin/watchtower.sh start` (do_start, lines ~142-163) blindly
`fuser -k`s whatever holds the target port when `port_in_use` — so a port-drift
or a `--port` collision can KILL a neighbor project's live service (the T-1802
wrong-dashboard incident). The reader (`lib/watchtower.sh::_watchtower_url`) is
already identity-hardened (T-1284/T-1290: verifies `/api/_identity` project_root
== ours before trusting the triple), but the WRITER is not.

This task hardens the writer: (1) refuse to kill a port holder that is not THIS
project's Watchtower; (2) identity-verify the served instance before writing the
`watchtower.{port,url}` triple, so the triple can never point at a neighbor. The
identity check is factored into a sourceable `lib/watchtower.sh` helper reused by
both reader and writer (DRY). The third item in the original description — a
`fw doctor` drift warning — is carved to a follow-up to keep this deliverable
bounded (one deliverable = launcher hardening).

Lives in vendored `.agentic-framework` → propagate upstream via channel-1
(follow-up).

## Acceptance Criteria

### Agent
- [x] `lib/watchtower.sh` exposes sourceable `_watchtower_identity_matches <url>` and `_watchtower_port_holder_is_ours <port>`; `_watchtower_url`'s existing inline identity check delegates to the new helper (no behavior change to the reader) — verified `_watchtower_url` still returns `http://192.168.10.107:3003`
- [x] `do_start` refuses to kill a port holder that is NOT this project's Watchtower: when `port_in_use` and the holder fails the identity check, it errors loud (names the foreign holder, suggests `--port N`) and exits non-zero WITHOUT sending any signal
- [x] `do_start` still recycles its OWN stale instance: when the port holder identity-matches this project, the existing TERM/KILL free-the-port path runs as before
- [x] After health check, `do_start` writes the `watchtower.{port,url}` triple ONLY after confirming the served instance identity-matches this project; on mismatch it fails loud and does not write a triple pointing at a foreign instance
- [x] `lib/watchtower.sh` sources cleanly under `set -euo pipefail` (the `&& return 0` load-guard does not abort a strict-mode sourcing caller) — verified
- [x] A test (`scripts/test-watchtower-guard.sh`) proves: foreign holder → `_watchtower_port_holder_is_ours` false (not killed); this project's running Watchtower → true; non-destructive (never kills the live instance). SKIPs cleanly if no Watchtower/curl — ALL PASS (positive :3003, negative :39517, read-only survival)
- [x] `bash -n` clean on both modified scripts + the test — verified (pre-existing shellcheck info/style notes in untouched code only; no new findings)

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
bash -n .agentic-framework/lib/watchtower.sh
bash -n .agentic-framework/bin/watchtower.sh
bash -n scripts/test-watchtower-guard.sh
FRAMEWORK_ROOT=/opt/termlink/.agentic-framework PROJECT_ROOT=/opt/termlink bash -c 'set -euo pipefail; source /opt/termlink/.agentic-framework/lib/watchtower.sh; type _watchtower_port_holder_is_ours >/dev/null && type _watchtower_identity_matches >/dev/null'
bash scripts/test-watchtower-guard.sh

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

### 2026-05-25 — Scope: launcher hardening now, doctor-drift lint as follow-up
- **Chose:** Ship the two writer-side fixes (foreign-port-kill guard + identity-verified triple write) in this task; carve the `fw doctor` drift-warning (3rd item in the original description) into a follow-up.
- **Why:** One deliverable = launcher hardening. The kill-guard is the high-severity fix (it can kill a neighbor's live service); the doctor lint is observability of a condition the guard now prevents at the source. Keeping them separate keeps this bounded and the diff reviewable.
- **Rejected:** Bundling the doctor lint here — would widen scope into `fw doctor` and delay the high-value kill-guard.

### 2026-05-25 — Identity check lives in lib/watchtower.sh, reused by reader+writer
- **Chose:** Factor the `/api/_identity` handshake into top-level `_watchtower_identity_matches` / `_watchtower_port_holder_is_ours` in the already-shared lib; have the reader's inline check delegate.
- **Why:** DRY — one identity implementation for both the reader (which already had it inline) and the new writer guard. Also makes the decision logic unit-testable without invoking the destructive `do_start`.
- **Rejected:** Duplicating the curl/grep handshake inside bin/watchtower.sh — two copies drift.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-25T15:40:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1803-watchtower-shared-launcher-deterministic.md
- **Context:** Initial task creation

### 2026-05-25T21:20:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.4)

- **Scan ID:** R-c08d99cb
- **Timestamp:** 2026-05-25T21:25:39Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-25T21:25:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
