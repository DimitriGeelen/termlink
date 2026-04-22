---
id: T-287
name: "Cross-project framework upgrade — fw-agent applies and validates governance fixes on .112 via TermLink"
description: >
  Inception: Cross-project framework upgrade — fw-agent applies and validates governance fixes on .112 via TermLink

status: captured
workflow_type: inception
owner: human
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-25T20:34:23Z
last_update: 2026-04-22T04:52:51Z
date_finished: null
---

# T-287: Cross-project framework upgrade — fw-agent applies and validates governance fixes on .112 via TermLink

## Problem Statement

Framework agent on .107 has governance fixes. Applying them to consumer projects (.112) is manual and error-prone. We want the fw-agent to connect via TermLink, apply the upgrade to an isolated copy, validate it, and report results — fully automated, no SSH, no manual steps.

**Key challenges:**
1. This terminal has no TermLink session — fw-agent can't target us
2. Upgrades are destructive — need isolation (worktree/clone)
3. Bidirectional connectivity needed (both hubs running, both agents need sessions)
4. Validation means: `fw doctor`, `fw audit`, `cargo test`, hooks firing

## Assumptions

- A1: fw-agent on .107 can reach our hub at .112:9100
- A2: A git worktree provides sufficient isolation for testing upgrades
- A3: fw-agent already knows how to run `fw update` and validate
- A4: `termlink push` + `remote exec` is sufficient for the fw-agent to drive the upgrade
- A5: The fw-agent's TermLink session supports exec (it's registered with appropriate capabilities)

## Exploration Plan

1. **Spike 1: Session topology** — Can we register a TermLink session here for fw-agent to target? Test `register --self` or a separate terminal. (15 min)
2. **Spike 2: Bidirectional connectivity** — Can fw-agent on .107 reach our hub? Does it have our profile saved? (10 min)
3. **Spike 3: Isolation** — Create a git worktree, verify `fw doctor` and `cargo test` work in it (10 min)
4. **Spike 4: Orchestration prototype** — Push upgrade instructions to fw-agent, have it exec commands on the worktree session (30 min)

## Technical Constraints

- Network: .107 ↔ .112 bidirectional, hubs on port 9100 both sides
- fw-agent on .107: already registered, tags master,claude
- .112 hub: running, secret known to .107 (from pickup-connection-instructions.md sent earlier)
- TermLink sessions need to be in the project directory for `fw` commands to work
- macOS on both sides (BSD tools, not GNU)

## Scope Fence

**IN:** Automated framework upgrade delivery + validation via TermLink
**OUT:** Building a general-purpose `termlink upgrade` command (that's a follow-up if GO), multi-project fleet upgrades, rollback mechanisms

## Acceptance Criteria

### Agent
- [x] Problem statement validated (5 spikes completed, bidirectional connectivity proven)
- [x] Assumptions tested (fw upgrade broken confirmed, T-615/T-617/T-618 now landed in framework)
- [x] Recommendation written with rationale (GO — see research artifact + decision)

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Read the research artifact and recommendation in this task
  2. Evaluate go/no-go criteria against findings
  3. Run: `fw inception decide T-XXX go|no-go --rationale "your rationale"`
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Bidirectional TermLink connectivity confirmed (fw-agent can exec on .112 session)
- Git worktree isolation works (fw doctor + cargo test pass in worktree)
- Upgrade sequence can be driven via remote exec (no interactive prompts)

**NO-GO if:**
- fw-agent cannot reach .112 hub (network/auth issues)
- Framework upgrade requires interactive input that can't be scripted
- Isolation overhead makes this slower than manual upgrade

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: Bidirectional connectivity proven, framework blockers T-615/T-617/T-618 landed

**Date**: 2026-03-27T06:45:05Z
## Recommendation

_Backfilled 2026-04-19 under T-1139/T-1112 scope — inception decide ran before `## Recommendation` became a required section. Content mirrors the `## Decision` block below for audit compliance (CTL-027)._

**Decision (retro-captured from Decision block):** GO

**Rationale:** Bidirectional connectivity proven, framework blockers T-615/T-617/T-618 landed

## Decision

**Decision**: GO

**Rationale**: Bidirectional connectivity proven, framework blockers T-615/T-617/T-618 landed

**Date**: 2026-03-27T06:45:05Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-25T20:34:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-25T21:00:00Z — research-artifact [agent]
- **Artifact:** `docs/reports/T-287-cross-project-upgrade.md`
- **Content:** Full inception research — 5 spikes, dialogue log, design decisions, path isolation gap, security findings

### 2026-03-26T23:30:00Z — framework-resolution [T-293 session]
- **Finding:** Priority 1 (T-615 hook enumeration) already landed in framework (commit d24bc1e)
- **Finding:** T-617 (upgrade audit trail) and T-618 (fleet upgrade to v1.3.0) also landed
- **Status:** Framework-side blockers 1-2 from pickup are resolved. Remaining: register --self integration, CLAUDE.md sync validation
- **Pushed:** T-287 findings + T-160 pickup delivered to fw-agent via TermLink file send

### 2026-03-27T06:45:05Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Bidirectional connectivity proven, framework blockers T-615/T-617/T-618 landed

### 2026-04-16T05:38:15Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)

### 2026-04-22T04:52:51Z — status-update [task-update-agent]
- **Change:** horizon: later → next
