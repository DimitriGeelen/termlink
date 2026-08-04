---
id: T-2529
name: "command.execute captures child output unbounded — daemon-OOM on a huge-output command (no MAX_LINE_BYTES twin for exec)"
description: >
  executor::execute uses cmd.output() which drains child stdout+stderr fully into Vec<u8> with no size cap; a yes/cat-/dev/zero (or accidental cat biglog) grows the long-lived daemon's heap until OOM. Command string is capped (MAX_COMMAND_LEN) but output is not. Fix: cap captured output at MAX_OUTPUT_BYTES, kill child + flag truncated on overflow.

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
created: 2026-08-04T13:05:53Z
last_update: 2026-08-04T13:06:06Z
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

# T-2529: command.execute captures child output unbounded — daemon-OOM on a huge-output command (no MAX_LINE_BYTES twin for exec)

## Context

**Reliability/resource defect on the CORE "control terminal sessions" verb (T-2468 incomplete-core lens).**
`termlink_session::executor::execute` (`crates/termlink-session/src/executor.rs:65`)
runs a shell command via `tokio::process::Command` and captures its output with
`cmd.output()` (line 99). `Command::output()` drains the child's **entire** stdout AND
stderr into `Vec<u8>` (`output.stdout` / `output.stderr`) with **no size cap**, then
returns them as `ExecResult.stdout` / `.stderr` (lines 104-108). The command *string*
is capped at `MAX_COMMAND_LEN` (65_536, line 15/23) — but the *output* has no
equivalent of the bus's `MAX_LINE_BYTES` (16 MiB) or `MAX_PAYLOAD_SIZE`.

A single `execute` of an unbounded producer (`yes`, `cat /dev/zero`, or an **accidental**
`cat biglog` / `find /`) grows **termlink's own daemon heap** at pipe speed until the
long-lived hub/session process is OOM-killed — taking the supervisor down, not a
transient child. This is amplification over "you already had a shell": the buffer lives
in the daemon's address space. It is BOTH a security DoS (authenticated `Execute`-scope
caller, reachable remotely via hub-forwarded RPC / `termlink_remote_exec` / MCP
`termlink_exec` / `termlink_dispatch`) AND a plain Reliability bug absent any malice.

**Reachability:** authenticated, `PermissionScope::Execute` (not pre-auth). The default
30s timeout already lets `yes` produce multiple GB; the caller-supplied timeout also has
no upper clamp, but the OUTPUT cap is the load-bearing fix — once output hits the cap the
child is killed regardless of timeout.

**Already-hardened siblings (honest scope):** the PTY/interact path is bounded — a 1 MiB
`VecDeque` scrollback ring (`scrollback.rs`); `kill_on_drop(true)` (line 95, T-2509)
already prevents timed-out-child orphaning. The one-shot `execute()` capture is the lone
hole. The timeout upper-clamp is a distinct smaller item (one-bug-one-task) and an
amplifier of THIS bug, not independent — noted, not bundled.

**Fix (small, in-authority — mirrors `MAX_LINE_BYTES` / `MAX_COMMAND_LEN`; no design
decision; no legitimate-use tension — no orchestrator needs unbounded *synchronous
captured* output; large output belongs on the ring-bounded PTY path or streamed):**
`cmd.output()` cannot cap mid-stream, so spawn with piped stdio and read stdout+stderr
**concurrently** each bounded to `MAX_OUTPUT_BYTES + 1` via `AsyncReadExt::take` (pipe
backpressure prevents OOM once we stop reading); on overflow kill the child and set a new
`ExecResult.truncated` flag; otherwise `child.wait()` for the real exit code. Thread the
cap through a private `execute_capped(..., max_output_bytes)` so a test can inject a tiny
cap without producing 16 MiB (mirrors the T-2525 `put_streaming_capped` pattern).

Origin: T-2468 subtract-and-deepen campaign, adversarial session-control resource-hunter
firing. Predecessors: T-2509 (kill_on_drop), the bus `MAX_LINE_BYTES` / `MAX_PAYLOAD_SIZE`
convention, T-2524/T-2525/T-2526 (this window's other exhaustion caps).

## Acceptance Criteria

### Agent
- [x] `executor::execute` no longer uses uncapped `cmd.output()`; it spawns with piped stdio and reads stdout+stderr concurrently (chunked `tokio::select!` loop), each bounded to `MAX_OUTPUT_BYTES` (16 MiB, mirroring `MAX_LINE_BYTES`), killing the child on overflow
- [x] `ExecResult` gains a `truncated: bool` field (`#[serde(default)]` for wire back-compat); it is `true` iff either stream hit the cap, `false` on normal completion
- [x] A private `execute_capped(..., max_output_bytes)` carries the cap; `execute()` delegates with `MAX_OUTPUT_BYTES` — so a test injects a tiny cap without emitting 16 MiB
- [x] Load-bearing test: `execute_kills_infinite_producer_promptly` (`yes`, cap 4096) returns `truncated==true` + bounded output within a 5s outer bound (child killed, no hang); `execute_truncates_over_cap` (bounded 500KB, cap 256) → `truncated==true`, `stdout.len()<=256`; `execute_no_truncate_under_cap` (`echo hi`) → `truncated==false`, full output
- [x] Load-bearing proof: neutralizing the cap check (`if false`) made `execute_truncates_over_cap` FAIL on its assertion; restore → 27/27 executor tests green. `cargo build -p termlink-session` clean; existing exec tests (echo/env/cwd/timeout/kill) pass; dependent crates termlink-mcp/termlink/termlink-hub build clean (new field broke no consumer)

<!-- Human section removed — fully agent-verifiable (a Rust cap + async producer test). -->

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
cargo build -p termlink-session
cargo test -p termlink-session --lib executor

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

**Symptom:** A single authenticated `command.execute` (or MCP `termlink_exec` /
`termlink_dispatch` / `termlink_remote_exec`) of an unbounded-output command — `yes`,
`cat /dev/zero`, or an accidental `cat biglog` / `find /` — grows the termlink
hub/session daemon's heap at pipe speed until the OS OOM-kills the long-lived process.
The supervisor dies, not just a transient child.

**Root cause:** `executor::execute` captured output with `cmd.output()`
(executor.rs:99), which reads the child's full stdout+stderr into `Vec<u8>` with no size
bound. The command *string* was capped (`MAX_COMMAND_LEN`, T-abuse guard) but the
*output* never was — no `MAX_LINE_BYTES`/`MAX_PAYLOAD_SIZE` twin on the exec path.

**Why structurally allowed:** the "control terminal sessions" verb family grew its
resource guards piecemeal — command length capped, timed-out children killed (T-2509,
kill_on_drop), PTY scrollback ring-bounded (1 MiB) — but the one-shot capture buffer's
size was never bounded. `cmd.output()` reads as "the convenient way to run a command",
hiding that it is an unbounded accumulation of peer-influenced bytes into the daemon's
own heap. No test drove a large-output command, so the hole stayed invisible.

**Prevention:** (1) the fix — cap captured output at `MAX_OUTPUT_BYTES`, killing the
child + flagging `truncated` on overflow. (2) a load-bearing test that runs a
producer exceeding an injected cap and asserts truncation + prompt return, so a
regression to unbounded reading fails CI. Broader structural note: this is the same
"peer-influenced unbounded accumulation into our own address space" class as T-2524
(artifact download) / T-2525 (hub staging) / T-2518 (bus line) — the recurring lesson
is that every place daemon code buffers externally-driven bytes needs an explicit cap;
a future canary that flags `Command::output()` / unbounded `read_to_end` on
externally-driven streams in daemon crates would surface the class (sibling of T-2527's
alloc-sink check; logged as a thought, not built here — one-bug-one-task).

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

### 2026-08-04T13:05:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2529-commandexecute-captures-child-output-unb.md
- **Context:** Initial task creation

### 2026-08-04T13:06:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-04 — built + verified [T-2468 subtract-and-deepen campaign, session-control resource-hunter firing]
- **Fix:** replaced `cmd.output()` (unbounded full stdout+stderr into Vec<u8>) with a `execute_capped(..., max_output_bytes)` private impl — piped stdio + a chunked `tokio::select!` read loop that appends 8KB chunks and, the instant either buffer exceeds `MAX_OUTPUT_BYTES` (16 MiB, mirrors bus `MAX_LINE_BYTES`), truncates + kills the child (so a flood on ONE stream triggers an immediate kill, not a block on the OTHER stream's EOF). `ExecResult` gains `truncated: bool` (`#[serde(default)]`). `execute()` delegates with the default cap.
- **Verified in code first (hunter can be wrong):** confirmed `cmd.output()` at executor.rs:99 was uncapped, command string WAS capped (MAX_COMMAND_LEN), PTY scrollback IS ring-bounded (1 MiB, so the PTY path is clean), and ExecResult is constructed at exactly ONE site with no exhaustive-match consumers → adding a field is low blast radius (mcp/cli/hub all rebuild clean).
- **Load-bearing:** `execute_truncates_over_cap` (bounded 500KB producer, cap 256 — bounded on purpose so the revert proof can't OOM) asserts `truncated==true` + `stdout.len()<=256`; neutralizing the cap check (`if false`) → full read, `truncated==false` → test FAILS; restored → 27/27 executor tests green. Plus `execute_kills_infinite_producer_promptly` (`yes`) proves the real DoS scenario (infinite producer killed within 5s), and `execute_no_truncate_under_cap` proves normal output is unaffected.
- **Scope honesty:** the caller-supplied `timeout` still has no upper clamp (handler.rs) — an AMPLIFIER of this bug, not independent; the output cap neutralizes the OOM vector regardless of timeout. Noted as a distinct one-bug-one-task item, not bundled.
- **Follow-up thought (NOT built, one-bug-one-task):** the class is "peer-influenced unbounded accumulation into the daemon's own address space" (T-2524 artifact-download / T-2525 hub-staging / T-2518 bus-line / this). A canary flagging `Command::output()` / unbounded `read_to_end` on externally-driven streams in daemon crates would surface it whole (sibling of T-2527's alloc-sink check).
