---
id: T-2633
name: "CLI log-path defaults leak to /tmp when HOME unset — route ~/.termlink/*.log through one loud resolver"
description: >
  default_substrate_log_path (substrate.rs:1142) and sibling log-path defaults use HOME.unwrap_or(/tmp), silently relocating observability NDJSON to volatile shared /tmp/.termlink when HOME unset (T-2607 class, reliability/Directive-2). Fold all ~/.termlink/*.log defaults through one loud HOME-anchored resolver.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/agent_find_idle.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/commands/substrate.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T10:42:46Z
last_update: 2026-08-12T12:13:31Z
date_finished: 2026-08-12T12:13:31Z
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

# T-2633: CLI log-path defaults leak to /tmp when HOME unset — route ~/.termlink/*.log through one loud resolver

## Context

Filed from the T-2631 round-4 charter-lens hunt (sibling of the T-2632 MCP fix).
The CLI crate's `~/.termlink/*.log` default-path helpers resolve HOME
inconsistently, and two variants silently relocate observability NDJSON when HOME
is unset — a Reliability (Directive #2: "no silent failures") + Portability
(Directive #4) issue, same family as T-2607/T-2629/T-2632 but on the log plane.

Verified site inventory (2026-08-12):
- `crates/termlink-cli/src/commands/substrate.rs:1142` `default_substrate_log_path`
  → `std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))`
  → **writes to world-writable, reboot-volatile `/tmp/.termlink/substrate.log`** when
  HOME unset. The true `/tmp` leak.
- `crates/termlink-cli/src/commands/agent_find_idle.rs:433` `find_idle_log_path` and
  `crates/termlink-cli/src/commands/channel.rs:10117` `queue_log_path` /
  `channel.rs:11703` `claim_log_path` → HOME unset → **CWD-relative
  `.termlink/<name>.log`** — a *different* silent relocation (per-invocation-varying,
  non-canonical, possibly-unwritable location; audit trail scatters).
- `remote.rs:3502` `rotation_log_path`, `3520` `heal_log_path`, `3917`
  `governor_log_path` → `std::env::var("HOME").ok()?` → return `None` → fail-loud
  (CORRECT — leave as-is; these are the reference pattern).

Fix: introduce one HOME-anchored resolver (mirroring the T-2629/T-2632 ladder —
NOT XDG, to avoid relocating an operator's existing `~/.termlink`), route all
five silently-relocating log-path helpers through it, and keep the loud last-resort
(UID-namespaced private dir + one-time `tracing::error!`) so an unset HOME is
observable rather than silent. The three `?`-returning helpers already fail loud
and stay unchanged.

## Acceptance Criteria

### Agent
- [x] A single HOME-anchored log-dir resolver (pure core + wrapper, mirroring T-2629/T-2632: HOME-set → `$HOME/.termlink`; unset/empty → UID-namespaced temp dir + one-time `tracing::error!`; NOT XDG) exists in the CLI crate. — `resolve_log_dir_from` (pure core) + `termlink_log_dir` (wrapper) + `warn_log_dir_last_resort_once` + `harden_log_dir_last_resort` in `commands/infrastructure.rs`.
- [x] `default_substrate_log_path` (substrate.rs), `find_idle_log_path` (agent_find_idle.rs), `queue_log_path` + `claim_log_path` (channel.rs) all resolve through it — no `/tmp` literal and no CWD-relative `.termlink` fallback remains for these. — all four now `super::infrastructure::termlink_log_dir().join("<name>.log")`; grep confirms the only `/tmp` left is an explanatory comment.
- [x] The three fail-loud helpers (`rotation_log_path`, `heal_log_path`, `governor_log_path`) are left unchanged (they correctly return `None` on unset HOME) — verified by a comment or test noting the deliberate asymmetry. — added a T-2633 asymmetry note above `rotation_log_path` in remote.rs; all three still return `Option`.
- [x] A pure-core unit test proves HOME-unset never yields a world-writable `/tmp/.termlink` nor a bare CWD-relative `.termlink`; load-bearing via temp-revert. — 4 tests in `infrastructure::tests`; `log_dir_home_unset_is_never_tmp_dot_termlink_nor_cwd_relative` is THE guard; reverting the resolver to the `/tmp/.termlink` fallback makes 3 of 4 fail (proven via temp-revert).
- [x] `cargo test -p termlink` (relevant filter) green; `cargo build -p termlink` succeeds. — full binary: 1035 passed, 0 failed; integration 174 passed; `cargo build -p termlink` Finished.

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
cargo test -p termlink infrastructure::tests::log_dir
cargo build -p termlink

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

**Symptom:** With HOME unset, `substrate.log` is written to world-writable
`/tmp/.termlink/`, and `find-idle.log` / `queue.log` / `claim.log` are written to a
CWD-relative `.termlink/` — both silently, with no warning; audit/observability
NDJSON lands in the wrong (and varying) place.

**Root cause:** The `~/.termlink/*.log` default-path helpers were written
independently over many tasks (T-2080/T-2085/T-2114/…) and drifted into three
different HOME-unset behaviors: `/tmp` fallback (substrate), CWD-relative fallback
(find_idle/queue/claim), and fail-loud `?` (rotation/heal/governor). Only the last
is correct.

**Why structurally allowed:** No shared log-dir resolver existed, so each helper
re-implemented HOME resolution ad hoc; no test asserted where any `*.log` default
resolves under HOME-unset, so the divergence was invisible. Same blindness as
T-2632 on the config plane.

**Prevention:** One HOME-anchored resolver with a load-bearing pure-core test that
fails if any routed helper ever falls back to `/tmp` or a bare CWD-relative
`.termlink`; the loud last-resort `tracing::error!` converts the silent relocation
into an observable event. Consolidation also removes the ad-hoc-per-helper drift
vector.

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

### 2026-08-12T10:42:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2633-cli-log-path-defaults-leak-to-tmp-when-h.md
- **Context:** Initial task creation

### 2026-08-12T12:06:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-573a62bd
- **Timestamp:** 2026-08-12T12:14:38Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — A single HOME-anchored log-dir resolver (pure core + wrapper, mirroring T-2629/T-2632: HOME-set → `$HOME/.termlink`; unset/empty → UID-namespaced temp dir + one-time `tracing::error!`; NOT XDG) exists
  - **AC-verify-mismatch** (narrow, heuristic) — `path=commands/infrastructure.rs in: A single HOME-anchored log-dir resolver (pure core + wrapper, mirroring T-2629/T-2632: HOME-set → `$HOME/.termlink`; unset/empty → UID-namespaced temp`

### 2026-08-12T12:13:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
