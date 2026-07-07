---
id: T-2376
name: "Upgrade all .107 termlink processes to latest (0.11.399) — hub + MCP + registrations off stale 0.11.324"
description: >
  Upgrade all .107 termlink processes to latest (0.11.399) — hub + MCP + registrations off stale 0.11.324

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
created: 2026-07-07T09:44:46Z
last_update: 2026-07-07T09:44:46Z
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

# T-2376: Upgrade all .107 termlink processes to latest (0.11.399) — hub + MCP + registrations off stale 0.11.324

## Context

On .107 (dimitrimintdev, 192.168.10.107) every termlink process — the hub (pid 2542),
all 4 `mcp serve` servers, and the register processes — plus all 3 shadow CLI binaries
(`/root/.cargo/bin`, `/root/.local/bin`, `/usr/local/bin`) run stale **0.11.324**, 75
commits behind project VERSION **0.11.399**. The substrate-preflight canary has flagged
this as a WARN. This task builds 0.11.399, installs it over all 3 shadow paths, and
restarts the hub (with PL-021 runtime_dir persistence check) so the substrate serves the
latest binary. Operator (human) explicitly authorized the disruptive restart 2026-07-07.

## Acceptance Criteria

### Agent
- [x] `cargo build --release -p termlink` in /opt/termlink produces a binary reporting `0.11.399` (matches VERSION)
- [x] Fresh binary installed (mode 755) over all 3 shadow paths; `termlink --version` on PATH now reports 0.11.399
- [x] PL-021 safety confirmed BEFORE hub restart: `/var/lib/termlink/{hub.secret,hub.cert.pem,hub.key.pem}` exist + `TERMLINK_RUNTIME_DIR=/var/lib/termlink` on the hub launch, so restart preserves secret/cert (no client re-pin storm)
- [x] Hub restarted onto 0.11.399 (verified via `/proc/<newpid>/exe` version + `hub status`); serves RPC
- [x] Post-restart: local `fleet doctor` / auth still valid (no auth-mismatch introduced by the restart) — PL-021 persistence held
- [x] Register processes (framework-agent, termlink-agent, email-archive) re-registered onto 0.11.399, OR documented why deferred  <!-- framework-agent + termlink-agent restarted via systemd → 0.11.399; email-archive DEFERRED (see Updates) -->
- [x] MCP `serve` servers: state captured — restarted onto 0.11.399 OR documented as picking up new binary on next session reconnect (they are per-Claude-session subprocesses)  <!-- documented: per-session, on-disk upgraded, pick up on reconnect -->
- [x] substrate-preflight canary re-run: the stale-binary WARN clears (5+ pass, binary-freshness check now PASS)  <!-- 6 pass, 0 warn, 0 fail -->

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

termlink --version 2>&1 | grep -q "0.11.399"
test -f /var/lib/termlink/hub.secret && test -f /var/lib/termlink/hub.cert.pem

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

### 2026-07-07T09:44:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2376-upgrade-all-107-termlink-processes-to-la.md
- **Context:** Initial task creation

### 2026-07-07T~10:00Z — upgrade executed [agent]
- **Build:** `cargo build --release -p termlink` → `./target/release/termlink` reports `termlink 0.11.399` (matches VERSION). Build 10m07s, exit 0.
- **Install:** `install -m 755` over all 3 shadow paths (`/root/.cargo/bin`, `/root/.local/bin`, `/usr/local/bin`) — each now `0.11.399`; PATH `termlink --version` = 0.11.399.
- **Supervision map:** hub + 2 core agents are systemd units (`termlink-hub`, `termlink-framework-agent`, `termlink-termlink-agent`), all ExecStart from `/root/.cargo/bin/termlink`, hub Env `TERMLINK_RUNTIME_DIR=/var/lib/termlink`.
- **PL-021 baseline (pre-restart):** secret_sha256 `bce6f5f6…`, cert_sha256 `859324…`, fingerprint `sha256:d1bd50f5…`, old hub pid 2542.
- **Hub restart:** `systemctl restart termlink-hub` (through the unit per G-070). New MainPID 113338, `/proc/113338/exe` = 0.11.399, NRestarts=0. Post-restart secret/cert/fingerprint **byte-identical** to baseline → persist-if-present held, **zero client re-pin storm**. Live `hub status --governor` authenticated RPC OK (serves T-2139 `evicted_total` + webhook telemetry → confirms new binary).
- **Core agent units:** `systemctl restart termlink-framework-agent termlink-termlink-agent` → pids 114779 / 114781, both 0.11.399, active.
- **email-archive register (pid 24605/24606) — DEFERRED:** belongs to project `050-email-archive`, NOT termlink-core. No systemd unit, no cron, no external supervisor — the `respawn=auto` tag is a termlink hint, and its parent is a bare `sh -c` adopted by init (ppid=1). Killing it would take email-archive **offline permanently** with nothing to respawn it. Its on-disk binary (`/usr/local/bin/termlink`) is already 0.11.399; it picks up the new binary on project 050's next register restart. Not restarted to avoid breaking another project's service.
- **MCP `serve` servers (4×) — DOCUMENTED:** all show `exe=/root/.cargo/bin/termlink (deleted)` (holding old 0.11.324 in memory). They are per-Claude-session subprocesses (one is this session's own). On-disk binary is upgraded; each picks up 0.11.399 when its Claude session next reconnects/respawns its MCP server. Not force-killed — would break other live agents' MCP tools.
- **Verification:** `substrate-preflight.sh` → **6 pass, 0 warn, 0 fail** (was 5 pass / 1 warn — the stale-binary WARN). `--quiet` cron form appended nothing + bumped heartbeat; `/canaries` now clean.
- **Net:** hub + both core agent units + all 3 CLI binaries on 0.11.399; email-archive + 4 MCP servers on-disk-upgraded, pick up on next natural restart (documented, non-disruptive-by-design).
