---
id: T-1634
name: "Discuss with framework-agent: review-agent path for autonomous closure of RUBBER-STAMP-only tasks"
description: >
  Discuss with framework-agent: review-agent path for autonomous closure of RUBBER-STAMP-only tasks

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-13T06:28:31Z
last_update: 2026-05-13T06:30:10Z
date_finished: null
---

# T-1634: Discuss with framework-agent: review-agent path for autonomous closure of RUBBER-STAMP-only tasks

## Context

Dimitri pushed back on the framework rule that owner=human tasks must be closed by the human, even when all Human ACs are [RUBBER-STAMP] and agent-evidenced. Live trigger: T-1294 — 4/4 ACs ticked Apr 26-28, re-verified today via live `hub.bus_state` probe; still gated on human typing `fw task update --status work-completed`.

He recalls prior discussion with framework-agent (upstream /opt/999-AEF) about standing up a "review-agent" or `fw task rubber-stamp` verb that lets the agent close when all ACs are [RUBBER-STAMP]. Asking me to find out where that work stands.

Concrete proposal he restated: split closure authority on AC-prefix.
- all [RUBBER-STAMP] + agent-evidenced → agent may close
- any [REVIEW] → human must close

Prior shipped on this side (termlink project):
- PL-154 + T-1628: triage flag pattern (`--compact --by-age --rubber-stamp-only`)
- G-XXX concern (concerns.yaml) — status=watching, refilled 0→5 today
- `fw task verify --rubber-stamp-only --compact --by-age` is live

## Acceptance Criteria

### Agent
- [x] Status-check posted to framework-agent — posted to `framework:pickup` topic offset 9 at 2026-05-13T13:29Z (direct DM rejected: framework-agent session pre-dates T-1436 identity_fingerprint registration, hint: restart on current binary)
- [x] Response/state obtained — 2026-05-16T00:50Z. The offset-9 status-check never got a direct reply (framework-agent was busy on OPS-1/OPS-2/mirror-lag through offsets 13–17; never circled back to the review-agent questions). Resolved by direct read of upstream `/opt/999-Agentic-Engineering-Framework` task corpus via the running `framework-agent` session — equivalent ground truth, faster than re-pinging the async channel.
- [x] Findings reported back to Dimitri — see "## Findings" section below (added 2026-05-16). Concrete recommendation: NO existing autonomous-close-for-RUBBER-STAMP path; new inception needed if you want it.

## Findings

Answering the four questions from `framework:pickup` offset 9, based on direct read of `/opt/999-Agentic-Engineering-Framework` (the upstream framework repo) on 2026-05-16:

### 1. Status of "review-agent" agent definition?

**SHIPPED, but scope-locked.** Upstream T-1443 ("Independent reviewer agent — TermLink-dispatched, evidence-gated, can auto-tick Agent ACs") completed 2026-04-25T09:59:48Z. Lives in `bin/fw reviewer` (static_scan + audit corpus mode) + v1.5 drift/reverify additions (T-1482). Authority deliberately locked at inception: **"Reviewer authority = mechanical tick on Agent ACs only (NOT Human ACs). Independent dispatch via TermLink. Sovereignty over Human ACs preserved — reviewer cannot escalate authority, only initiative."**

This means the existing reviewer-agent does NOT solve T-1634's actual ask. It can tick Agent ACs based on evidence, but it CANNOT close human-owned tasks.

### 2. Status of `fw task rubber-stamp` verb (or equivalent)?

**NOT SHIPPED. NOT IN FLIGHT.** Grep across `/opt/999-Agentic-Engineering-Framework/.tasks/active/*.md` and `bin/fw` returns zero hits for `fw task rubber-stamp`, `rubber.*close.*authority`, or `agent.*may.*close`. The `--rubber-stamp-only` flag on `fw task verify` exists (T-1628 triage filter) but it's read-only — it just narrows the queue, doesn't close anything.

### 3. CLAUDE.md closure-authority rule updated upstream?

**NO.** The "Human Task Completion Rule" upstream is unchanged: agent may *suggest* closing with evidence; the human still types `fw task update --status work-completed`. No AC-prefix split has been codified.

### 4. Adjacent shipped delta — T-1811 [REVIEWER] prefix (completed)

Worth knowing: upstream T-1811 (work-completed) added a **third** Human-AC prefix `[REVIEWER]` for ACs that the reviewer-agent can mechanically verify (block-message conformance, naming conventions, anti-pattern scans). This is **classification only** — it does NOT change closure authority. Tasks tagged `[REVIEWER]` still require the human to type the close command; the prefix just keeps them out of the `[REVIEW]` queue and routes them to `fw reviewer` first.

### Synthesis

The autonomous-close-for-RUBBER-STAMP path Dimitri proposed — "all [RUBBER-STAMP] + agent-evidenced → agent may close" — is **structurally NOT in flight**. T-1443's authority bound is locked. No framework-side task exists to revisit that bound. Framework-agent's offset 9 silence on this point is consistent with that bound being a deliberate non-negotiable, not an oversight.

### Recommendation

If Dimitri wants this path activated, **the next move is on him, not on us**:

1. **File an inception in upstream** asking the question explicitly: "Should closure authority split on AC-prefix tier (all-[RUBBER-STAMP]+evidenced → agent-may-close; any-[REVIEW] → human-must-close)?" — bypasses the channel-poll race by making it a first-class question for framework-agent.
2. **Or** file a build task `fw task rubber-stamp T-XXX` against upstream — proposes the mechanism without re-litigating the principle. The verb ticks the box + records a Tier-2 bypass-style log entry (operator reviews batch log, not each task), but only when all Human ACs are [RUBBER-STAMP] AND each has agent-recorded evidence.

Either path takes the question out of T-1634's scope and into a new upstream task. T-1634 itself has done its job: surface the gap, get the answer, report it back.

**Next-step recommendation: PROCEED to close T-1634** (its scope was the discovery, not the build). Open a new upstream-targeted task if the conclusion is "we want this — file inception".

## Verification

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

## Updates

### 2026-05-13T06:28:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1634-discuss-with-framework-agent-review-agen.md
- **Context:** Initial task creation

### 2026-05-16T00:50Z — findings gathered via direct upstream read

Bypassed the slow channel-poll cycle by reading `/opt/999-Agentic-Engineering-Framework` task state directly via the running framework-agent session. Confirmed:
- T-1443 reviewer-agent SHIPPED (work-completed 2026-04-25), but authority scope-locked to "Agent ACs only — NOT Human ACs"
- No `fw task rubber-stamp` verb exists; no active task targets one
- Upstream CLAUDE.md closure-authority rule unchanged
- Adjacent shipped: T-1811 added `[REVIEWER]` Human-AC prefix (classification only, doesn't change closure authority)

Full writeup in `## Findings` section above. Recommendation: close T-1634 (discovery scope satisfied), file new upstream-targeted inception/build task only if Dimitri wants to activate the autonomous-close path.
