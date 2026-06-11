---
id: T-2155
name: "RCA: misread budget at post-compaction /resume — historical task-output JSON parsed as current state, propose structural fix for framework /resume skill"
description: >
  Inception: RCA: misread budget at post-compaction /resume — historical task-output JSON parsed as current state, propose structural fix for framework /resume skill

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-11T10:32:26Z
last_update: 2026-06-11T10:36:15Z
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

# T-2155: RCA: misread budget at post-compaction /resume — historical task-output JSON parsed as current state, propose structural fix for framework /resume skill

## Problem Statement

After /resume from compaction, the agent has no structural prompt to verify the
current budget level against `.context/working/.budget-status` (the canonical
cache). Instead, post-compaction context injection includes historical tool
results as system-reminders — and those reminders can contain stale JSON that
LOOKS current (e.g. `{"level":"urgent","tokens":273016,...}` from a prior
session's Read of a Task tool ephemeral output file under
`/tmp/claude-0/.../tasks/<id>.output`).

This session reproduced the slip end-to-end: agent claimed budget was 273K
(urgent) at start, narrated the entire session as having "27K headroom up to
the user-authorized 300K", and wrapped at "~298K" — when the actual current
budget at session start was 159350 (level=ok), verified by
`cat .context/working/.budget-status`. ~140K of real headroom was un-used.

## Analysis

GO — file structural fix proposal to framework-agent via `framework:pickup`
topic. (Canonical Recommendation block below — this section captures the
underlying RCA detail referenced by that block's Evidence.)

### Symptom
Agent narrates budget level based on historical tool-result JSON pulled from
system-reminders, not the canonical cache. Behaviour constrained as if at
critical when actually at OK.

### Root cause
Two compounding gaps:
1. The `/resume` skill workflow (gather → summarize → suggest) does NOT include
   "cat `.context/working/.budget-status`". It reads handover, git status,
   tasks, tool counter, web server — but skips the budget cache.
2. The SessionStart:compact context-recovery flow re-injects historical tool
   results verbatim. A historical Task tool output containing budget-shaped
   JSON is structurally indistinguishable to the agent from a current read of
   the cache (same key names: `level`, `tokens`, `timestamp`, `source`).

### Why structurally allowed
- CLAUDE.md "After context compaction" section names `fw resume status` + `fw
  resume sync` but does NOT name `.context/working/.budget-status` as a
  required check.
- The budget-gate hook caches level/tokens in the file but never re-asserts it
  into agent context. Enforcement is runtime (blocks tools) but visibility is
  pull-only.
- No framework signal forces "ground budget claims against cache".
- Historical system-reminders are not visually marked as historical — they look
  identical to current tool results.

### Prevention (structural fix proposal)

Three options, ordered by least-invasive:

**A. Extend `/resume` skill (smallest change).**
   Add Step 1.6 to the gather phase:
   ```
   6. cat `.context/working/.budget-status` 2>/dev/null
   ```
   And add to the Summary template:
   ```
   - Budget: {level} ({tokens} tokens) from cache
   ```
   One-line addition. Ships via userSettings:resume update.

**B. SessionStart:compact hook surfaces current budget (best-leverage).**
   Hook re-reads `.context/working/.budget-status` post-compaction and
   prepends "Current budget: level={X} tokens={Y}" to the persisted-output
   block, BEFORE any historical tool results. This makes ground truth the
   first thing the agent sees, not buried in stale snapshots.

**C. CLAUDE.md doc-only (weakest).**
   Update "After context compaction" recovery steps to name the cache file
   explicitly. Relies on agent reading + obeying — same failure mode that
   produced this slip.

Recommended: **B as primary** (forces correct context independently of agent
behaviour). **A as fallback / defence-in-depth.**

### Pickup to framework-agent
This fix lives at the framework layer (`/resume` ships from userSettings;
SessionStart:compact hook ships from framework). Per T-1814-class escalation
pattern, post to `framework:pickup` topic with: symptom, root cause, three
options, recommendation, this task's ID for traceback.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Is Option B (SessionStart:compact hook surfaces current budget) the right primary fix vs Option A (extend /resume skill) alone?**
  confidence: 2
  disposition: answered
  rationale: B forces correct context independently of agent behaviour (hook-side enforcement); A relies on agent reading + obeying the skill — same failure mode that produced this slip. B as primary + A as defence-in-depth is the operator-leverage answer. Reproduced this turn: agent did follow /resume's existing steps but no step named the budget cache, so behaviour-only fix would have closed nothing.

- **IW-2: Does the structural fix belong in the framework (`/opt/999-AEF`) or this consumer project (`/opt/termlink`)?**
  confidence: 3
  disposition: answered
  rationale: Both surfaces ship from framework — `/resume` is a userSettings-level skill installed by framework, SessionStart:compact hook is framework-defined. Project-side fix would not propagate to other consumer projects. Pickup-to-framework-agent is the correct escalation path.

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

**Recommendation:** GO

**Rationale:**

Direct evidence: this session I parsed {"level":"urgent","tokens":273016,...} from a system-reminder echoing a prior session's Read of /tmp/claude-0/.../tasks/bp1ad5lmy.output (a Task tool ephemeral output file) and treated it as the current budget state. Actual current budget at session start was 159350 tokens (level=ok), verified by cat .context/working/.budget-status. The result was a session-long misread that constrained behaviour as if 27K from critical when ~140K of real headroom existed. The /resume skill workflow does not include 'cat .context/working/.budget-status' — it reads handover, git status, tasks, tool counter, web server. Post-compaction context-recovery injects historical tool results as system-reminders, which can contain stale JSON that LOOKS current. Structural fix is framework-side (/resume skill ships from userSettings, SessionStart:compact hook ships from framework). GO recommended because the slip is reproducible (any future post-compaction /resume will hit it whenever a historical tool result contains budget-shaped JSON) and the fix is small (one-line addition to the skill + summary template line).

**Evidence:**

- **Live reproduction this session.** Agent parsed `{"level":"urgent","tokens":273016,...}` from a SessionStart:compact persisted-output block (echo of prior session's Read of `/tmp/claude-0/.../tasks/<id>.output` Task tool ephemeral). Narrated entire session as "27K headroom up to 300K"; actual budget at session start was 159350 tokens (level=ok) per `cat .context/working/.budget-status`. Result: ~140K of real headroom un-used.
- **`/resume` skill gap.** Current skill (userSettings:resume) gathers handover + git status + tasks + tool counter + web server — does NOT include `cat .context/working/.budget-status`. See `.claude/commands/resume.md` Step 1 enumeration. Hook-side budget enforcement is pull-only; nothing forces agent to ground claims against canonical cache.
- **CLAUDE.md "After context compaction" section** (search for "After context compaction (mid-session recovery)") names `fw resume status` + `fw resume sync` — does NOT name `.context/working/.budget-status` as required read.
- **SessionStart:compact context-recovery flow** re-injects historical tool results verbatim. Historical Task tool output containing budget-shaped JSON (same key names `level`/`tokens`/`timestamp`/`source`) is structurally indistinguishable from current cache read.
- **Pickup target identified.** Fix lives at framework layer: `/resume` ships from `userSettings:`, SessionStart:compact hook ships from `framework:` — project-side fix would not propagate. Pickup-to-framework-agent via `framework:pickup` topic is the correct escalation path (T-1814-class).
- **Fix is bounded and reversible.** Option A: one-line addition to `/resume` Step 1 + summary template line (`Budget: {level} ({tokens} tokens) from cache`). Option B: SessionStart:compact hook prepends `Current budget: level={X} tokens={Y}` to persisted-output BEFORE historical snapshots — makes ground truth the first thing agent sees. Recommended: B primary + A defence-in-depth.
- **Sibling task** [T-2156](http://192.168.10.107:3003/review/T-2156) (the pickup envelope to framework-agent) is already captured horizon=next, awaiting GO here to authorize the post.

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

### 2026-06-11T10:33:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
