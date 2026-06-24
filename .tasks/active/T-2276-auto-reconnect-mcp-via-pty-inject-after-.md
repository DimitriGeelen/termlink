---
id: T-2276
name: "auto-reconnect MCP via PTY-inject after binary upgrade"
description: >
  Inception: auto-reconnect MCP via PTY-inject after binary upgrade

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-24T11:39:09Z
last_update: 2026-06-24T11:44:30Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2276: auto-reconnect MCP via PTY-inject after binary upgrade

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Is the Claude Code session a registered termlink session with a PTY we can `inject` into?**
  confidence: 3
  disposition: answered
  rationale: NO (conditional). `termlink inject <TARGET>` only reaches termlink-MANAGED sessions (TARGET = session id/display name; no inject-by-tty/pid). This session is `claude(28635)←bash←su←sudo`, not among the 4 registered sessions (termlink list) — unreachable. Feasible ONLY if claude is launched under termlink (register --shell / claude-fw). Deployment-pattern prerequisite, not retrofittable to a running foreign session.

- **IW-2: Does Claude Code support a non-interactive MCP reconnect that a single injected line can trigger (vs an arrow-key interactive `/mcp` menu)?**
  confidence: 3
  disposition: answered
  rationale: NO. claude-code-guide (2026-06-24): `/mcp` is an interactive panel only, takes NO inline args, NO `/mcp reconnect` form exists; stdio MCP servers are NOT reconnected by it — the ONLY documented path to pick up a replaced stdio binary is a FULL session restart. No SIGHUP/config-watch/flag trigger.

- **IW-3: Is PTY-injecting a slash command into the live TUI safe + effective (submits cleanly; reconnect mid-turn does not corrupt in-flight state)?**
  confidence: 3
  disposition: dissolved
  rationale: MOOT — IW-2 shows there is no reconnect command to inject. The only effective injected sequence would be a full session restart (exit + relaunch), which is exactly what the existing claude-fw auto-restart (T-179) already does via the `.restart-requested` signal — no new PTY-inject feature needed.

## Decision (proposed: NO-GO on the literal idea; pivot recorded)

**NO-GO** on "PTY-inject `/mcp reconnect`": (a) the command does not exist —
`/mcp` is interactive-only and does not reconnect stdio servers; (b) only a full
session restart picks up a new stdio MCP binary; (c) `termlink inject` reaches
only termlink-managed sessions, which this (sudo-launched) session is not.

**Pivot (the real mechanism, for a future build task):** the sanctioned way to
make a session self-activate a new binary is a **full restart**, and termlink
already has the primitive — **claude-fw auto-restart (T-179)**: writing
`.context/working/.restart-requested` makes the claude-fw wrapper relaunch with
`claude -c`, respawning `termlink mcp serve` on the new binary. So a
self-healing deploy = "after install, signal claude-fw-wrapped sessions to
restart" — gated on the session running under claude-fw AND under termlink
management. File a build inception on THAT if we want the deploy to heal its own
last mile. Awaiting human confirmation via `fw task review T-2276`.

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
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:**

Pending exploration of three assumptions: (A1) the Claude Code session is a registered/injectable termlink PTY; (A2) Claude Code supports a non-interactive /mcp reconnect that a single injected line can trigger (vs an interactive menu); (A3) injecting a slash command into the TUI PTY actually submits without disrupting the live turn. No evidence gathered yet.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

### 2026-06-24T11:39:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
