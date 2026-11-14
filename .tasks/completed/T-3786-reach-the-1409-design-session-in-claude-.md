---
id: T-3786
name: "Reach the 1409 design session in Claude Desktop: can peer mail surface there?"
description: >
  Operator 2026-11-14: the 1409 design session runs in the Claude Desktop client as
  dimitri-mint-dev; mail to 1409-sprind lands at the root control session instead.
  On .107 claude-desktop and termlink mcp serve run as dimitri-mint-dev: MCP tools,
  but no terminal to inject into and no prompt hook. Question: can a Desktop session
  be a mail recipient, and how?

status: work-completed
workflow_type: inception
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-11-14T10:24:14Z
last_update: 2026-11-14T14:17:15Z
date_finished: 2026-11-14T14:17:15Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.6                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
bvp_scores_proposed:
  - ts: '2026-11-14T11:54:07Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3786: Reach the 1409 design session in Claude Desktop: can peer mail surface there?

## Problem Statement

Project 1409's design session runs in the Claude Desktop client as the non-root user `dimitri-mint-dev`. Mail
addressed to `1409-sprind` lands at 1409's root control session instead (1409 reply fee98d39, 2026-11-14), so
anything meant for the design session travels through the operator by hand. The operator (2026-11-14): "Wherever
the design session is in the desktop client, I don't know if we can facilitate that kind of communication, it
would be good though." Why now: T-3781/T-3782 gave that user's sessions a place on the hub; the remaining gap is
that the desktop session itself cannot receive.

## Assumptions

- A-1: the design session is a Claude Desktop chat (not a Claude Code session inside the desktop app).
- A-2: the desktop session reaches TermLink only through the MCP server `termlink mcp serve` it launched.
- A-3: Claude Desktop shows the model nothing an MCP server pushes on its own (no hook, no surfaced notification),
  so delivery would have to be pulled by the session or the operator.

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

- **IW-1: What kind of session is the 1409 design session (Claude Desktop chat, Claude Code in the desktop app,
  or other), and what can it run or call?**
  confidence: 3
  disposition: answered
  rationale: Claude Code 2.1.321 run by claude-desktop (stream-json, no -p) in /opt/1409-sprind with project
    settings loaded, so 1409's sidecar mail hooks run on every prompt (report S1, ps/pwdx 2026-11-14).
- **IW-2: Can that session be a mail recipient (own address, inbox, a way for mail to reach its next turn)?**
  confidence: 3
  disposition: answered
  rationale: yes in design — it already has the inbox `1409-sprind/design` — but every read fails as
    dimitri-mint-dev ("Hub is not running (no socket at /run/user/1000/termlink/hub.sock)", 637+228 transcript
    hook lines, latest 11:55Z today); fix = read via the user's TCP hub (report S2, option A).

## Exploration Plan

- S1 (read-only, 20 min): what runs as dimitri-mint-dev: process tree under claude-desktop, the MCP server's
  parent, any Claude Code (`claude`) processes; whether hooks are configured. Answers IW-1.
- S2 (read-only, 30 min): what each kind of session can receive: Claude Desktop (MCP tools, resources, prompts,
  notifications — documented behaviour), Claude Code (UserPromptSubmit hook, as our own sessions use with the
  sidecar receiver). Answers IW-2 with options.
- Report: `docs/reports/T-3786-desktop-session-reach.md`.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

- The session belongs to another project (1409) and another user: read-only inspection only; nothing in that
  user's home or config is changed by this inception.
- No secrets are read or copied (MCP configs may hold tokens: read structure, not values).

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->
IN: how mail can reach the 1409 design session; options with cost. OUT: building any of them (needs GO); changing
1409's own routing (theirs to decide); reaching other desktop users.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
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

**Recommendation:** GO

**Rationale:** The design session is Claude Code run by the desktop app and already has its own inbox
(`1409-sprind/design`) and the mail hooks; mail never surfaces only because termlink, run as dimitri-mint-dev
with no `--hub`, looks for a hub socket in that user's runtime dir. Option A (termlink falls back to the user's
TCP hub when it has a hub link) is one bounded, reversible rule that fixes every caller without a vendored change.

**Evidence:**
- ps/pwdx: three `claude` (Claude Code 2.1.321) processes under claude-desktop, cwd /opt/1409-sprind,
  `--setting-sources=user,project,local`, no `-p` (docs/reports/T-3786-desktop-session-reach.md S1).
- 1409's .claude/settings.json: UserPromptSubmit sidecar-inbox + sidecar-receiver-adapter.
- Transcript hook lines: 637× NOT CHECKED reading `inbox:…/1409-sprind/design`, 228× the project inbox,
  error "Hub is not running (no socket at /run/user/1000/termlink/hub.sock)", latest 2026-11-14T11:55Z (S2).
- The user's route exists: hubs.toml profile 127.0.0.1:9100 with its own token (T-3756/22c) and the T-3781 link.

## Decisions

### 2026-11-14 — how to reach the 1409 design session (operator ruling, chat)
- **Chose:** A — termlink falls back to the user's TCP hub (its hub link address + its own credential) when no
  `--hub` is given, no local hub socket exists and the caller is non-root with a hub link.
- **Why:** the measured fault is termlink choosing a hub socket that cannot exist for this user; A fixes it at
  that one point for every caller (mail hook, MCP, scripts) with no vendored change.
- **Rejected:** B (AEF passes `--hub`: upstream, fixes only fw's calls); C (do nothing).
- **Conditions the operator set:** no compromise on quality or design principles: explicit (says which hub it
  used), fail closed (half a link or a refused credential is an error, never an empty inbox), nothing existing
  switched off, proven live (sentinel to `1409-sprind/design` shown in the design session's transcript).
- Operator words in chat: "Well, we want to make it working, but we don't want the great quality and design
  principles, right? Or the design ideas." then "But if A is the right solution, I'm happy to go forward with
  that." Read as: GO on A without compromising quality (overturnable).
- Formal `fw inception decide` is Tier 0 (operator only): Watchtower http://192.168.10.107:3003/inception/T-3786.

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

**Decision**: GO

**Rationale**: The design session is Claude Code run by the desktop app and already has its own inbox
(`1409-sprind/design`) and the mail hooks; mail never surfaces only because termlink, run as dimitri-mint-dev
with no `--hub`, looks for a hub socket in that user's runtime dir. Option A (termlink falls back to the user's
TCP hub when it has a hub link) is one bounded, reversible rule that fixes every caller without a vendored change.

**Date**: 2026-11-14T14:17:14Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-11-14T11:54:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-11-14T14:17:14Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** The design session is Claude Code run by the desktop app and already has its own inbox
(`1409-sprind/design`) and the mail hooks; mail never surfaces only because termlink, run as dimitri-mint-dev
with no `--hub`, looks for a hub socket in that user's runtime dir. Option A (termlink falls back to the user's
TCP hub when it has a hub link) is one bounded, reversible rule that fixes every caller without a vendored change.

### 2026-11-14T14:17:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e0d4e7f4
- **Timestamp:** 2026-11-14T14:17:17Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Scope:** static anti-pattern scan; PASS means no anti-pattern was found, not that the criteria hold
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-1
     - evidence: `IW-1 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`
  2. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-2
     - evidence: `IW-2 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-36d24587
- **Timestamp:** 2026-11-14T14:17:17Z
- **Overall:** CONFIRMED
- **Claims:** 2

| Claim | Type | Status |
|-------|------|--------|
| `T-3756` | task | ✓ pass |
| `T-3781` | task | ✓ pass |
