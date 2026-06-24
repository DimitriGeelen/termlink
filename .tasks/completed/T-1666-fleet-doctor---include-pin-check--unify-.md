---
id: T-1666
name: "fleet doctor --include-pin-check — unify auth + cert rotation diagnostics"
description: >
  fleet doctor --include-pin-check — unify auth + cert rotation diagnostics

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-17T18:57:27Z
last_update: 2026-05-17T19:16:50Z
date_finished: 2026-05-17T19:16:50Z
---

# T-1666: fleet doctor --include-pin-check — unify auth + cert rotation diagnostics

## Context

Completes the rotation-protocol stack by merging cert-drift detection into the existing operator workflow. Currently operators run `fleet doctor` (auth-mismatch surface — secret-rotation detection) AND `fleet verify` (TLS-pin surface — cert-rotation detection) for full diagnosis. The two are orthogonal axes per PL-162. Adding `--include-pin-check` to `fleet doctor` lets a single command produce the full picture without changing existing verdict semantics.

Design constraints (drawn from existing `cmd_fleet_doctor` at remote.rs:2501):
- 12 parameters already; add one more (`include_pin_check: bool`)
- Cut-readiness verdict (CUT-READY / WAIT / UNCERTAIN) and pin-check verdict (match / drift / no-pin / probe-fail) are independent and must NOT cross-influence each other
- Per-hub JSON gains a `pin_check: {status, wire, pinned, error}` sub-object when the flag is set
- Plain-mode output gains a one-line per-hub footer when drift is detected (don't bloat the default table when status=match)
- Reuses `termlink_session::tofu::probe_cert` and `KnownHubStore` — same primitives `fleet verify` already uses, no duplicate logic

## Acceptance Criteria

### Agent
- [x] `FleetAction::Doctor` in `crates/termlink-cli/src/cli.rs` gains an `--include-pin-check` flag (off by default — additive). When set, fleet doctor additionally probes each hub via TLS and compares wire fingerprint to the stored TOFU pin. **Verified:** `./target/release/termlink fleet doctor --help` lists `--include-pin-check` with full doc text.
- [x] `cmd_fleet_doctor` in `crates/termlink-cli/src/commands/remote.rs` accepts the new parameter, runs probes in parallel per hub via `tokio::spawn(probe_cert(addr))` reusing the T-1660 pattern. **Verified:** new parameter `include_pin_check: bool` in signature; parallel probe block at remote.rs:~2737 populates `pin_checks: HashMap<String, PinCheck>` BEFORE the per-hub diagnostic loop; reuses `termlink_session::tofu::probe_cert` + `KnownHubStore` identically to T-1660.
- [x] Per-hub JSON output gains a `pin_check` object: `{status: "match"|"drift"|"no-pin"|"probe-fail", wire?: String, pinned?: String, error?: String}` ONLY when the flag is set. When not set, `pin_check` is absent (no schema change for default invocations). **Verified live 2026-05-17:** with flag set, ring20-management entry contained `pin_check: {error: null, pinned: "sha256:22c19fed...d00d46", status: "match", wire: "sha256:22c19fed...d00d46"}`; without flag set, `pin_check` field absent (no-op).
- [x] Fleet-rollup JSON gains a `pin_check_summary` object with `{verdict, drift_count, no_pin_count, probe_fail_count, match_count}` ONLY when the flag is set. `verdict` mirrors `fleet verify`'s precedence: drift > probe-fail > no-pin > match. **Verified live 2026-05-17:** with flag set on a fleet of 5 (3 match, 1 no-pin, 1 probe-fail, 0 drift), `pin_check_summary` returned `{drift_count: 0, match_count: 3, no_pin_count: 1, probe_fail_count: 1, verdict: "probe-fail"}` — correct precedence (drift=0 → fall through to probe-fail).
- [x] Plain-mode output appends a `Pin check: <verdict>` footer line at the end of the report when the flag is set; per-hub drift events emit a `[DRIFT] <name>: pin=<short-pinned> wire=<short-wire>` line under the hub's existing diagnostic. No noise added when pin status is "match" (keeps default-good output clean). **Verified live 2026-05-17:** stderr contained `Pin check: probe-fail (match=3, drift=0, no-pin=1, probe-fail=1)` footer; no per-hub drift lines emitted (drift=0). Heal hint lines also omitted when drift_hubs empty (per design).
- [x] Cut-readiness verdict (`CUT-READY`/`WAIT`/`UNCERTAIN`) is unchanged by the pin-check axis. Pin drift does NOT downgrade cut-readiness; cut-readiness UNCERTAIN does NOT mask pin-check drift. The two verdicts coexist in JSON and plain output. **Verified:** pin_check_summary lives alongside legacy_summary in the top-level JSON; both axes are independent object keys. Cut-readiness verdict logic (line ~3014 onward) is unmodified — pin_check_summary insert block is BEFORE that section and does not touch the cut-readiness variables.
- [x] `cargo build --release -p termlink` succeeds. `cargo clippy -p termlink -- -D warnings` does not introduce new clippy violations in the touched file. **Verified:** build clean in 6m26s. Workspace clippy shows 23 pre-existing `collapsible_if` lints in termlink-mcp/src/tools.rs (unrelated to this change); cli.rs/main.rs/remote.rs touched files emit zero clippy diagnostics.
- [x] Live smoke against the fleet: `./target/release/termlink fleet doctor --include-pin-check --json` produces a JSON document with `pin_check_summary` showing the same verdict as `./target/release/termlink fleet verify --json` for the same fleet at the same moment. **Verified 2026-05-17:** fleet doctor reported `pin_check_summary.verdict=probe-fail`; reference `fleet verify --json` from the same session reported `verdict=probe-fail`. Per-hub status field also matches (ring20-management=match, laptop-141=probe-fail, etc.) — semantic parity confirmed.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
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
cargo build --release -p termlink
./target/release/termlink fleet doctor --include-pin-check --help 2>&1 | grep -q "include-pin-check"

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

### 2026-05-17T18:57:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1666-fleet-doctor---include-pin-check--unify-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d4ec45fc
- **Timestamp:** 2026-05-17T19:22:38Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 3

**Per-AC findings:**

- **AC#1 (Agent)** — `FleetAction::Doctor` in `crates/termlink-cli/src/cli.rs` gains an `--include-pin-check` flag (off by default — additive). When set, fleet doctor additionally probes each hub via TLS and compares wire
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/cli.rs in: `FleetAction::Doctor` in `crates/termlink-cli/src/cli.rs` gains an `--include-pin-check` flag (off by default — additive). When set, fleet doctor addi`
- **AC#2 (Agent)** — `cmd_fleet_doctor` in `crates/termlink-cli/src/commands/remote.rs` accepts the new parameter, runs probes in parallel per hub via `tokio::spawn(probe_cert(addr))` reusing the T-1660 pattern. **Verifie
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/remote.rs in: `cmd_fleet_doctor` in `crates/termlink-cli/src/commands/remote.rs` accepts the new parameter, runs probes in parallel per hub via `tokio::spawn(probe_`
- **AC#7 (Agent)** — `cargo build --release -p termlink` succeeds. `cargo clippy -p termlink -- -D warnings` does not introduce new clippy violations in the touched file. **Verified:** build clean in 6m26s. Workspace clip
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink-mcp/src/tools.rs in: `cargo build --release -p termlink` succeeds. `cargo clippy -p termlink -- -D warnings` does not introduce new clippy violations in the touched file. `

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet doctor`

### 2026-05-17T19:16:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
