---
id: T-2066
name: "fleet governor-status --watch --log <FILE> — append-only NDJSON transition log (T-2028 §6 #10 Track G, audit-trail axis)"
description: >
  fleet governor-status --watch --log <FILE> — append-only NDJSON transition log (T-2028 §6 #10 Track G, audit-trail axis)

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
created: 2026-06-08T23:16:33Z
last_update: 2026-06-08T23:34:27Z
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

# T-2066: fleet governor-status --watch --log <FILE> — append-only NDJSON transition log (T-2028 §6 #10 Track G, audit-trail axis)

## Context

T-2064 (Track E) + T-2065 (Track F) shipped the continuous-monitor + notify-hook
for `fleet governor-status --watch`. Operators can now leave a watch terminal
running AND get paged on transitions. What's still missing: a persistent audit
trail. When a 3am Slack message says "hub-a hit capacity at 03:42:17Z", the
operator opens a fresh terminal at 09:00 with no record of what happened.

T-1671 solved the analogous problem for rotation events: append one NDJSON line
to `~/.termlink/rotation.log` per `fleet doctor --watch` transition, then
`fleet history` reads it back. Track G adds the parallel append-only log for
governor transitions.

Schema is flat (jq-friendly), numeric where rotation.log was string, and nullable
for the pre-T-2049 `dedupe_hits` field that some hubs lack:

```json
{
  "ts": "2026-06-08T22:58:02Z",
  "hub": "workstation-107-public",
  "kind": "transition",
  "old_reach": "ok",
  "new_reach": "ok",
  "old_conn_active": 3, "new_conn_active": 4,
  "old_cap_hits": 0, "new_cap_hits": 0, "cap_hits_delta": 0,
  "old_rate_hits": 0, "new_rate_hits": 0, "rate_hits_delta": 0,
  "old_dedupe_hits": null, "new_dedupe_hits": null, "dedupe_hits_delta": null
}
```

A future `fleet governor-history` retrospective view can read this file
(deferred — out of T-2066 scope). For now operators `jq` it directly.

## Acceptance Criteria

### Agent
- [x] `FleetAction::GovernorStatus` gains a `--log <PATH>` flag, requires `--watch` (clap `requires`). — cli.rs `#[arg(long, value_name = "PATH", requires = "watch")] log: Option<std::path::PathBuf>`.
- [x] Pure helper `build_governor_log_entry(hub, kind, ts, prev, new) -> serde_json::Value` emits the flat NDJSON schema with numeric counters + nullable dedupe fields. — remote.rs ~line 2840; verified live: smoke produced `{ts, hub, kind, old_reach, new_reach, old_conn_active, new_conn_active, old_cap_hits, new_cap_hits, cap_hits_delta, old_rate_hits, new_rate_hits, rate_hits_delta, old_dedupe_hits, new_dedupe_hits, dedupe_hits_delta}` with `new_dedupe_hits: null`.
- [x] `append_governor_log(path, entry)` opens append + create, writes one `{}\n` line per call, parent dir auto-created if missing. — `OpenOptions::new().create(true).append(true)` + `write_all("{}\n".as_bytes())`; parent dir check at top with `create_dir_all`.
- [x] Write failures (disk full, permission denied) log to stderr but never crash the watch (best-effort, same shape as T-1671 `append_rotation_log`). — `if let Err(e) = res { eprintln!(...) }`; no `return Err` — function is fn(), not Result.
- [x] Watch loop fires the log append on every per-hub transition / new / removed (same gate as `--notify`, NOT on baseline). — three new `if let Some(path) = log.as_deref()` arms parallel to the notify arms; baseline branch unchanged. Live smoke: 1 transition fired during 22s watch ⇒ 1 NDJSON line in `/tmp/gov-test.log`.
- [x] ≥2 unit tests pin: counter delta math in the entry + dedupe-null serialization (None side renders as JSON null, not omitted). — `build_governor_log_entry_computes_deltas_and_string_reach` + `build_governor_log_entry_serializes_null_for_missing_sides`. 831 → 833 bin lib tests.
- [x] CLAUDE.md BACKPRESSURE row updated with `--log` example. — row 1170 paragraph extended with T-2066 description + dual recipe (`--watch 30 --log ~/.termlink/governor.log --notify /usr/local/bin/page-on-cap.sh`). jq retrospective example included.

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

cd /opt/termlink && cargo build -p termlink --release 2>&1 | tail -3
cd /opt/termlink && out=$(cargo test -p termlink --release --bin termlink build_governor_log_entry 2>&1); echo "$out" | tail -10; echo "$out" | grep -q "2 passed; 0 failed"
cd /opt/termlink && grep -q "governor.*--log\|--log.*governor" CLAUDE.md

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

### 2026-06-08T23:16:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2066-fleet-governor-status---watch---log-file.md
- **Context:** Initial task creation
