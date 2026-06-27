---
id: T-2292
name: "V1: per-agent identity by default"
description: >
  RC1 fix. Make per-agent identity the DEFAULT (crypto already shipped T-1693/G-056; this is defaults wiring). register/be-reachable/listener-heartbeat set a stable per-agent-id key ~/.termlink/identities/<agent_id>.key instead of shared $HOME/.termlink/identity.key. Clean cutover, no DM-history migration. ACs: register defaults to per-agent key keyed on agent_id; be-reachable + heartbeat set it; whoami shows DISTINCT fingerprint per co-resident agent; DM-topic discontinuity documented; transport trust (hub.secret/TLS) untouched. Foundation for V2/V6 (identity auths the direct socket).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:reliable-comms]
components: []
related_tasks: [T-2291]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-27T17:05:54Z
last_update: 2026-06-27T17:09:58Z
date_finished: null
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
---

# T-2292: V1: per-agent identity by default

## Context

Arc-003 (reliable-comms) foundation slice. RC1 from the T-2291 inception RCA: co-resident agents on host .107 share one host fingerprint, collapsing DM topics and erasing per-agent attribution. The crypto already shipped (T-1693/G-056); this task only flips the DEFAULT so per-agent keys are used without an explicit flag. Design trail: `docs/reports/T-2291-cross-agent-comms-inception.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `register` (and `register --self`) defaults to a stable per-agent key at `~/.termlink/identities/<agent_id>.key`, derived from agent_id, instead of the shared `$HOME/.termlink/identity.key` — `TERMLINK_AGENT_ID` branch added to `resolve_identity_key_path` (registration.rs) + `load_identity_or_create` (channel.rs, lockstep) + `bind_per_agent_identity_default` (session.rs, creates+pins on register). Live proof: two co-resident agents under one $HOME got distinct 32-byte seeds `bc747640…` / `aea5baa4…`.
- [x] `/be-reachable` and `listener-heartbeat.sh` set/use the same per-agent key path — both now `export TERMLINK_AGENT_ID="$agent_id"` so heartbeat/DM posts sign with the per-agent key.
- [x] `termlink whoami --json` shows a DISTINCT `identity_fingerprint` for two co-resident agents on the same host — register bakes the per-agent fingerprint into SessionMetadata (which whoami reads); distinct keys → distinct fingerprints, proven by `per_agent_paths_produce_distinct_fingerprints` + live distinct seeds.
- [x] Clean cutover: the DM-topic naming discontinuity (old shared-fp topics vs new per-agent topics) is documented; no history migration attempted — `docs/operations/per-agent-identity.md`.
- [x] Transport trust (`hub.secret` HMAC + TLS cert pinning) is untouched — verified by a passing `fleet doctor` against an existing hub after the change — only client signing-key selection changed; `fleet doctor` connects + auths against live hubs (all reachable hubs `[PASS]`).

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
cargo test -p termlink-session --lib agent_identity::
cargo test -p termlink-session --lib resolve_identity_key_path
bash -n scripts/listener-heartbeat.sh
bash -n scripts/be-reachable.sh
test -f docs/operations/per-agent-identity.md

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

### 2026-06-27 — two resolvers, not one

- **What changed:** The inception named one resolver (`registration.rs:48`), but
  there are actually **two** identity-key resolvers that must agree: the
  fingerprint path (`registration.rs::resolve_identity_key_path`) AND the
  post-signing path (`channel.rs::load_identity_or_create`). They are explicitly
  documented as "in lockstep" (the wire `sender_id` must equal the SessionMetadata
  fingerprint). Both got the identical `TERMLINK_AGENT_ID` branch.
- **Plan impact:** None to scope — still "defaults wiring" — but the change
  touched 3 Rust files (+2 scripts) instead of 1.
- **Triggered:** Precedence decision locked as **FILE > DIR > AGENT_ID > shared
  default** (agent_id is a smart default below explicit operator overrides, not
  above them). `register` had to actively *create* the key (via a new
  `bind_per_agent_identity_default` helper) because the fingerprint resolver is
  read-only by design; without creation, `whoami` would show no fingerprint
  until first post.

### 2026-06-27 — agent_id was never plumbed into identity

- **What changed:** Discovered `resolve_identity_key_path` took **no agent_id
  input** at all — resolution was purely env-driven, and the heartbeat
  `metadata.agent_id` was explicitly a free-form *label, not identity*. V1's
  essence is **unifying** the logical agent_id with the crypto identity via a new
  `TERMLINK_AGENT_ID` env var, which the scripts now export.
- **Plan impact:** Confirmed the crypto was done (T-1693/G-056); the gap was a
  missing env→key-path binding, exactly as the RCA predicted.
- **Triggered:** `sanitize_agent_id` added (filesystem-safety: a logical id with
  `/`, `:`, or `..` must not escape `identities/`).

## Decisions

### 2026-06-27 — per-agent default via TERMLINK_AGENT_ID env, additive branch

- **Chose:** A new `TERMLINK_AGENT_ID` env var that, when set, diverts the key
  path to `~/.termlink/identities/<agent_id>.key`; unset → unchanged shared
  default. Slotted below `TERMLINK_IDENTITY_DIR` in precedence.
- **Why:** Purely additive and backward compatible — single-agent hosts and all
  existing `--identity-key`/`TERMLINK_IDENTITY_DIR` deployments are unaffected.
  Co-resident agents opt in simply by declaring their agent_id (which
  `/be-reachable` + heartbeat already know).
- **Rejected:** (a) Changing the *shared* default unconditionally — would break
  single-agent hosts and force a fleet-wide re-pin with no opt-out. (b) Deriving
  the key deterministically from agent_id without a file — loses the chmod-600
  on-disk key the rest of the system expects.

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

### 2026-06-27T17:05:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2292-v1-per-agent-identity-by-default.md
- **Context:** Initial task creation

### 2026-06-27T17:08:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
