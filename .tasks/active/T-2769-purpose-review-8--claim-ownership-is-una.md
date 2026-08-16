---
id: T-2769
name: "Purpose review 8 — claim ownership is unauthenticated, and identity is host-scoped not agent-scoped"
description: >
  Charter verb 3 'claim work' rests on exactly-one-owner (PL-262), but the hub enforces it by comparing two CALLER-SUPPLIED strings. Identity is also host-scoped while three of four charter verbs are agent-scoped.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-16T16:38:41Z
last_update: 2026-08-16T16:38:41Z
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

# T-2769: Purpose review 8 — claim ownership is unauthenticated, and identity is host-scoped not agent-scoped

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

- **IW-1: Bind `claimer` to the authenticated sender identity, or give claims their own signature?**
  confidence: 2
  disposition:
  rationale: `channel.post` already proves the cheaper option works — it rejects a
  `sender_id` that does not match the fingerprint derived from `sender_pubkey_hex`
  (channel.rs:777-787). Reusing that check in the claim path is the smallest change.
  The alternative (sign the claim itself) is stronger but duplicates a mechanism that
  already exists one function away.

- **IW-2: What breaks in the field, and what is the migration?**
  confidence: 1
  disposition:
  rationale: THE decisive question. Today `claimer` is a free string — the CLI resolves
  it from `$TERMLINK_AGENT_ID` or `be-reachable.state` (T-1857), so real callers pass
  human-readable ids like `ring20-concierge`, NOT identity fingerprints. Binding
  `claimer` to the authenticated identity would reject **every current caller** on a
  fleet with ~1000-commit-stale hosts. This is the T-2700 shape exactly: a correct
  protection that begins refusing live peers. Needs a compatibility path (accept-and-warn
  before enforce?) and probably a human decision on the cutover.

- **IW-3: Does the documented orchestrator pattern survive an authenticated `claimer`?**
  confidence: 1
  disposition:
  rationale: `docs/operations/substrate-orchestrator-recipe.md` has one agent claim and
  then `claim-transfer` to a different worker. `claim-transfer` requires `by == claimed_by`,
  so the orchestrator's own identity must match — fine. But `to_owner` is then a
  *different* agent's id, and that worker later renews/releases with its own `claimer`.
  If claimer must equal the authenticated sender, `to_owner` has to be that worker's
  identity fingerprint rather than a free label, which changes the recipe's ergonomics
  and every caller that passes a friendly name. Must be answered before F1 is built, or
  the fix silently breaks the substrate's flagship pattern.

- **IW-4: Is F1 even reachable by an untrusted party, and does that change its severity?**
  confidence: 2
  disposition:
  rationale: Claim RPCs require the fleet HMAC secret, so this is not an anonymous
  internet-facing hole — the realistic threat is a buggy or confused co-tenant agent,
  not an attacker. That lowers urgency but does NOT dissolve the finding: F2 shows
  co-located agents already share one identity, so "confused co-tenant" is the *normal*
  configuration here, not a corner case. Severity should be argued explicitly rather
  than assumed high because the word "unauthenticated" appears.

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

**Rationale:** Eighth critical pass; prior axes were breadth (T-2468), non-goal guards (T-2678), guard execution (T-2683), Usability+Portability (T-2690), positive claims vs provers (T-2694), refusal taxonomy (T-2698), architecture invariants (T-2702). None examined AUTHORIZATION of the substrate primitives. Two findings. F1: channel.claim/release/renew/claim-transfer read claimer via param_str(params,'claimer') with no signature, pubkey or fingerprint anywhere in the claim path, so CLAIM_NOT_OWNED (-32017) compares two caller-supplied strings and stops only honest mistakes. The same file proves the standard exists: channel.post line 777 rejects a sender_id that does not match the fingerprint derived from sender_pubkey_hex, with in-code rationale naming the exact attack ('own key but claim any sender_id'). So the hub authenticates messages but not work ownership, and PL-262's disjointness property rests on client honesty. F2: measured live today, two agents on .107 (/opt/termlink and /opt/050-email-archive) share identity fingerprint d1993c2c3ec44c94, so their DMs land in one topic and receipts cross-mark; identity is host-scoped while discover/exchange/claim are agent-scoped. GO on F1 only: it mirrors an existing in-file check, is bounded and testable. F2 changes wire identity semantics for every peer and would re-key existing dm topics on a fleet with ~1000-commit-stale hosts, so it is human-sovereign.

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
