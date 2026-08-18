---
id: T-2534
name: "Bind claim verbs to verified identity (T-1427 pattern) — execute T-2454 GO"
description: >
  Execute the T-2454 GO decision: all 5 claim verbs (claim/renew/release/transfer/force-release)
  enforce ownership against a caller-supplied free-text claimer/by string with NO
  cryptographic identity binding, while channel.post correctly binds identity via
  T-1427 (sender_id==fingerprint(pubkey)). Any Interact-scoped peer can release/renew/transfer
  another agent's claim → double-grant / silent work-loss. Fix: apply the T-1427 signature
  pattern to claim params (protocol change across hub+session+cli+mcp). GO-to-build
  is the human's per T-2454.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-08T07:59:23Z
last_update: '2026-08-18T18:58:38Z'
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:34Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:38Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2534: Bind claim verbs to verified identity (T-1427 pattern) — execute T-2454 GO

## Context

Executes the **GO decision recorded in T-2454** (inception, round-12 HIGH). Re-confirmed
in code 2026-08-08 by a fresh adversarial claim-primitive hunter (T-2468 campaign).

**The gap:** all 5 claim verbs enforce exclusive ownership against a **caller-supplied,
spoofable free-text string** (`claimer` / `by` / `to_owner` JSON param), with NO binding
to the connection's verified cryptographic identity. `channel.post` does it correctly
(T-1427: rejects `CHANNEL_IDENTITY_MISMATCH` unless `sender_id == fingerprint_of(pubkey)`
at `crates/termlink-hub/src/channel.rs:679-694`). The claim verbs carry no
`sender_pubkey_hex` / `signature` params at all.

**Reachable exploit:** `channel.claims` is Observe-scoped and returns `{claim_id, claimer}`
world-readably within the mesh. Any Interact-scoped peer can then:
- `channel.release {claim_id, claimer:"<victim>", ack:true}` → ownership check passes
  (`claimed_by == "<victim>"`), the **victim's cursor advances past the offset** → the
  in-flight unit is silently marked consumed, slot reopens = **silent work-loss**.
- `channel.claim_transfer` / `renew` spoofed as the victim → yanks / extends an in-flight
  claim = **double-dispatch** (two workers on the same offset).

**Defect sites (ownership gate fed the unverified param):**
- `crates/termlink-hub/src/channel.rs:1638-1646` (release), `:1841` (renew),
  `:1759-1766` (transfer `by`), `:1573` (claim misattribution).
- `crates/termlink-bus/src/meta.rs:481` (and siblings `:431/:557/:623`):
  `if claimed_by != claimer { return Err(ClaimNotOwned) }` — `claimer` is the unverified
  param. Each handler signature is `(bus, id, params)` — no per-connection identity is
  threaded in.

**Confirmed CLEAN (do NOT touch — the mechanics are sound):** acquire atomicity
(DELETE-expired + INSERT in one tx + `UNIQUE INDEX idx_claims_topic_offset_active`, no
TOCTOU); TTL clamp to 1h (`.min(3_600_000)`, `channel.rs:1579-1583/1846-1850`);
`now + ttl` overflow-safe (`saturating_add`, `meta.rs:416/686`); monotonic-forward renew
(T-2510); atomic single-tx transfer; collision-proof `claim_id` (T-2461). The ONLY hole
is identity binding.

## Recommended Fix (from T-2454 GO — protocol change, scope carefully)

Apply the **T-1427 signature pattern** to the claim verbs: thread the connection's verified
fingerprint into the claim/renew/release/transfer handlers and require the ownership param
(`claimer` / `by`; and `claimer` on `claim` for correct attribution) to equal it — mirroring
`channel.post`'s `sender_id == fingerprint_of(verifying_key)` gate. This is a coordinated
change across **hub** (handlers accept + verify `sender_pubkey_hex` + `signature`), **session**
(client signs claim requests), **cli** + **mcp** (pass the signature params). `force-release`
stays Control-scoped (operator bypass, by design).

**Open scope question for the human (why this is human-owned):** should the substrate become
a fully authenticated boundary like `post`, or is a lighter binding (e.g. connection-identity
without per-request signatures) acceptable given the ADR "trusted-mesh" framing? The T-2454
GO says authenticate it; the exact mechanism + wire-compat strategy (old clients omit
signature — reject, or grandfather?) is the human's call.

## Acceptance Criteria

### Agent
<!-- Populated once the human authorizes the build + picks the wire-compat strategy.
     Draft (pending human scope decision above): -->
- [ ] Claim/renew/release/transfer handlers verify `sender_pubkey_hex` + `signature` and reject `CLAIM_IDENTITY_MISMATCH` unless the ownership param == `fingerprint_of(verifying_key)` (mirror T-1427 in `channel.rs:679-694`)
- [ ] `claim` records `claimed_by` = the verified fingerprint (correct attribution, not the raw param)
- [ ] session/cli/mcp sign claim requests; wire-compat strategy per the human's decision (reject-unsigned vs grandfather)
- [ ] Load-bearing test: a release/transfer request whose signature does NOT match the named `claimer` is rejected `CLAIM_IDENTITY_MISMATCH` (temp-revert the identity check → test lets the spoof through)
- [ ] `force-release` remains operator-Control-scoped (unchanged); existing honest-path claim tests stay green

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-08T07:59:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2534-bind-claim-verbs-to-verified-identity-t-.md
- **Context:** Initial task creation
