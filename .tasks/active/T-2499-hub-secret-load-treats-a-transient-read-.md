---
id: T-2499
name: "hub secret load treats a transient read error as missing and silently regenerates — full fleet re-auth (PL-021 class)"
description: >
  hub secret load treats a transient read error as missing and silently regenerates — full fleet re-auth (PL-021 class)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T12:37:21Z
last_update: 2026-08-02T12:37:21Z
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

# T-2499: hub secret load treats a transient read error as missing and silently regenerates — full fleet re-auth (PL-021 class)

## Context

`load_existing_hub_secret` (`crates/termlink-hub/src/server.rs:100`) reads the
persistent `hub.secret` with `std::fs::read_to_string(&path).ok()?` — collapsing
EVERY read error into `None`. `None` makes `generate_and_write_hub_secret` (:119)
skip reuse and `std::fs::write` a FRESH secret over the existing file. So a
*transient* read failure on an EXISTING valid secret — EACCES after a perms glitch,
EIO/ESTALE on an NFS-backed runtime_dir, EINTR — is indistinguishable from "file
genuinely absent", and the hub silently ROTATES its HMAC secret. Every cross-host
agent's cached secret goes stale → the full-fleet re-auth PL-021 (`Token validation
failed: invalid signature`) the codebase documents at length — triggered here not by
a real rotation but by a momentary read hiccup, with only an info-level "Hub secret
written" line. Only `NotFound` should mean "generate"; any other IO error must refuse
to overwrite a secret that may still be valid on disk. Directive-#2 (silent failure
with fleet-wide blast radius).

## Acceptance Criteria

### Agent
- [ ] A read error that is NOT `NotFound` (EACCES/EIO/ESTALE/EINTR) no longer silently regenerates the secret — it propagates LOUD so the hub refuses to overwrite a possibly-valid secret
- [ ] `NotFound` (genuine first-deploy absence) still generates a fresh secret — no behavior change on the legitimate path
- [ ] A present-but-invalid secret file (wrong length / non-hex) still regenerates — no behavior change (matches the existing T-933 "regen after corrupt" test)
- [ ] The read-classification is factored into a pure `classify_secret_read` helper so the "read-error != missing" rule is unit-testable without touching the global runtime_dir
- [ ] Regression tests pin all four cases (NotFound→generate, other-error→refuse, valid→reuse, invalid→regenerate)
- [ ] `cargo test -p termlink-hub --lib` passes (existing T-933 persist test + new)

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
cargo test -p termlink-hub --lib secret 2>&1 | tail -5 | grep -q "test result: ok"

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

**Symptom:** On hub restart, a momentary read failure of an existing, valid
`hub.secret` (EACCES/EIO/ESTALE/EINTR — e.g. an NFS-backed runtime_dir or a perms
race) causes the hub to silently generate and write a BRAND-NEW secret. Every
cross-host agent's cached secret is now stale → fleet-wide `Token validation failed:
invalid signature` (PL-021), indistinguishable from a real rotation, with only an
info-level "Hub secret written" line.

**Root cause:** `load_existing_hub_secret` used `std::fs::read_to_string(&path).ok()?`.
`.ok()` discards the error TYPE, collapsing `NotFound` (legitimate: no secret yet →
generate) with every other IO error (the file exists but couldn't be read → must NOT
regenerate) into a single `None`. The caller treats `None` as "generate a fresh one",
so a transient read error triggers a silent secret rotation.

**Why structurally allowed:** `.ok()?` on a file read is an ergonomic idiom that reads
as "get the contents or give up", but on a SECRET file the discriminant between
"absent" and "unreadable" is load-bearing — one means generate, the other means refuse.
The persist-if-present work (T-933) focused on the happy path (reuse when present) and
never distinguished the failure modes of the read itself. No lint flags `.ok()` erasing
an error kind that the caller then branches on.

**Prevention:** Extract a pure `classify_secret_read(io::Result<String>) ->
io::Result<Option<String>>` — `NotFound => Ok(None)` (generate), other `Err => Err`
(refuse/propagate loud), valid hex => `Ok(Some)`, invalid content => `Ok(None)`
(regenerate) — and unit-test all four cases. Rule captured as a learning: `.ok()` /
`.unwrap_or` on a read whose error KIND the caller branches on erases the load-bearing
discriminant; match the kind explicitly. Same no-silent-failures class as PL-283/284.

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

### 2026-08-02T12:37:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2499-hub-secret-load-treats-a-transient-read-.md
- **Context:** Initial task creation
