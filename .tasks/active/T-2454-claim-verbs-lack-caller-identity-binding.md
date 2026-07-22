---
id: T-2454
name: "claim verbs lack caller-identity binding — ownership on spoofable claimer string (round-12 HIGH)"
description: >
  Inception: claim verbs lack caller-identity binding — ownership on spoofable claimer string (round-12 HIGH)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-22T09:54:58Z
last_update: 2026-07-22T09:56:57Z
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

# T-2454: claim verbs lack caller-identity binding — ownership on spoofable claimer string (round-12 HIGH)

## Problem Statement

TermLink's coordination substrate (claim primitive #1) promises **exclusive
ownership**: at most one claimer owns a `(topic, offset)` work unit at a time.
That guarantee is what lets an AEF orchestrator fan work across N workers without
two workers processing the same unit. Round-12 adversarial review found the
invariant is enforced only against a **spoofable, world-readable `claimer`
string** — the five claim verbs (`channel.claim/renew/release/claim_transfer/
claim_force_release`) read `claimer`/`by`/`to_owner` straight from JSON params
and compare them to the stored `claimed_by`, with NO caller-identity binding.
By contrast `channel.post` binds identity cryptographically (T-1427:
`sender_id == fingerprint(sender_pubkey_hex)` verified against the payload
signature). The comms half of the substrate is identity-bound; the coordination
half is not.

**Who / why now:** the substrate now underpins the AEF parallel-orchestrator
arc (arc-011) — the exact consumer whose correctness depends on this invariant.
A misbehaving OR buggy orchestrator that passes the wrong `claimer` can release
or steal another worker's live claim, producing silent double-work.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->
- The trusted-mesh threat model treats all token/Unix-scope holders as
  semi-trusted, so this is primarily an **accountability + accidental-double-work**
  hardening, not an external-attacker breach (tempers severity from HIGH toward
  MED under that model — a key thing the decision must weigh).
- The correct binding mechanism is the T-1427 signature pattern applied to claim
  params, because the connection layer carries a *scope* not an *agent identity*.

## Open Questions

- **IW-1: Should the fix sign claim params (T-1427 pattern) or derive identity from the connection?**
  confidence: 2
  disposition: answered
  rationale: The connection layer only grants a scope (Unix→Execute, TCP→token
  scope), never a per-agent identity — agent identity in TermLink comes from
  signed payloads (channel.rs:684). So signing claim params (sender_pubkey_hex +
  signature over canonical claim bytes, verify `claimer == fingerprint`) is the
  only mechanism consistent with the existing model. Deriving from the connection
  would require a new per-connection identity handshake (larger).

- **IW-2: Is this GO now, or DEFER pending the arc-011 threat model?**
  confidence: 1
  disposition: deferred
  rationale: Severity hinges on whether orchestrator+workers are mutually trusted
  (accidental-double-work only) or adversarial (active steal). That is an arc-011
  scoping call the human owns; the fix design is the same either way, but the
  priority isn't. Human decides GO-now vs DEFER-to-arc-011.

- **IW-3: Backward compatibility — do we hard-require signatures or phase them in?**
  confidence: 2
  disposition: answered
  rationale: Existing claim callers (CLI/MCP/skills) send unsigned params; a hard
  cutover breaks every current claim. Phase-in: hub accepts unsigned during a
  migration window (warn-log), clients start signing, then flip to require. Mirror
  of how signed posts rolled out. This is a build-task detail, not a blocker.

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

The exploration (the adversarial review) is DONE — see `docs/reports/
T-2454-claim-identity-binding-inception.md`. Remaining work on a GO decision is
the BUILD, decomposed one-bug/one-slice-per-task:
1. Spike (time-box 1h): canonical claim-byte format + sign/verify helper in
   `termlink-session` reusing the T-1427 `fingerprint_of` + signature primitives.
2. Hub-side verification slice: thread verify into all five claim handlers,
   phase-in (accept unsigned + warn during migration window).
3. Client-side signing slice: CLI/MCP claim commands sign params.
4. Flip to require-signed + regression tests (double-grant scenario now rejected).

## Technical Constraints

- **Trusted-mesh model** — the fix must not break the same-UID-Unix ergonomics
  (a local operator running `termlink channel claim` must still work without
  managing keys manually; reuse the session's existing identity key).
- **Wire backward-compat** — claim RPCs are live in the field (skills, MCP,
  orchestrator); a hard signature requirement is a breaking change → phase-in
  required (IW-3).
- **No new clock source needed** for THIS gap, but the sibling MED-2 (wall-clock
  lease expiry → early-expiry double-grant on NTP step) is noted as a separate,
  smaller follow-up task, not folded in here (one-bug-one-task).

## Scope Fence

**IN:** caller-identity binding for the five claim mutating verbs (design +,
on GO, a phased build). Reuse T-1427 primitives.
**OUT (separate tasks):** MED-2 wall-clock expiry hardening; hiding `claimer`
from Observe-scoped `channel.claims` (a defense-in-depth extra, not the fix);
any change to the SQL state machine (verified correct this round).

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

Round-12 adversarial review confirmed a genuine hole in the coordination substrate's core exclusive-ownership invariant: all five claim verbs (claim/renew/release/transfer/force-release) read claimer/by/to_owner as plain JSON params and enforce ownership against that spoofable string (channel.rs:1618, meta.rs:431/557/623), while channel.post properly binds identity via T-1427 signature (sender_id==fingerprint(pubkey), channel.rs:684). channel.claims exposes {claim_id,claimer} at Observe scope, so any Interact-scoped peer can release/transfer another agent's claim → double-grant (two workers process the same offset). Recommend GO to design the fix, but scope carefully: the correct fix is the T-1427 signature pattern applied to claim params (a protocol change across hub+session+cli+mcp), which is why this is an inception not a direct build. Decision (and GO-to-build) is the human's.

**Evidence:**

- `crates/termlink-hub/src/channel.rs:1618` — `handle_channel_release_with` reads
  `claimer` from params; passes it verbatim to `bus.release_claim(claim_id,
  claimer, ack)`. No connection identity threaded in (handler sig is `(bus, id,
  params)`).
- `crates/termlink-hub/src/channel.rs:684` — `channel.post` DOES bind identity:
  `sender_id != fingerprint_of(verifying_key)` → `CHANNEL_IDENTITY_MISMATCH`
  (T-1427). This is the pattern the claim verbs are missing.
- `crates/termlink-bus/src/meta.rs:431/557/623` — ownership guards compare
  `claimed_by != claimer` where `claimer` is the attacker-controlled param.
- `channel.claims` is Observe-scoped (`server.rs:422`) and returns
  `{claim_id, claimer}` (`meta.rs:669-679`) → the identifiers needed to spoof are
  world-readable within the mesh.
- **Verified CLEAN this round (SQL state machine is sound):** acquire atomicity
  (DELETE-expired + INSERT in one tx under the meta mutex + `UNIQUE INDEX
  idx_claims_topic_offset_active`, no TOCTOU); renew no-resurrect; TTL clamped to
  1h (no permanent claim); release monotonic cursor advance; transfer atomic
  (single UPDATE, lease preserved); force-release + transfer gated at Control
  scope. The ONLY hole is the missing identity binding.
- **Secondary vector (separate task):** MED-2 wall-clock lease expiry
  (`meta.rs:826-831 now_unix_ms = SystemTime::now`) → an NTP forward step can
  expire a live lease early → transient double-grant. Smaller, orthogonal fix.
- Full write-up: `docs/reports/T-2454-claim-identity-binding-inception.md`.

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

### 2026-07-22T09:56:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
