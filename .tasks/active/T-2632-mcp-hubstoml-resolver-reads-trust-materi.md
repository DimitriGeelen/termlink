---
id: T-2632
name: "MCP hubs.toml resolver reads trust material from world-writable /tmp when HOME unset (T-2607 read-side mirror)"
description: >
  MCP hubs.toml resolver reads trust material from world-writable /tmp when HOME unset (T-2607 read-side mirror)

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
created: 2026-08-12T10:35:22Z
last_update: 2026-08-12T10:35:22Z
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

# T-2632: MCP hubs.toml resolver reads trust material from world-writable /tmp when HOME unset (T-2607 read-side mirror)

## Context

Three MCP functions in `crates/termlink-mcp/src/tools.rs` resolve the hub-profile
config (`~/.termlink/hubs.toml`) via `std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())`:
`resolve_hub_profile` (7322), `list_all_hub_profiles` (7372), and
`read_bootstrap_from_map` (10604). `hubs.toml` carries hub addresses,
`secret_file` paths, and `bootstrap_from` reauth anchors — trust material. When
`HOME` is unset (systemd unit without `Environment=HOME`, `env -i`, minimal
container), all three silently read `/tmp/.termlink/hubs.toml`, a world-writable,
reboot-volatile shared path. This is the read-side security mirror of the
write-side T-2607 (identity_dir) / T-2629 (CLI config.rs) fixes, in the MCP crate,
which that sweep never touched. Fix mirrors the blessed T-2629 config-dir ladder
(HOME-anchored, deliberately NOT XDG — routing through the session crate's
XDG-inclusive `resolve_identity_dir()` would relocate an operator's existing
hubs.toml when `XDG_STATE_HOME` is set), duplicated into the MCP crate per the
T-2069 "no cross-crate sharing for tiny pure helpers" convention.

## Acceptance Criteria

### Agent
- [x] A pure `resolve_mcp_config_dir_from(home: Option<&str>, temp: &Path, uid: u32) -> (PathBuf, bool)` core exists: HOME-set (non-empty) → `home/.termlink` (bool=false, behavior-preserving); HOME-unset/empty → `temp/termlink-<uid>` (bool=true), matching T-2629's last-resort path shape so CLI and MCP agree.
- [x] An `mcp_hubs_toml_path()` wrapper resolves via that core, and on the last-resort branch emits a one-time `tracing::error!` and hardens the dir to mode 0700; all three sites (7322/7372/10604) call it instead of the inline `HOME.unwrap_or("/tmp")`.
- [x] Unit tests prove: home-set uses `$HOME/.termlink/hubs.toml`; empty HOME is treated as unset (not `/.termlink`); HOME-unset never resolves under a shared `.termlink` at a world-writable root; last-resort honors the temp base + UID namespacing. Tests hold `HOME_TEST_LOCK`.
- [x] The load-bearing guard test FAILS when the fix is reverted to `HOME.unwrap_or("/tmp")` (proven via temp-revert), and PASSES with the fix.
- [x] `cargo test -p termlink-mcp mcp_config_dir` is green; `cargo build -p termlink-mcp` succeeds.

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

cargo test -p termlink-mcp --lib mcp_config_dir
cargo build -p termlink-mcp

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

**Symptom:** When the MCP server runs with `HOME` unset, `resolve_hub_profile`,
`list_all_hub_profiles`, and `read_bootstrap_from_map` read hub trust config from
`/tmp/.termlink/hubs.toml` instead of `~/.termlink/hubs.toml` — with no error.

**Root cause:** `std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())`
treats an unset/empty HOME as if it were `/tmp`, then joins `.termlink/hubs.toml`.
`/tmp` is world-writable and reboot-volatile. An unprivileged local user can plant
`/tmp/.termlink/hubs.toml` with attacker-controlled hub addresses, `secret_file`
paths, and `bootstrap_from` reauth anchors, which the MCP server then trusts for
`channel post` / `fleet bootstrap-check` / reauth. Same class as T-2607/T-2629 but
on the READ side and in the MCP crate.

**Why structurally allowed:** The T-2607 unification hardened the identity/trust
dir and T-2629/T-2630 covered the CLI crate, but the MCP crate has its own ad-hoc
TOML resolvers (kept toml-crate-free) that were never routed through a hardened
resolver. No test asserted where hubs.toml is read from under HOME-unset, so the
`/tmp` fallback was invisible.

**Prevention:** A pure `resolve_mcp_config_dir_from` core with a load-bearing guard
test that fails if the resolver ever falls back to a shared world-writable
`.termlink` — reverting to `HOME.unwrap_or("/tmp")` re-fires it. The loud one-time
`tracing::error!` on the last-resort branch converts the silent relocation into an
observable, auditable event (Directive #2).

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

### 2026-08-12T10:35:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2632-mcp-hubstoml-resolver-reads-trust-materi.md
- **Context:** Initial task creation
