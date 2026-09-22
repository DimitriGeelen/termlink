---
id: T-3076
name: "Binary blob on the notify rail via artifact.put"
description: >
  Inception: Binary blob on the notify rail via artifact.put

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-011]
components:
  - scripts/notify-sidecar.sh
  - scripts/journal-mirror.sh
  - tests/notify-blob-fixtures.sh
related_tasks: []
created: 2026-09-22T14:25:34Z
last_update: 2026-09-22T15:02:29Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-09-22T14:57:19Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-22T14:57:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 6
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=6 (lines=123,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3076: Binary blob on the notify rail via artifact.put

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

- **IW-1: Can a SHELL SCRIPT put and get artifact bytes, or only reference them?**
  MEASURED, and this is the whole finding. `termlink channel post --artifact-ref
  <ref>` exists, so the rail CAN attach a pointer to a topic message. But there is
  **no `termlink artifact` subcommand at all** — `artifact.put` / `artifact.get`
  are protocol methods (`control.rs:316,323`), routed by the hub
  (`router.rs:143,146`), and wrapped by library functions
  (`send_artifact_via_client`, `download_artifact_via_client` —
  `artifact.rs:137,525`) whose ONLY CLI caller is `commands/file.rs`, i.e. the
  session-targeted `file send` / `file receive`. So a shell script can carry the
  REFERENCE and cannot move the BYTES.
  confidence: 3
  disposition: answered
  rationale: grep of ARTIFACT_PUT/ARTIFACT_GET callers + `channel post --help`

- **IW-2: Should TermLink grow `artifact put` / `artifact get` CLI verbs?**
  THE SOVEREIGN QUESTION, and not mine. It is new CLI surface, which this project
  prunes aggressively (P4 deleted 52 tools). FOR: it is a PRIMITIVE, not
  orchestration — squarely TermLink's job under charter non-goal 4 — and both
  verbs are thin wrappers over functions that already exist and are already
  exercised by `file send`. AGAINST: every new verb is surface that must be
  justified, and `file send` already covers the session-addressed case.
  confidence: 2
  disposition:
  rationale:

- **IW-3: eager or lazy fetch, and is `--expected-sha256` mandatory?**
  Deferred behind IW-2 — the shape of the fetch depends on whether there is a verb
  to fetch with. My position: mandatory, because `artifact_ref` IS a sha256 so
  verification is free, and T-2472 already established that without it the output
  must not claim "verified".
  confidence: 2
  disposition:
  rationale:

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

**Recommendation:** GO — but the GO is on two thin CLI verbs, and that is a
sovereign call (IW-2), so this task is PARKED awaiting it rather than closed.

**Rationale:**

My first answer ("TermLink's blob transport does not drop in") was wrong and the
correction is now measured twice over. Blob-on-a-topic already exists end to end:
`artifact.put` stores content-addressed bytes, `channel.post --artifact-ref
<sha256>` attaches the pointer to a topic message, `artifact.get` fetches them
back, and the hub routes all three. None of that needs building.

What is missing is narrower and more mundane than a design problem: **there is no
`termlink artifact` CLI subcommand**, so a shell script — which is what this whole
rail is made of — can post the reference but cannot move the bytes. The
put/get functions are reachable only from inside `file send` / `file receive`,
which are session-addressed and require the receiver to be actively listening.

So the remediation is two thin wrappers over `send_artifact_via_client` and
`download_artifact_via_client`, both already exercised by `file send`. Small, and
a primitive rather than orchestration — which puts it on TermLink's side of
charter non-goal 4 rather than AEF's.

**Evidence:**
- `crates/termlink-protocol/src/control.rs:316,323` — ARTIFACT_PUT / ARTIFACT_GET
- `crates/termlink-hub/src/router.rs:143,146` — both routed
- `crates/termlink-session/src/artifact.rs:137,525` — send/download wrappers
- `grep -rn 'download_artifact_via_client' crates/termlink-cli/src` → only file.rs
- `termlink artifact --help` → `unrecognized subcommand`
- `termlink channel post --help` → `--artifact-ref <ARTIFACT_REF>` present


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

### 2026-09-22T14:27:43Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-011

### 2026-09-22T15:00:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
