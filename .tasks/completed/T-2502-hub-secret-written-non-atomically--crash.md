---
id: T-2502
name: "hub secret written non-atomically — crash mid-write truncates hub.secret, silent fleet-wide re-auth (PL-021)"
description: >
  hub secret written non-atomically — crash mid-write truncates hub.secret, silent fleet-wide re-auth (PL-021)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T16:54:41Z
last_update: 2026-08-02T16:57:46Z
date_finished: 2026-08-02T16:57:46Z
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

# T-2502: hub secret written non-atomically — crash mid-write truncates hub.secret, silent fleet-wide re-auth (PL-021)

## Context

`generate_and_write_hub_secret` (crates/termlink-hub/src/server.rs:156) persists
the hub's 32-byte HMAC secret — the **fleet-wide auth root** — with
`std::fs::write(&path, &secret_hex)`. `std::fs::write` opens `hub.secret` with
`O_TRUNC` and writes in place. If the process dies between open and write
completion (SIGKILL / OOM-kill / power loss / `ENOSPC`), `hub.secret` is left
**truncated or partial** on disk. On the next boot `load_existing_hub_secret`
reads the partial file, rejects it (`len != 64` / non-hex → `Ok(None)`, T-2499),
and **regenerates a fresh secret** — silently rotating the fleet auth root. Every
cross-host agent's cached secret is now stale → fleet-wide auth-mismatch
(`invalid signature`), i.e. the PL-021 / G-058 re-auth cascade this codebase
documents at length. Only an info-level `Hub secret written` log marks it.

This is the exact defect T-2501 fixed for the TOFU pin store, one level up in
blast radius (the auth root vs a pin), and a **distinct function** from T-2499
(which hardened the *read* path's error classification; this is the *write*
path's atomicity). The rest of the codebase already writes integrity-sensitive
files atomically via temp-then-rename (`known_peers.rs::save`,
`registration.rs::write_atomic`, `agent_identity.rs`, `tofu.rs` post-T-2501);
the hub secret is the divergent one — and the highest-value file of them all.

No-silent-failures campaign (directive #2 Reliability), on the charter's
auth-integrity path — sibling of T-2497/T-2498/T-2499/T-2500/T-2501.

## Acceptance Criteria

### Agent
- [x] `generate_and_write_hub_secret` writes `hub.secret` to a sibling temp path then atomically `rename`s it over the real path — no more truncate-in-place of the auth root. (server.rs:156 — 0600 temp `open`/`write_all`/`flush` then `fs::rename`)
- [x] The secret temp file is created with mode `0600` up-front (never world-readable, even briefly), and the final file remains `0600`; non-unix falls back to plain temp+rename. The generated value and its format (64-char lowercase hex) are unchanged. (`.mode(0o600)` on OpenOptions + defensive `set_permissions`; test asserts `mode & 0o777 == 0o600`)
- [x] Existing T-933 test `hub_secret_persists_across_calls` stays green (persist-if-present + regenerate-on-corrupt), and the T-2499 `classify_secret_read_*` tests stay green. (454/454 hub-lib pass)
- [x] A test proves the atomic write: after `generate_and_write_hub_secret`, no leftover `hub.secret.tmp` remains beside the secret path, and the secret round-trips (second call reuses it). (`hub_secret_written_atomically`)
- [x] `cargo test -p termlink-hub --lib` passes. (454 passed; 0 failed)

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

out=$(cargo test -p termlink-hub --lib 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-hub --lib hub_secret 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** After an abrupt hub termination (SIGKILL / OOM-kill / power loss)
or a disk-full condition during hub-secret generation, `hub.secret` is left
truncated/partial on disk. The next hub boot rejects the partial secret and
regenerates a fresh one, silently rotating the fleet HMAC auth root — every
cross-host agent then sees `invalid signature` / auth-mismatch (PL-021 / G-058),
with only an info-level `Hub secret written` log to mark the rotation.

**Root cause:** `generate_and_write_hub_secret` used
`std::fs::write(&path, &secret_hex)`, which truncates the target in place before
writing. The integrity boundary — "`hub.secret` on disk is always a complete,
valid 64-hex secret or absent" — is violated by the truncate/write window: a
crash there produces a partial file that satisfies neither invariant, and the
downstream T-2499 read path (correctly) treats present-but-invalid as
"regenerate", turning the torn write into a silent rotation.

**Why structurally allowed:** T-933 established persist-if-present and T-2499
hardened the *read* path's error classification, but neither addressed *write*
atomicity — the write was framed as "generate and save", not "save crash-safely".
The codebase had already converged on temp+rename for other integrity-sensitive
files (`known_peers.rs`, `registration.rs`, `agent_identity.rs`, and `tofu.rs`
as of T-2501), but the single highest-value file — the fleet auth root — kept
the naive truncate-in-place write, and no test exercised a mid-write crash.

**Prevention:** (1) The fix writes to a `0600` sibling temp file then atomically
`rename`s it over `hub.secret` — POSIX `rename(2)` guarantees the reader sees the
whole old (absent) or whole new file, never a truncated one; a crash mid-write
leaves only the discardable temp. (2) A test asserts no leftover temp file after
generation and the secret round-trips, catching a regression to `fs::write`.
(3) Learning PL-287 (atomic writes for integrity-sensitive files) already
generalizes this; this task confirms the auth root is covered.

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

### 2026-08-02T16:54:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2502-hub-secret-written-non-atomically--crash.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c6b3d896
- **Timestamp:** 2026-08-02T16:58:43Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T16:57:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
