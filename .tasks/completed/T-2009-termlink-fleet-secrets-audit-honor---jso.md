---
id: T-2009
name: "termlink fleet secrets-audit: honor --json on --target-cache-without-check-drift usage error"
description: >
  termlink fleet secrets-audit: honor --json on --target-cache-without-check-drift usage error

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T21:07:28Z
last_update: 2026-06-05T21:07:28Z
date_finished: 2026-06-05T21:31:37Z
---

# T-2009: termlink fleet secrets-audit: honor --json on --target-cache-without-check-drift usage error

## Context

Sibling slice to T-2008. After scanning all `std::process::exit(2)` sites in
the CLI commands (4 total: help.rs:124 fixed by T-2008, infrastructure.rs:1312
and remote.rs:2810 are intentional exit-code-as-signal — already json-aware
upstream), one remaining usage-error case at `remote.rs:6488` in
`cmd_fleet_secrets_audit` drops `--json`:

```rust
if target_cache.is_some() && check_drift.is_none() {
    eprintln!("error: --target-cache requires --check-drift <PATH> ...");
    std::process::exit(2);
}
```

The function already takes `json: bool` but never consults it in this early
bail. Scripts wrapping `termlink fleet secrets-audit --json --target-cache X`
without `--check-drift` get empty stdout + stderr text + exit 2.

Same fix shape as T-2008: when `json` is true, emit
`{"ok": false, "error": "<msg>"}` on stdout; preserve plain-text stderr
behavior for non-JSON callers; exit 2 unchanged.

## Acceptance Criteria

### Agent
- [x] **Bug reproduced at HEAD before fix:** `termlink fleet secrets-audit --json --target-cache /tmp/x.hex` (without `--check-drift`) emits plain stderr text and exits 2 with EMPTY stdout.
- [x] **Fix:** `crates/termlink-cli/src/commands/remote.rs::cmd_fleet_secrets_audit`'s early bail emits a JSON envelope on stdout when `json` is true. Shape matches T-2008 / execution.rs / identity.rs convention: `{"ok": false, "error": "<msg>"}`. Exit code stays 2.
- [x] **Plain path unchanged:** non-JSON call still prints to stderr (byte-for-byte identical message) and exits 2.
- [x] **Live smoke (release build):** all three forms pass:
  - JSON mode: stdout = valid JSON `{ok:false, error:"...--check-drift..."}`, stderr empty, exit 2
  - Plain mode: stdout empty, stderr non-empty, exit 2
  - Happy path unaffected: `termlink fleet secrets-audit --json` (no incompatible flags) still emits a normal audit envelope, exit 0.
- [x] **Full CLI test suite** (`cargo test -p termlink --bins --quiet`) green at 814+ in suite; pre-existing `isolate_rejects_non_git_dir` flake acceptable if passes solo.

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

cargo build --release --bin termlink --quiet
test -x target/release/termlink
python3 -c "import subprocess,json; r=subprocess.run(['target/release/termlink','fleet','secrets-audit','--json','--target-cache','/tmp/nonexistent.hex'],capture_output=True); d=json.loads(r.stdout); assert d['ok'] is False and '--check-drift' in d['error'], d; assert r.returncode==2, r.returncode; assert r.stderr==b'', r.stderr"
python3 -c "import subprocess; r=subprocess.run(['target/release/termlink','fleet','secrets-audit','--target-cache','/tmp/nonexistent.hex'],capture_output=True); assert r.returncode==2, r.returncode; assert r.stdout==b'', r.stdout; assert b'--check-drift' in r.stderr, r.stderr"

## RCA

**Symptom:** `termlink fleet secrets-audit --json --target-cache /path/x.hex`
(without `--check-drift`) produced empty stdout + plain-text stderr + exit 2.
Scripts wrapping this verb to harvest the JSON envelope got nothing parseable.

**Root cause:** `cmd_fleet_secrets_audit` accepts `json: bool` but its
T-1824-introduced early bail (`if target_cache.is_some() && check_drift.is_none()`)
predates the JSON envelope contract. The bail used a direct
`eprintln!` + `std::process::exit(2)` without consulting `json` — same
code-flow-ordering bug as T-2008 in `commands/help.rs`.

**Why structurally allowed:** Two compounding factors:
1. The T-1824 ship pre-dated the T-1914 / T-1915 / T-1917 `--json`
   error-path audit (which scoped to hub-down operational errors only).
2. The follow-up T-2008 sister-fix didn't sweep beyond `commands/help.rs`
   even though the same pattern existed in `commands/remote.rs`. T-2009
   IS the sweep that should have happened then; the cycle-13 audit lens
   ("usage-error envelope parity") was scoped per-file, not per-codebase.

**Prevention:**
- **Live smoke in Verification block** asserts envelope shape + stderr
  emptiness on JSON-mode + exit code on both modes. Regression on the
  fix itself would fail the verification gate.
- **Convention reinforced:** two consecutive fixes (T-2008 + T-2009)
  applying the same shape — every CLI usage-error path with `json: bool`
  in scope MUST consult it before exiting. Pattern now well-established.
- **Codebase-wide audit completed:** the `grep -rn 'std::process::exit(2)' crates/termlink-cli/`
  ran at T-2009 scoping classified all 4 sites:
  - help.rs:124 → fixed by T-2008
  - infrastructure.rs:1312 → intentional exit-code signal (`no-pin` TOFU educational path; already json-aware upstream)
  - remote.rs:2810 → intentional exit-code signal (`fleet history --analyze` PL-021 detector; `emit_pl021_report` already takes `json_out`)
  - remote.rs:6488 → fixed by T-2009
  No further usage-error-without-json sites remain. Audit closed.
- **Follow-up captured if a third site emerges:** extract a shared
  `cli_usage_error_exit(json: bool, msg: &str) -> !` helper analogous to
  `json_error_exit`. Not filing eagerly — only worth doing when a third
  call site forces the abstraction.

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

## Recommendation

**Recommendation:** GO — ship the cycle-13 slice 6 + audit-close.

**Rationale:** Closes the final usage-error-without-json site in the CLI.
Same shape, same conventions, same testing pattern as T-2008. As a bonus,
T-2009's RCA carries the codebase-wide audit conclusion: 4 sites total,
2 fixed, 2 intentional exit-code-signal paths (already json-aware
upstream). The cycle-13 `--json` parity arc on usage errors is now
complete.

**Evidence:**
- Live smoke (all 3 paths): JSON mode emits valid envelope + stderr empty + exit 2; plain mode unchanged; happy `--json` path still emits normal audit envelope + exit 0
- Full CLI test suite: 816/0 in suite + known `isolate_rejects_non_git_dir` flake (passes solo)
- Audit walk: `grep -rn 'std::process::exit(2)' crates/termlink-cli/` enumerated all 4 sites; T-2009 closes the last user-visible one

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

### 2026-06-05T21:07:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2009-termlink-fleet-secrets-audit-honor---jso.md
- **Context:** Initial task creation
