---
id: T-2457
name: "identity-binding gap below the verification boundary — coordination/governance state keyed on unverified params (round-13 class: cv_index Q1 HIGH + rate-limit Q2 MED, generalizes T-2454)"
description: >
  Inception: identity-binding gap below the verification boundary — coordination/governance state keyed on unverified params (round-13 class: cv_index Q1 HIGH + rate-limit Q2 MED, generalizes T-2454)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-22T18:02:27Z
last_update: 2026-07-22T18:06:43Z
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

# T-2457: identity-binding gap below the verification boundary — coordination/governance state keyed on unverified params (round-13 class: cv_index Q1 HIGH + rate-limit Q2 MED, generalizes T-2454)

## Problem Statement

TermLink binds sender identity cryptographically for exactly one operation:
`channel.post` rejects `sender_id != fingerprint_of(sender_pubkey_hex)`
(`CHANNEL_IDENTITY_MISMATCH`, T-1427, channel.rs:684). Round-13 adversarial review
found **three shipped coordination/governance guards that do NOT sit behind that
boundary** — they compare against unverified request params instead:

- **A — claim verbs** (`release`/`renew`/`transfer`) key ownership on the
  spoofable `claimer`/`by` param. Filed as T-2454 (HIGH, first-discovered).
- **B — cv_index current-value** records `(topic, cv_key) -> offset` from the
  UNSIGNED `metadata.cv_key` (channel.rs:793) with no `cv_key == sender_id` check.
  Producer A can post to `agent-presence` with `cv_key = "<B's fp>"` and evict/
  impersonate B's presence entry → find-idle / push-wake discovery spoof (**HIGH**).
- **C — governor rate-limiter** keys per-sender buckets on the spoofable
  `params.from`/`sender_id` (governor.rs:98), charged at the transport layer BEFORE
  the signature check (server.rs:1149). Rotate `from` → evade own limit; spoof
  victim's `from` → drain their bucket (**MED**).

**For whom / why now:** the substrate underpins arc-011 (parallel orchestrator),
whose correctness depends on these exact guards (exclusive ownership, honest
presence discovery, working backpressure). One root cause, three instances —
worth deciding the boundary once rather than three ad-hoc patches. Full write-up:
`docs/reports/T-2457-identity-binding-below-verification-boundary-inception.md`.

## Assumptions

- Trusted-mesh threat model treats token/Unix-scope holders as semi-trusted, so
  all three are accountability/accidental-double-work hardening (MED) rather than
  external-attacker breaches (HIGH) — the severity, and thus GO-now vs
  DEFER-to-arc-011, hinges on the arc-011 mutually-trusting-vs-adversarial call
  (shared with T-2454 IW-2).
- The correct binding mechanism reuses the T-1427 `fingerprint_of` + signature
  primitives, hoisted so guards outside/before `channel.post` can call them.

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

- **IW-1: Is this one boundary decision (shared primitive) or three independent per-instance fixes?**
  confidence: 2
  disposition: answered
  rationale: All three guards fail the same way — comparing against unverified
  params because the only verified identity (T-1427) is produced inside the
  `channel.post` handler. A reusable `verify_sender_identity(params, conn)`
  hoisted from channel.rs:684 lets every guard (claim/cv_index/rate-limit) key on
  the same verified fingerprint. One decision parameterizes all three builds
  (A=T-2454, B, C). Treating them independently would triplicate the plumbing.

- **IW-2: Can the naive per-instance fixes ship as-is, or do they regress prior work?**
  confidence: 3
  disposition: answered
  rationale: Verified NO for both new instances. B: forcing `cv_key == sender_id`
  breaks arbitrary-key broadcast-replay (#9) where cv_key is a room-state/doc key,
  not an identity — needs a per-topic identity-keyed policy. C: keying the limiter
  on connection identity (peer_addr/pid) regresses PL-218/PL-209 (per-pid buckets
  bloated to ~380K fleet-wide; the `from`-first precedence was the deliberate
  T-2432 fix) — needs a dual-bucket (conn cap pre-verify + identity limit
  post-verify) charge model. Both are design changes, which is why this is an
  inception not a direct build.

- **IW-3: Is this GO-now, or DEFER pending the arc-011 threat model?**
  confidence: 1
  disposition: deferred
  rationale: Severity (MED accountability vs HIGH active-attack) hinges on whether
  arc-011's orchestrator+workers are mutually trusting or adversarial — the same
  human-owned call as T-2454 IW-2. The fix design is identical either way; only
  the priority differs. Human decides GO-now vs DEFER-with-T-2454.

## Exploration Plan

The exploration (adversarial review) is DONE — see the research artifact. On a GO,
the work is: (1) design spike — a `verify_sender_identity` primitive hoisted from
`channel.post` (time-box 2h); then one-bug-one-task builds: (2) cv_index
identity-keyed-topic policy (instance B); (3) governor dual-bucket charge model
(instance C); (4) flip T-2454 (instance A) to the shared primitive.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** the boundary-model decision (shared `verify_sender_identity` primitive +
per-topic cv_index identity policy + governor dual-bucket charge model) and, on GO,
the three phased instance builds (B cv_index, C governor, A=T-2454 flip).
**OUT (separate tasks / not this):** the Q3 receipt-frontier monotonicity fix
(already shipped this round as T-2456); hiding `claimer`/`cv_key` from Observe-scope
reads (defense-in-depth, not the fix); the same-UID-UDS→Execute transport trust
model (documented ADR §7 deviation, its own inception); the SQL claim state machine
(verified correct in T-2454).

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

Round-13 adversarial review found two NEW instances (cv_index cv_key presence-spoof HIGH; governor rate-limit bucket spoof/evade MED) of the SAME root as T-2454's claim finding: coordination/governance guards compare against unverified request params (cv_key, from/sender_id) because they run below/before channel.post's T-1427 signature verification. Each naive per-instance fix hits a real tension (cv_key==sender_id breaks arbitrary-key broadcast-replay #9; connection-keying the rate limiter regresses PL-218 bucket bloat). The shared question — where the verification boundary belongs for governance/coordination state — should be decided once and parameterize all three instance builds. GO to design; human owns the arc-011 threat-model call (same IW as T-2454).

**Evidence:**

- **Instance B (cv_index, HIGH):** `crates/termlink-hub/src/channel.rs:793-797` —
  `cv_index::record(&topic, cv_key, offset)` from `env.metadata.get("cv_key")` with
  no identity check. `channel.rs:696-698` — metadata explicitly excluded from the
  signed bytes. `crates/termlink-hub/src/cv_index.rs:106,215` — monotonic-max,
  one-offset-per-key (A's higher offset evicts B's key). Contrast the bound path at
  `channel.rs:684` (`sender_id != expected_fp` → CHANNEL_IDENTITY_MISMATCH).
- **Instance C (governor, MED):** `crates/termlink-hub/src/governor.rs:98-109`
  (`derive_sender_key` precedence `from`→`sender_id`→`peer_addr`→`peer_pid`), charged
  at `crates/termlink-hub/src/server.rs:1149` before the post handler's verify.
  Fresh-full-bucket mint at `governor.rs:266`. PL-218/PL-209 (380K-bucket bloat) is
  why the naive connection-key fix regresses — the `from`-first order was the T-2432
  fix.
- **Instance A (claim, HIGH):** T-2454 + `docs/reports/T-2454-claim-identity-binding-inception.md`
  (already filed; this inception generalizes it).
- **CLEAN (verified this round):** receipt frontier / await-ack identity-bound
  (`crates/termlink-session/src/ack_retry.rs:107`); the one LOW receding-frontier
  quirk was fixed as **T-2456** (commit a9064ff8, 434 hub tests green).
- Full write-up: `docs/reports/T-2457-identity-binding-below-verification-boundary-inception.md`.

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

### 2026-07-22T18:03:27Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
