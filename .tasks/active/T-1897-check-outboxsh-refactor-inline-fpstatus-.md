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
last_update: 2026-05-31T11:42:20Z
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

**Symptom:** check-outbox.sh `--with-presence` could mis-render a peer-fp as
OFFLINE when their LIVE listener is on a hub that appears LATER in hubs.toml
than another hub the same fp posts to. Concrete instance: self-fp
d1993c2c3ec44c94 posts presence to both laptop-141 AND workstation-107-public
(via /broadcast-chat fan-out). The inline mapping committed to laptop-141
(first in hubs.toml ordering), which has no LIVE listener — so the row showed
OFFLINE even though the LIVE listener was running on workstation-107-public.

**Root cause:** Section A's "first-seen wins" rule (lines 296-297 of the prior
check-outbox.sh, comment `# First-seen wins per fp.`) commits a peer-fp to the
first hub where it's seen as a presence sender, then never reconsiders. The
algorithm treats "fp is a presence sender on hub X" as a binding identity
claim, when in reality it's just "this fp has posted here at some point."
Broadcast fan-out makes the binding noisy across hubs.

**Why structurally allowed:** The inline join was written before the multi-hub
fan-out pattern (broadcast skill T-1856) became common — at design time, peers
were assumed to post presence to their own local hub only. Once
/broadcast-chat shipped, the assumption broke silently because:
(a) T-1457's canonical case (peer 6604a2af on laptop-141 only) didn't trip it
(b) test smoke for --with-presence shipped against a single-listener fleet
(c) no test fixture exercised the "fp on multiple hubs with LIVE on non-first" case
The 75 LOC of inline join was also a duplication-of-truth risk: when the
identical algorithm got extracted into peer-presence-lookup.sh for /check-arc
(T-1896), the bug was found and fixed in the new copy but stayed latent in the
old one until this refactor.

**Prevention:**
1. **Single source of truth** — `peer-presence-lookup.sh` is now the only
   implementation of the fp→status join. Both /check-outbox and /check-arc
   consume it. Correctness fixes land in one place.
2. **Algorithm encoded as data structure** — `_fp_to_hubs[fp]` (multi-hub set)
   makes the multi-residence reality explicit; the resolve walk is forced to
   handle it. The prior `_fp_to_hub_name[fp]` (single hub) hid the case.
3. **Helper exposed as testable verb** — `--all` + `--json` give cheap smoke
   coverage. A regression test that asserts a known multi-hub fp routes to its
   LIVE hub would catch the next "first-seen-wins drift."

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
