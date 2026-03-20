---
id: T-196
name: "Architectural mitigation for macOS BSD/GNU shell incompatibilities"
description: >
  Inception: How to architecturally mitigate macOS BSD/GNU shell incompatibilities
  in the framework's bash scripts (date -d, declare -A, head -n -1, stat -c).
  Priority: Linux (must) > macOS (must) > Windows/WSL (bonus).

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [framework, macos-compat, portability]
components: []
related_tasks: [T-160]
created: 2026-03-20T20:19:57Z
last_update: 2026-03-20T20:34:46Z
date_finished: null
---

# T-196: Architectural mitigation for macOS BSD/GNU shell incompatibilities

## Problem Statement

Framework bash scripts use 26+ GNU-specific features that break on macOS (BSD userland).
12 are critical (block execution): `date -d`, `declare -A`, `find -printf`, `stat -c`.
Affects episodic generation, audit reports, metrics, and task updates.

**Platform priority:** Linux (must maintain) > macOS (must fix) > Windows/WSL (bonus).

## Assumptions

- A1: Framework already has `lib/compat.sh` with `_sed_i()` — can be expanded (VALIDATED)
- A2: `_date_to_epoch()` pattern in episodic.sh is correct but not shared (VALIDATED)
- A3: Hot-path hooks don't have critical GNU-isms (VALIDATED — breakage is cold-path only)
- A4: Python3 is available as fallback on both platforms (VALIDATED — used in hooks already)
- A5: Associative arrays used in framework have <20 entries (VALIDATED — O(n) scan is fine)

## Exploration Plan

1. [x] Full audit of GNU-isms across all framework scripts (Agent 1 — found 26+)
2. [x] Evaluate portable date/time alternatives (Agent 2 — 6 approaches compared)
3. [x] Research bash version strategies from real-world projects (Agent 3 — nvm/rbenv/git/bats)
4. [x] Analyze hook performance constraints (Agent 4 — hot-path is fine, cold-path needs fix)
5. [x] Design concrete compat.sh expansion (Agent 5 — ~130 lines, 4 function families)
6. [ ] Human review of findings + Go/No-Go decision

See full research: `docs/reports/T-196-macos-bsd-gnu-compat-research.md`

## Technical Constraints

- macOS ships bash 3.2 (2007) — no `declare -A`, no `readarray`
- macOS ships BSD date — no `-d` flag
- macOS ships BSD head — no negative line counts
- Linux uses GNU coreutils — all current code works
- Windows/WSL uses GNU coreutils — would work if bash scripts work on Linux
- Hot-path hooks must stay <50ms (currently ~18ms bash + ~47ms python)
- Framework is a separate repo — changes via pickup prompt, not direct edit

## Scope Fence

**IN:** Architectural recommendation for portable compat layer, pickup prompt for framework
**OUT:** Actually editing framework files (separate repo), Windows-native support (no bash)

## Acceptance Criteria

- [x] Problem statement validated (26+ GNU-isms, 12 critical)
- [x] Assumptions tested (A1-A5 all validated)
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Solution fits in one shared file (~160 lines)
- No new external dependencies required
- Hot-path hooks remain unaffected
- Linux compatibility maintained (GNU date remains primary path)
- Can be deployed incrementally

**NO-GO if:**
- Requires bash 4+ as hard dependency (breaks macOS OOTB)
- Requires Python as hard dependency
- Adds >10ms latency to hot-path hooks
- Breaks existing Linux behavior

**Assessment: All GO criteria met. No NO-GO criteria triggered.**

## Verification

test -f docs/reports/T-196-macos-bsd-gnu-compat-research.md

## Decisions

**Decision**: GO

**Rationale**: Expand lib/compat.sh with 4 portable function families (~130 lines). All GO criteria met: single file, no new deps, hot-path unaffected, Linux primary path maintained. Framework already has the pattern.

**Date**: 2026-03-20T20:25:09Z
## Decision

**Decision**: GO

**Rationale**: Expand lib/compat.sh with 4 portable function families (~130 lines). All GO criteria met: single file, no new deps, hot-path unaffected, Linux primary path maintained. Framework already has the pattern.

**Date**: 2026-03-20T20:25:09Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-20T20:25:09Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Expand lib/compat.sh with 4 portable function families (~130 lines). All GO criteria met: single file, no new deps, hot-path unaffected, Linux primary path maintained. Framework already has the pattern.
