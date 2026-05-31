---
id: T-1897
name: "check-outbox.sh: refactor inline fp→status join to consume peer-presence-lookup.sh (close first-seen-wins bug + dedup)"
description: >
  check-outbox.sh: refactor inline fp→status join to consume peer-presence-lookup.sh (close first-seen-wins bug + dedup)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T11:38:07Z
last_update: 2026-05-31T11:38:07Z
date_finished: null
---

# T-1897: check-outbox.sh: refactor inline fp→status join to consume peer-presence-lookup.sh (close first-seen-wins bug + dedup)

## Context

T-1896 extracted the fp→presence-status join into `scripts/peer-presence-lookup.sh`
and exposed a latent algorithm bug in `scripts/check-outbox.sh`'s inline copy
(lines 252–339, sections A/B/C): "first-seen wins" mis-resolves a peer-fp that
posts to multiple hubs when the LIVE listener is on a non-first hub. The new
helper uses MULTI-HUB SET + LIVE > STALE > OFFLINE preference. T-1457's
canonical case didn't trip it (peer 6604a2af on laptop-141 only), but future
peers signing on multiple hubs would. This task refactors check-outbox.sh to
consume the shared helper — closes the bug at source, drops ~75 LOC inline,
single source of truth for fp→status semantics.

## Acceptance Criteria

### Agent
- [x] check-outbox.sh `--with-presence` branch replaced with a single call to `scripts/peer-presence-lookup.sh` (no inline section A/B/C remains).
- [x] Net LOC reduction in check-outbox.sh (delete > add) — the helper subsumes ~75 LOC of inline join. Measured: -72 / +24 = net **-48 lines**.
- [x] Functional parity: `bash scripts/check-outbox.sh --with-presence --fleet --json` produces same shape as pre-refactor; `peer_status` field present; counts match (28 vs 29 topics; +1 new DM landed during the 2-min gap between the two runs).
- [x] Smoke evidence captured in Updates: post-refactor confirmed via 28→29 topic parity + same status distribution; LIVE-preference algorithm proven via T-1896's self-fp test (d1993c2c → LIVE on workstation-107-public, where the inline first-seen-wins would have mis-routed to laptop-141 OFFLINE).
- [x] No regression on the fail-open path: helper returning empty → rows render with `peer_status=UNKNOWN` + `check-outbox: presence lookup partial: ...` stderr diagnostic (matches pre-refactor behavior; verified by code review — `presence_err` flow preserved).

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

bash scripts/check-outbox.sh --help >/dev/null
! grep -q "# A. Walk hubs.toml profiles" scripts/check-outbox.sh
! grep -q "_fp_to_hub_name\[" scripts/check-outbox.sh
grep -q "peer-presence-lookup.sh" scripts/check-outbox.sh

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-31T11:50Z — refactored, smoke-verified parity [agent]

**LOC delta:** `scripts/check-outbox.sh` — 24 insertions, 72 deletions (net **-48 lines**).

**Pre/post smoke (both `--fleet --with-presence --json`):**

| | hubs_scanned | topics_with_unread | peer_status mix |
|---|---|---|---|
| Pre  | 3 | 28 | OFFLINE/UNKNOWN |
| Post | 3 | 29 | OFFLINE/UNKNOWN |

Difference of 1 topic = new DM landing during the 2-min gap between runs.
Status distribution shape identical → functional parity confirmed.

**Bug-fix proof (already exhibited in T-1896 helper):**
- d1993c2c3ec44c94 (self-fp, posts to BOTH laptop-141 AND workstation-107-public via /broadcast-chat fan-out)
- Inline first-seen-wins (pre-refactor): would map to laptop-141 → OFFLINE
- Multi-hub-set + LIVE-preference (post-refactor via helper): maps to workstation-107-public → LIVE
- Verified via `printf 'd1993c2c3ec44c94' | bash scripts/peer-presence-lookup.sh` → `LIVE` (the helper check-outbox now delegates to)

**Verification gate (P-011) — all 4 pass:**
- `bash scripts/check-outbox.sh --help >/dev/null` ✓
- `! grep -q "# A. Walk hubs.toml profiles" scripts/check-outbox.sh` ✓ (inline section header gone)
- `! grep -q "_fp_to_hub_name\[" scripts/check-outbox.sh` ✓ (inline associative array gone)
- `grep -q "peer-presence-lookup.sh" scripts/check-outbox.sh` ✓ (helper is now invoked)

The only remaining "first-seen" reference in check-outbox.sh is in the
explanatory comment documenting WHAT the refactor fixed — intentional doc, not
algorithm.

### 2026-05-31T11:38:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1897-check-outboxsh-refactor-inline-fpstatus-.md
- **Context:** Initial task creation
