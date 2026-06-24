---
id: T-1888
name: "Investigate T-1431 e2e regression — dm:handoff-rubber* missing on chat-arc"
description: >
  T-1884 S2 ran T-1431's [RUBBER-STAMP] Human AC Step 5: termlink channel list --prefix dm: | grep handoff-rubber — exit=1, no match. T-1431's AC asserts the skill works end-to-end. Either (a) the evidence smoke from 2026-05-30 was hub-local and dm topics were cleaned up since, or (b) the /agent-handoff skill regressed. Diagnose which, fix or refresh evidence accordingly. Source: docs/reports/T-1884-S2-results.md.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [bug]
components: []
related_tasks: [T-1431, T-1884]
created: 2026-05-30T22:00:08Z
last_update: 2026-05-31T07:04:44Z
date_finished: 2026-05-31T07:04:44Z
---

# T-1888: Investigate T-1431 e2e regression — dm:handoff-rubber* missing on chat-arc

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Determine current state of `dm:handoff-rubber*` topics on local hub — confirmed: zero matches (topics are fp-keyed, never name-keyed)
- [x] Determine same state on each remote hub in `~/.termlink/hubs.toml` — N/A, local fp-keyed topic `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` exists with 11 envelopes, federation probe redundant
- [x] Classify root cause: (d) other — verification AC misspecification, NOT skill regression
- [x] Skill is healthy. Fixed T-1431 Step 5 with correct pattern + inline note explaining DM topic naming convention
- [x] Documented finding in `## RCA`; T-1431 inline-references T-1888 as the fix source

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

**Symptom:** T-1884 S2 ran T-1431's [RUBBER-STAMP] Human AC Step 5 (`termlink channel list --prefix dm: | grep handoff-rubber`) — exit 1, no match. Looked like e2e regression.

**Root cause:** Step 5's grep pattern was misspecified. DM topics in TermLink are named `dm:<self-fp>:<peer-fp>` using 16-hex fingerprint pairs derived from identity keys — they NEVER include the friendly peer name (`handoff-rubber-stamp`). The actual topic created by the smoke is `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` (self-DM on shared host .107 — same host key for self and registered peer). Topic exists with 11 envelopes; skill works. Investigated and confirmed via `termlink channel info` showing count=11, single sender_id=d1993c2c3ec44c94.

**Why structurally allowed:** No validator existed that runs Human AC Steps against current state and reports drift between Steps and reality. T-1884's S2 was the first such pass; it caught this on the first run. The misspecified Step survived since 2026-05-01 because the AC was never executed against current topic-naming conventions — the operator would have noticed only at rubber-stamp time.

**Prevention:** T-1885 (`fw independent-review`) v0.1 orchestrator is the structural prevention — it runs Step commands against current state for every unchecked Human AC, surfacing misspec-vs-regression confusion as INCONCLUSIVE/FAIL. This task is the prevention case that proves the orchestrator's value: a quality issue in the AC, not the skill, found by independent re-verification. Apply: ACs whose Steps reference topic names should always use the fingerprint-pair pattern, never the friendly name. Documented as a one-line note inline in T-1431 Step 5.

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-30T22:00:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1888-investigate-t-1431-e2e-regression--dmhan.md
- **Context:** Initial task creation

### 2026-05-31T00:30Z — investigation-complete [agent autonomous]
- **AC 1 (local):** `termlink channel list --prefix dm:` shows no `handoff-rubber` topic; correct fp-keyed self-DM topic `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` exists with `[forever]` retention.
- **AC 2 (remote):** No need to probe remote hubs — local hub has the canonical fp-keyed topic with 11 envelopes (count=11 via `channel info`). Confirms the smoke landed and persists. The verification step on remote was redundant; root cause is local AC pattern, not federation.
- **AC 3 (classify):** Root cause = (d) other — verification AC misspecification. NOT a skill regression. Skill works correctly: `delivered.offset` returned, topic carries envelopes.
- **AC 4 (fix path):** Refreshed T-1431 Step 5 with the correct pattern + inline note explaining DM topic naming. No follow-up fix task needed (skill is healthy).
- **AC 5 (RCA):** Documented above. T-1885 v0.1 (`fw independent-review`) is the structural prevention.
- **Cross-reference:** This investigation is the canonical proof-of-value for T-1885 — the orchestrator surfaced a quality bug in the AC itself, which would have wasted operator time at rubber-stamp click.

### 2026-05-31T07:02:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7ec98833
- **Timestamp:** 2026-05-31T07:04:45Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — Determine same state on each remote hub in `~/.termlink/hubs.toml` — N/A, local fp-keyed topic `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` exists with 11 envelopes, federation probe redundant
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/hubs.toml in: Determine same state on each remote hub in `~/.termlink/hubs.toml` — N/A, local fp-keyed topic `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` exists with 11 e`

### 2026-05-31T07:04:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
