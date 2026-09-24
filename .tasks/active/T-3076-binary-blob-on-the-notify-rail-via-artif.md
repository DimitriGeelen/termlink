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
last_update: 2026-09-24T20:25:41Z
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

arc-011 slice S2 ("payload may carry a binary blob") requires the notify rail
(a shell script, `scripts/notify-sidecar.sh`) to move artifact BYTES over
`termlink channel post`, not merely a reference. Measured (IW-1, re-confirmed
2026-09-24): `send_artifact_via_client`/`download_artifact_via_client`
(`crates/termlink-session/src/artifact.rs:137,525`) are the only functions
that move the bytes, and their only callers are `commands/file.rs` (`file
send`/`file receive`), `commands/remote.rs`, and the MCP tool layer — no
`termlink artifact` CLI subcommand exists. A shell script can attach
`--artifact-ref` but cannot produce it. Full writeup:
`docs/reports/T-3076-artifact-cli-verbs-inception.md`.

## Assumptions

- **A-049** (validated 2026-09-24): `send_artifact_via_client` /
  `download_artifact_via_client` are safe to call from a 4th site (CLI
  `artifact put`/`get`) without modification. Evidence: both take only
  generic parameters (`Client`, peer/target string, payload/sha256,
  manifest, identity, cache, ctx) — no session-specific coupling found on
  direct read of `artifact.rs:130-175` and `:525-545`.

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
  confidence: 3
  disposition: answered
  rationale: RESOLVED by the operator 2026-09-23, recorded in
    `.context/arcs/arc-011.yaml` SQ-3 — "YES, add the artifact CLI verbs".
    Recorded here verbatim, not re-decided: this agent hit the Tier-0 gate
    directly attempting `fw inception decide --help` (2026-09-24), confirming
    the decide-verb itself is human-only. This entry only transcribes the
    already-made Sovereign decision so the human's `fw inception decide`
    pass is a one-line confirmation, not a re-litigation.

- **IW-3: eager or lazy fetch, and is `--expected-sha256` mandatory?**
  Deferred behind IW-2 — the shape of the fetch depends on whether there is a verb
  to fetch with. My position: mandatory, because `artifact_ref` IS a sha256 so
  verification is free, and T-2472 already established that without it the output
  must not claim "verified".
  confidence: 3
  disposition: answered
  rationale: IW-2 resolved YES, so the deferral clears. `artifact_ref` IS the
    sha256 (confirmed at `artifact.rs:525` — `download_artifact_via_client`
    takes `sha256: &str` directly), so verification is free; making
    `--expected-sha256` mandatory rather than optional on the future `get`
    verb costs nothing and pre-empts a T-2472-class "output claims verified
    but wasn't" gap. This is a build-time detail, not a second Sovereign
    question — no operator counter-signal exists.

## Exploration Plan

Single spike, time-boxed to reading (no code written — Inception Discipline
forbids build artifacts pre-GO):
1. Grep confirmed IW-1's claim still holds (2026-09-24 re-check: still true —
   `send_artifact_via_client`/`download_artifact_via_client` have exactly 3
   caller sites, none of them a bare CLI verb).
2. Read both functions' signatures to test A-049 (do they need anything
   session-specific that would block a 4th, CLI-only caller?). Validated:
   no — both take only generic params.
3. Confirmed no CLI name collision (`grep '"artifact"' cli.rs` → empty).
Findings written to `docs/reports/T-3076-artifact-cli-verbs-inception.md`.

## Technical Constraints

None found beyond what governs the existing `file send`/`file receive`
callers: the hub must advertise `artifact.put`/`artifact.get`
(capability-gated fallback to legacy `file.*` events already exists and is
reused, not reimplemented) and `download_artifact_via_client` already caps
in-memory accumulation at `max_artifact_download_bytes()` (no new unbounded
sink — relevant to the T-2531 drain-sink convention this repo enforces).

## Scope Fence

**IN:** two thin CLI wrapper subcommands, `termlink artifact put <path> --to
<peer>` and `termlink artifact get <sha256> --expected-sha256 <sha256> -o
<path>`, calling the existing `send_artifact_via_client` /
`download_artifact_via_client` with no changes to those functions.
`--expected-sha256` mandatory on `get` (IW-3).

**OUT:** any change to the artifact.put/get protocol methods, the hub-side
router, or the capability-fallback logic; MCP tool changes (already covered);
the notify-rail's own consumption of the new verb (that is arc-011 S2's build
task, filed separately on GO, not this inception).

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

**Recommendation:** GO. (Updated 2026-09-24: IW-2, the sovereign call this
recommendation was parked on, is now RESOLVED — operator decided YES,
`.context/arcs/arc-011.yaml` SQ-3, 2026-09-23. IW-3's mandatory
`--expected-sha256` follows directly. This task is ready for
`fw inception decide T-3076 go` — Tier-0, human-only; this agent confirmed
the gate directly by attempting `fw inception decide --help`, which the
Tier-0 hook refused. Nothing found today changes the recommendation below.)

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
