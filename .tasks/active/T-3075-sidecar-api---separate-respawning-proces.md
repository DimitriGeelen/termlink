---
id: T-3075
name: "Sidecar API - separate respawning process independent of the hub"
description: >
  Inception: Sidecar API - separate respawning process independent of the hub

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [arc:arc-011]
components:
  - scripts/notify-sidecar-api.sh
  - scripts/notify-sidecar.sh
  - scripts/notify-injector.sh
  - tests/notify-sidecar-api-fixtures.sh
  - docs/design/arc-011-sidecar-api-architecture.md
related_tasks: []
created: 2026-09-22T14:24:30Z
last_update: 2026-09-24T20:29:06Z
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
  - ts: '2026-09-22T14:26:42Z'
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
  - ts: '2026-09-22T14:57:32Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (target_blast_radius:inception-T-2189); tier=4 
      (workflow:inception); effort=7 (lines=167,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3075: Sidecar API - separate respawning process independent of the hub

## Problem Statement

The operator specified a "sidecar API — a separate, always-respawning process,
independent of the hub, because the hub goes down" for arc-011 slices S1
(sender-side sidecar API) and S12 (role-swap reply). Read literally that
collides with TermLink's charter (hub-mediated strict star,
`no_federation_tripwire.rs`), so before any build the question is: what can
that API carry without becoming a second bus, and does the surrounding
orchestration (queue, priority, inject, verify) even belong in this repo?
Full analysis: `docs/design/arc-011-sidecar-api-architecture.md`. Answered so
far via operator resolution (IW-1/IW-2, citing SQ-1) — see Open Questions.

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

- **IW-1: Does a sidecar API violate TermLink's charter, or not?**
  The charter's load-bearing noun is "**hub-mediated** — a strict star: spokes never
  talk peer-to-peer, the hub mediates all coordination", enforced by
  `no_federation_tripwire.rs` (T-2569). The operator wants an API explicitly
  independent of the hub *because the hub goes down*. Answer depends entirely on
  what the API CARRIES: agent-to-agent DELIVERY (contradicts) vs LOCAL control of
  this host's own mailbox and prompt (does not — every inter-agent hop still goes
  through the hub).
  confidence: 3
  disposition: answered
  rationale: `docs/design/arc-011-sidecar-api-architecture.md` §6 draws the bright
    line (may read/act on this host's OWN state; may never move a message
    *between* hosts) and SQ-1's resolution (arc-011.yaml, operator, 2026-09-23:
    "the injector is a PRIMITIVE built over TermLink's own session control, not
    an orchestration engine") confirms the LOCAL-CONTROL reading is the one the
    operator approved by keeping this work in TermLink — building a second bus
    here would be incompatible with that same decision. Conditional on staying
    inside the §6 bright line; a tripwire test enforcing it (mirroring
    `no_federation_tripwire.rs`) is a build-time item, not an inception gate.

- **IW-2: Does this functionality belong to TermLink or to AEF?**
  Charter non-goal 4: "**Not a workflow or orchestration engine** — TermLink
  provides the primitives." But queue → check-prompt-free → inject → verify-working
  IS orchestration. If that is true, the injector belongs to AEF and TermLink
  supplies only the primitives it already has (durable topics, artifact.put, PTY
  inject, presence). Getting this wrong builds the right thing in the wrong repo.
  confidence: 3
  disposition: answered
  rationale: RESOLVED by the operator 2026-09-23 as SQ-1 in
    `.context/arcs/arc-011.yaml` — the injector stays in TermLink, reframed as a
    primitive rather than an orchestration engine, because it "already lives
    here, is proven end to end (T-3069), and moving it would put the PTY
    classifier and the hub read on opposite sides of a project boundary."
    Recorded here verbatim, not re-decided by this agent.

- **IW-3: What is the failure model the API is actually buying?**
  "The hub goes down" is the stated reason. But the hub going down does not stop
  the sidecar reading its own journal or injecting into a local PTY — those are
  already local. So what does an API add that a local process reading local state
  does not? Candidate answers: cross-host delivery when the hub is down (needs
  peer-to-peer, contradicts IW-1), or a stable control surface for the startup
  chain. These have very different scopes.
  confidence: 2
  disposition: answered
  rationale: Design-doc §3 (steelman) gives the defensible answer under the
    now-confirmed local-control reading: today's poller is opaque (liveness only
    inferred from a heartbeat-file mtime); a local API turns it into something
    addressable ("what's in my queue", "inject this now", "are you alive") —
    which keeps answering during a hub outage precisely because none of those
    questions crosses the host boundary. This is analysis, not a fresh operator
    decision, so confidence is capped at 2 rather than 3 — nothing has tested it
    against a real outage yet. Not reopened as a second Sovereign question: it
    describes what the already-approved local-control scope buys, it does not
    ask for a new scope.

- **IW-4: Who respawns the respawner, portably?**
  systemd answers it on .107 and not elsewhere. D4 Portability (weight 3) is the
  lowest-weighted directive, so a systemd-only answer may be acceptable — but that
  should be a decision, not a default.
  confidence: 1
  disposition: deferred
  rationale: GENUINELY OPEN — the one item on this task with no existing
    operator ruling anywhere (arc-011.yaml carries no SQ for it). Surfaced to
    the human as a Sovereign question rather than defaulted: "systemd-only
    respawn is D4-acceptable (lowest-weighted directive) given the current
    single-host (.107) deployment — confirm, or require a portable fallback
    before build?" Deferring rather than dissolving because it gates a real
    build decision (T-3075's own S1/S12 build task would need to pick one),
    not because it is moot.

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN (local-control reading, confirmed by SQ-1):** a local API answering
`status` / `queue` / `inject <id>` / `agent-state` / `ack <offset>` about
*this host's own* mailbox and prompt; an always-respawning supervisor for it;
re-resolving this host's own FQDN/IP on change.

**OUT:** anything that moves a message *between* hosts (that is `channel.post`
via the hub, full stop); the respawn mechanism's portability (IW-4, open);
writing any of this code before a GO decision (Inception Discipline).
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

GO on the ANALYSIS, not a build. (Updated 2026-09-24.) IW-2 — TermLink or AEF —
is now RESOLVED (SQ-1, arc-011.yaml, operator, 2026-09-23): the injector stays
in TermLink as a primitive, not an orchestration engine. That resolution
carries IW-1: the operator kept this work in TermLink specifically under a
"primitive, not orchestration engine" framing, which is only charter-consistent
under the LOCAL-CONTROL reading described in
`docs/design/arc-011-sidecar-api-architecture.md` §6 — so IW-1 is answered
(no charter violation, conditional on staying inside the §6 bright line, which
a future tripwire test should enforce rather than merely document). IW-3 has a
defensible analytical answer in the same doc §3. **IW-4 (portable respawn) is
the one item with no existing operator ruling anywhere** — surfaced above as a
live Sovereign question, not decided here. This task is ready for
`fw inception decide T-3075 go` once the human either answers IW-4 or accepts
it as a build-time detail to settle inside the follow-up build task; this
agent cannot invoke that verb itself (Tier-0, confirmed by the gate refusing
even `fw inception decide --help`).

**Evidence:**
- `docs/design/arc-011-sidecar-api-architecture.md` §1-§9 — full charter/AEF
  analysis, written 2026-09-22, re-checked against SQ-1's resolution today.
- `.context/arcs/arc-011.yaml` SQ-1 — operator resolution text, verbatim.
- `crates/termlink-hub/tests/no_federation_tripwire.rs` (T-2569) — the existing
  enforcement mechanism a future local-control tripwire would mirror.

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

### 2026-09-22T14:26:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-22T14:27:42Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-011
