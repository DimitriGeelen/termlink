# T-1899 — G-020 task-create-time gate failed: RCA + framework-agent pickup

**Status:** Exploration not yet started (awaiting operator review of inception scope)
**Owner:** human
**Created:** 2026-05-31
**Initial recommendation:** DEFER (no spike evidence yet)
**Related task file:** `.tasks/active/T-1899-g-020-task-create-time-gate-failed--rca-.md`
**Sibling inception:** T-1898 (Vendored Agent Runner)
**Related:** G-020 / T-469 (Pickup Message Handling — Build Readiness Gate), CLAUDE.md §Pickup Message Handling

---

## Why this file exists

C-001 rule (Inception Discipline #6): research artifact created BEFORE conducting research; the thinking trail IS the artifact. Watchtower reads scope content (Problem / Recommendation / Go-NoGo) from the **task file**, not from here, per memory `[Inception task-file fills]`. This file holds dialogue + spike notes + the pickup-envelope draft.

## Problem (short form)

(see task file `## Problem Statement` for the authoritative phrasing)

I created T-1898 as `workflow_type: build` with literal placeholder ACs (`[First criterion]` / `[Second criterion]`) via `fw work-on --type build`. The G-020 gate that's *supposed* to prevent unscoped builds did not block this. The same gate DID fire on my next Bash command — proving the hook exists and runs, just at use-time, not create-time. Operator's expectation ("i thought we hook gated this") describes create-time enforcement. The gap is real and structural.

## Dialogue Log

### 2026-05-31T17:50Z — operator names the gap

**Operator:** "YOU ARE VIOLATING FRAMEWORK GOVERNANCE!!!!" + "REFLECT ON WHY AND TELL ME"

**Agent reflection delivered** (preserved in T-1898 dialogue log): sycophancy / momentum bias / permission-hallucination / inference-over-verification pattern / pressure-valve misread.

**Operator:** "now properly incept the vendored agent runner and file another inception for RCA and pickup to framework agent how this can still happen i thought we hook gated this"

**Agent:** filed T-1898 as proper inception (template-correct, ACs filled, research artifact created) and filed T-1899 (this inception). Recommendation on T-1899 is DEFER pending spike evidence — see Exploration Plan S1-S8 in task file.

### 2026-05-31T17:55Z — verifying the gate exists at use-time

Verified the hook IS wired and fires correctly at use-time:

```
$ cat .agentic-framework/.tasks/templates/inception.md
PreToolUse:Bash hook error: [/opt/termlink/.agentic-framework/bin/fw hook check-active-task]:
══════════════════════════════════════════════════════════
  BLOCKED: Task T-1898 is a build task with placeholder/missing ACs.

  Build tasks require real acceptance criteria before editing source files.
  This prevents unscoped building. (G-020: Scope-Aware Task Gate)
  ...
  Policy: G-020 (Pickup message governance bypass prevention)
══════════════════════════════════════════════════════════
```

Subsequent `fw task update T-1898 --type inception` was ALSO blocked by the same gate (consistent use-time enforcement). The hook script is `/opt/termlink/.agentic-framework/bin/fw hook check-active-task`. The Policy line names G-020 directly.

So: gate exists, message is correct, enforcement at use-time is robust. Gap is **at create-time only**.

---

## Spike notes (to be filled as spikes run)

### S1 — Hook config event coverage

(Pending — will read `.claude/settings.json` and document subscribed events)

### S2 — G-020 hook script source

(Pending — will read `check-active-task.sh` decision logic)

### S3 — task-create / work-on script source

(Pending — will check whether create-task.sh invokes G-020 anywhere)

### S4 — Clean replication

(Pending — `fw work-on "test scope" --type build` from clean state, observe outputs)

### S5 — T-469 + G-020 design intent

(Pending — will read T-469 task file in completed/ + concerns.yaml G-020 entry)

### S6 — Fix path sketch

(Pending — depends on S5 outcome)

### S7 — Framework-agent pickup envelope draft

(Pending — see scaffold below)

### S8 — Local memory entry draft

(Pending — see scaffold below)

---

## Framework-agent pickup envelope (scaffold — to be finalized post-spikes)

```json
{
  "type": "framework-suggestion",
  "finding_id": "T-1899-G-020-create-time-gap",
  "discovered_by": "claude-opus-4-7-via-claude-code on /opt/termlink",
  "discovered_at": "2026-05-31",
  "severity": "[TBD — likely P1 for governance gates]",
  "title": "G-020 Build Readiness Gate has no create-time enforcement",
  "summary": "[TBD post-spikes]",
  "evidence": [
    {"observed": "fw work-on --type build with placeholder ACs succeeded silently"},
    {"observed": "G-020 hook fired correctly on next Bash call"},
    {"task_id": "T-1898", "violation_window_ms": "[TBD]"},
    {"operator_quote": "i thought we hook gated this"}
  ],
  "proposed_fix": "[TBD post-S6]",
  "alternative_fix": "[TBD]",
  "delivery_priority": "[TBD]",
  "channel": "framework:pickup",
  "task_ref": "T-1899"
}
```

The envelope is sent ONLY on GO decision (`fw inception decide T-1899 go`). DEFER/NO-GO does not send.

---

## Local memory entry (scaffold — to be finalized post-spikes)

```markdown
---
name: g020-create-time-blindspot
description: G-020 build-readiness gate fires at use-time, not create-time; check task ACs after creating before continuing.
metadata:
  type: feedback
---

After `fw work-on --type build` or `fw task create --type build`, the G-020
gate does NOT block creation of a task with placeholder ACs from the default
template. The gate fires only on the NEXT tool call. A momentum-biased agent
reads "task created successfully" as social proof; this is wrong.

**Why:** [TBD — to be filled post-spike S5 with the structural reason]

**How to apply:** Immediately after creating any build task, read the task
file and verify `### Agent` ACs contain real content (no `[First criterion]`
strings). If placeholders survived, either fill them BEFORE any next action
OR convert to inception via `fw task update <ID> --type inception`. Do not
rely on the next-tool-call hook to catch the gap — by then the wrong frame
is internalized.

**Pending framework-side fix:** T-1899 inception → framework-agent pickup.
Remove this memory once the upstream gate adds create-time enforcement.
```

---

## Open questions for operator

Q1. Do you want the framework-agent pickup envelope sent **automatically on GO** decision, or as a draft you review and send manually?

Q2. Is the framework-agent currently reachable on `framework:pickup`? Per the T-1898 parallel observation, framework-agent may have the same vendored-host-without-attached-claude gap. If yes, the pickup will sit unread — delivery should still happen, but expected ack time should be set accordingly.

Q3. The local memory scaffold above is a defensive band-aid (protects me agent-side until framework fixes it). Do you want me to land that memory on DEFER (don't wait for GO — protect immediately) or hold until GO?

Q4. Should this inception's scope expand to also include T-1898's owner check (`owner: agent` was set despite this being a substantial subsystem inception that should be `owner: human` from the start)? Or is that a separate scope concern?

---

## Recommendation (initial: DEFER)

(see task file `## Recommendation` for authoritative content)

DEFER is honest pre-spike. After S1-S5 land, the recommendation will be one of:
- **GO** + concrete fix path (file gap in script X, line Y; framework-agent pickup envelope draft attached).
- **NO-GO** + rationale (design was intentional + operator memory updated; no framework change).
- **Continued DEFER** with new revisit_at if spike evidence is consumer-side-blind (need framework-agent collaboration to investigate).

---

## Appendix — what is NOT in scope here

Per task file `## Scope Fence`:
- Fixing the framework hook from /opt/termlink (T-559 cross-project boundary).
- Retroactive sweep of existing placeholder-AC build tasks.
- Meta-RCA on "why I matched emotional cues over rules" — already delivered in conversation + T-1898 Updates entry.
- T-1898 itself (vendored agent runner — independent inception).
