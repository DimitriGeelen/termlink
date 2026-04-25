---
id: T-1269
name: "Apply PL-075 fix — reinstall consumer pre-push hook + restore .agentic-framework/VERSION"
description: >
  T-1252 fixed the framework hook template (lib/hooks.sh L410-412) to stop
  stamping .agentic-framework/VERSION, but install-hooks was never re-run on
  this consumer. Deployed .git/hooks/pre-push L50-52 still has the buggy
  block, causing fw doctor to warn "version mismatch pinned=1.5.307
  installed=0.9.1294". Reinstall hook + restore VERSION to upstream pin.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [hooks, version, framework-drift]
components: []
related_tasks: [T-1252]
created: 2026-04-25T20:17:05Z
last_update: 2026-04-25T20:17:05Z
date_finished: null
---

# T-1269: Investigate framework VERSION drift — handover stamps .agentic-framework/VERSION with consumer version

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
- [x] `.git/hooks/pre-push` no longer contains the `.agentic-framework/VERSION` write block
- [x] `.agentic-framework/VERSION` content equals the `.framework.yaml` `version:` field (1.5.307)
- [x] `fw doctor` shows no "Version mismatch" warning
- [x] PL-075 application field updated from "TBD" to cite T-1269

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

test "$(grep -cE '> .*agentic-framework/VERSION' .git/hooks/pre-push)" = "0"
test "$(cat .agentic-framework/VERSION)" = "$(grep '^version:' .framework.yaml | awk '{print $2}')"
.agentic-framework/bin/fw doctor 2>&1 | grep -v 'Version mismatch' >/dev/null && ! .agentic-framework/bin/fw doctor 2>&1 | grep -q 'Version mismatch'
grep -A 6 '^- id: PL-075' .context/project/learnings.yaml | grep -q 'T-1269'

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
