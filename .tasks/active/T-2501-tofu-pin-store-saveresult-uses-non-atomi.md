---
id: T-2501
name: "TOFU pin store save_result uses non-atomic truncate-write — crash mid-write truncates known_hubs, silent trust downgrade"
description: >
  TOFU pin store save_result uses non-atomic truncate-write — crash mid-write truncates known_hubs, silent trust downgrade

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
created: 2026-08-02T16:40:50Z
last_update: 2026-08-02T16:40:50Z
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

# T-2501: TOFU pin store save_result uses non-atomic truncate-write — crash mid-write truncates known_hubs, silent trust downgrade

## Context

`KnownHubStore::save_result` (crates/termlink-session/src/tofu.rs:157) persists
the TOFU pin store with `std::fs::write(&self.path, ...)`. `std::fs::write` opens
the real path with `O_TRUNC` — the existing content is destroyed *first*, then
the new bytes are written. If the process dies between the truncate and the
completed write (SIGKILL, OOM-kill, power loss, or `ENOSPC` mid-write), the
`known_hubs` file is left **truncated or partial** on disk. On the next process
start the partial file loads → the pins that were lost are now **absent** → the
next connection to those hubs silently re-trusts-on-first-use (TOFU), which is
exactly the MITM window the pin exists to close.

T-2438 made the *`Err`-return* path loud (the `save()` eprintln warning), but a
process kill or power-loss is not an `Err` — it is process death mid-write, which
truncate-in-place cannot survive. The rest of this crate already persists
integrity-sensitive files atomically via write-temp-then-rename
(`registration.rs::write_atomic`, `known_peers.rs::save`, `agent_identity.rs`);
the TOFU store is the odd one out. POSIX `rename(2)` within one filesystem is
atomic, so a reader always sees either the complete old file or the complete new
one — never a truncated one.

This is the no-silent-failures campaign (directive #2 Reliability), and it sits
directly on the charter's discover/auth-integrity path — sibling of
T-2497/T-2498/T-2499/T-2500.

## Acceptance Criteria

### Agent
- [x] `save_result` writes to a sibling temp path then atomically `rename`s it over `self.path` (mirrors `registration.rs::write_atomic`) — no more truncate-in-place of the live file. (tofu.rs:157 — `fs::write(&tmp)?; fs::rename(&tmp, &self.path)`)
- [x] The parent-dir creation and the exact on-disk file format (header comments + `host_port fp first_seen last_seen` lines + trailing newline) are unchanged — only the write mechanism changes. (`lines.join` unchanged; `create_dir_all` retained)
- [x] Existing tofu tests stay green: `save_result_persists_and_reloads_pin` (T-2438 happy path), `save_result_errs_on_unwritable_path` (failure still surfaces), `store_persists_to_disk`, `remove_persists_to_disk`. (17/17 tofu tests pass)
- [x] A test proves an atomic write: after `save_result`, no leftover temp file remains beside `self.path`, and the store round-trips (write → new store from same path → pin present). (`save_result_is_atomic_no_leftover_temp`)
- [x] `cargo test -p termlink-session --lib tofu` passes. (17 passed; full lib 397 passed, 0 failed)

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

out=$(cargo test -p termlink-session --lib tofu 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** After an abrupt process termination (SIGKILL / OOM-kill / power
loss) or a disk-full condition during a TOFU pin save, the `known_hubs` file is
left truncated or partially written. On the next process start the partial file
loads with some (or all) pins missing; the next connection to an affected hub
silently re-trusts-on-first-use instead of verifying the pinned fingerprint — a
trust downgrade / MITM window, with no error surfaced (the file loaded "fine",
it was just short).

**Root cause:** `save_result` used `std::fs::write(&self.path, ...)`, which
truncates the live file to zero (`O_TRUNC`) *before* writing the new content.
The window between truncate and write-completion is not crash-safe: if the
process dies in that window, the on-disk file reflects the partial state. The
integrity boundary here is "the pin store on disk must always be a complete,
valid snapshot" — truncate-in-place violates that by design.

**Why structurally allowed:** T-2438 correctly hardened the *`Err`-return* path
(a write that fails now warns loudly) but framed the risk as "the write call
returns an error". Process death mid-write is not an `Err` — the call never
returns — so it fell outside that fix's model. Meanwhile the rest of the crate
had already converged on write-temp-then-`rename` for exactly this reason
(`registration.rs::write_atomic`, `known_peers.rs`, `agent_identity.rs`); the
TOFU store simply predated / diverged from that convention and no test exercised
a mid-write crash, so the gap was invisible.

**Prevention:** (1) The fix writes to a sibling temp path then atomically
`rename`s it over the live file — POSIX `rename(2)` guarantees a reader sees
either the whole old or whole new file, so a crash can never leave a truncated
`known_hubs`; the original file survives untouched if the write dies. (2) A test
asserts no leftover temp file remains after a save and the store round-trips,
so a regression to `fs::write` (which would leave the file non-atomically
written) is caught. (3) Learning generalizes: integrity-sensitive files must be
written atomically (temp+rename), never truncate-in-place — the `Err` path is
necessary but not sufficient.

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

### 2026-08-02T16:40:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2501-tofu-pin-store-saveresult-uses-non-atomi.md
- **Context:** Initial task creation
