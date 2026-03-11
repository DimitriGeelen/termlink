# T-094: Volatile Conversation Loss — Prevention Research Artifact

> Created: 2026-03-11 | Status: Complete | Task: T-094

## Problem

Research conversations generate valuable insights that live only in conversation context.
When a session ends without explicit artifact capture, all content is permanently lost.

**The triggering event:** A full session of Agent Mesh research (integrating TermLink into
the framework for multi-agent coordination) was conducted on 2026-03-11. Session ended
without creating a task, writing a research artifact, or committing anything. All content
permanently lost until partially recovered from the following session's conversation history.

## Root Cause (Confirmed by 5-Agent Investigation)

The framework's task gate and enforcement hooks only fire on **file operations** (Write/Edit/Bash).
A long conversation that produces no file writes bypasses all structural enforcement completely.
Agent discipline is the only guard — and it failed.

Three independent safeguards all missed the same event:

| Safeguard | Why it missed |
|---|---|
| C-001 artifact-first rule | No inception task was created to trigger it — C-001 only fires once an inception task exists |
| Session capture protocol | Agent didn't run it; no SessionEnd hook exists to enforce it |
| Commit cadence rule (every 15-20 min) | No file ops = no commits = no checkpoints |

**Deeper root:** All three failed for the same reason — the task gate is file-operation-bound.

---

## 5-Agent Investigation Findings

### Agent 1: Hook Coverage — FAIL (platform limitation)

Claude Code exposes exactly 4 hook event types:
- `PreToolUse` — fires before any tool
- `PostToolUse` — fires after any tool
- `PreCompact` — fires before context compaction (manual `/compact` only)
- `SessionStart` — fires at session initialization (with matchers)

**No conversation-level hook exists.** No `PostMessage`, `OnResponse`, `PreMessage`, or similar.
A session with zero tool calls fires zero hooks. This is a Claude Code platform limitation — it
cannot be remediated within the framework. Hooks are tool-boundary events only.

**Verdict:** Cannot add a hook that fires on conversation length. Architecturally impossible.

### Agent 2: Session Lifecycle — Root cause confirmed

**No `SessionEnd` hook type exists in Claude Code.** When a session ends normally (user closes
terminal), zero hooks fire. Framework can only inject capture logic at:

1. PostToolUse checkpoint (fires per tool call, can auto-handover at 170K tokens)
2. Manual `fw handover` (user-triggered, no enforcement)
3. Session capture checklist (manual discipline protocol, not automated)
4. PreCompact (fires only on explicit `/compact`, not normal exit)

Lost session had zero file writes → zero tool calls after conversation → zero PostToolUse
triggers → no auto-handover → session ended silently.

**Verdict:** Three injection points all require either tool activity or explicit user action.
No structural way to intercept a conversation-only session ending.

### Agent 3: Protocol Rules — Exact gap identified

C-001 ("Research Artifact First") requires an inception task with `workflow_type: inception`
to already exist. The lost session started as `/resume` → `proceed` → organic conversation.
No task was created, so C-001's trigger condition was never met.

The CLAUDE.md rule at "Working with Tasks" says "Before reading code, editing files, or invoking
skills" — but conversing with the human is none of these. The rule has no trigger on conversation.

**Proposed rule addition ("Exploratory Conversation Guard"):**

> When in `/resume` or autonomous mode and the conversation shifts to exploring a new
> architectural question, design problem, or assumption space (3+ substantive exchanges
> on a topic not covered by an active task):
> 1. Stop and create an inception task: `fw inception start "[question]"`
> 2. Create `docs/reports/T-XXX-*.md` immediately
> 3. Record prior dialogue in `## Dialogue Log` section
> 4. Continue exploration with artifact as live document
> 5. Commit after each substantive exchange

### Agent 4: `/capture` Skill — FULLY FEASIBLE

Skills are `.claude/commands/*.md` prompt files. When user types `/capture`, the prompt is
injected and the skill executes within the current session with access to all tools
(Bash, Read, Write, git).

**What it CAN do:**
- Write files (creates artifact with task ID + timestamp + user-provided content)
- Run `git commit` via Bash
- Read session state (focus, active task)
- Ask user for summary/content input

**What it CANNOT do:**
- Access conversation history automatically (no hook exposes it)
- Run silently in background (requires user invocation)

**Design:** User types `/capture` → skill asks "What to capture?" → user pastes summary →
skill creates `docs/reports/T-XXX-capture-{timestamp}.md` → commits it → reports back.
Acts as an emergency ejector seat: when user realizes conversation is untracked, one
command writes it to disk.

**Verdict:** GO. Low complexity (<100 lines), no breaking changes, opt-in.

### Agent 5: Episodic Pattern Mining — Known pattern, HIGH recurrence risk

This is NOT a one-off. Known failure pattern **FP-004** ("Context exhaustion before handover")
was documented since T-059. Practice **P-009** explicitly names the anti-pattern ("running 72
tool calls without committing, generating skeleton handovers with [TODO] placeholders").

Evidence of recurrence:
- Two skeleton handovers generated in 24 hours (S-2026-0311-0949: 34 unfilled [TODO] sections,
  S-2026-0311-1021: partially enriched)
- FP-004 mitigation in place but bypassed by workflow gap (manual enrichment step never done)
- 0 explicit "session research lost" incidents in episodic memory — but 2 near-misses in 24h

**Root:** Mitigations (FP-004, P-009) are detective not preventive. The handover-enrichment
step is manual. Skeleton handover auto-generation gives false sense of security.

**Verdict:** First explicit loss recorded. Recurrence risk HIGH. Structural (not discipline)
fix required.

---

## Synthesized Findings

### What Cannot Be Fixed (Platform Limits)
- No conversation-level hook can be added to Claude Code
- No SessionEnd hook exists — normal session exit is invisible to the framework
- Automatic background conversation capture is impossible

### What CAN Be Fixed (3 layers)

**Layer 1 — Protocol (CLAUDE.md rule change):**
Add "Exploratory Conversation Guard" rule. Agent must stop at 3 substantive exchanges on
untracked topic and create inception task + artifact. This is behavioral enforcement, not
structural, but it closes the C-001 trigger gap.

**Layer 2 — Tooling (`/capture` skill):**
Emergency ejector seat for when Guard rule fails or user initiates. One command writes
conversation summary to disk and commits. Needs to be built and registered in both
`.claude/commands/` and framework-level commands.

**Layer 3 — Pattern update (FP-004 + gaps.yaml):**
Register the specific sub-variant: "conversation-only sessions" as a named gap. Update FP-004
to explicitly call out this scenario. This makes the problem visible in future audits.

---

## Remediation Design

### R-1: Exploratory Conversation Guard (CLAUDE.md)
**Type:** Protocol rule addition
**Where:** `Inception Discipline` section of CLAUDE.md, after current C-001 rule
**What:** Add the "Exploratory Conversation Guard" rule (Agent 3 draft)
**Effort:** Small — edit CLAUDE.md, no code changes

### R-2: `/capture` Skill
**Type:** New skill file
**Where:** `.claude/commands/capture.md` (project) + `skills/capture.md` (framework)
**What:** Emergency ejector seat for untracked conversation content
**Effort:** Small — one markdown file with prompt + Bash commands

### R-3: Gap Registration
**Type:** Framework tracking
**Where:** `gaps.yaml` + `patterns.yaml` FP-004 update
**What:** Register "conversation-only session loss" as named gap G-XXX
**Effort:** Trivial — YAML edits

### R-4: Framework-Wide Distribution
**Type:** Framework PR (for framework agent)
**Where:** Framework repo (not this project)
**What:** Propagate R-1 (protocol rule) and R-2 (capture skill) to all framework consumers
**Effort:** Medium — PR in framework repo

---

## Implementation Tasks

- T-095: Add Exploratory Conversation Guard to CLAUDE.md
- T-096: Build `/capture` skill (`.claude/commands/capture.md`)
- T-097: Register gap G-XXX + update FP-004 in patterns.yaml
- T-098: Create OneDev issue for framework-agent PR

---

## OneDev Entry

See: `https://onedev.docker.ring20.geelenandcompany.com/termlink`
Issue created: (see task T-098)

---

## Dialogue Log

**Human (2026-03-11):** "That's bad news we lost it!!! Before anything else deep think about
how we can prevent this in the future this is really really bad."

**Agent analysis:** Root cause identified as structural gap — task gate is file-op only,
conversation content is invisible to all enforcement mechanisms.

**Human:** "Spawn 5 agents to explore, but first document our conversation from before to
be picked-up after. After the 5 agent exploration we need to design remediation, create tasks
to implement and create a Onedev entry for the framework agent to pickup and incorporate.
Once PR is created a prompt with all details to pick-up PR need to be generated on the console
so the user can cut and paste this to the framework agent."

**Human:** "Why are we spawning background agents and not using our terminal agent functionality?"

**Agent:** The Agent Mesh (TermLink-coordinated agents) doesn't exist yet — that's what
the recovered research was designing. We used Claude Code's built-in Agent tool as the
current available mechanism. This is precisely the gap the Agent Mesh would close.
