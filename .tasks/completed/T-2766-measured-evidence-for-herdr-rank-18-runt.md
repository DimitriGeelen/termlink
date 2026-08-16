---
id: T-2766
name: "Measured evidence for herdr rank 18: runtime_dir default divergence on this host"
description: >
  While checking messages, termlink hub status reported 'not running' on a host where systemd shows termlink-hub.service active and accepting TLS connections. That is the rank-18 class live: the four-step runtime_dir resolution in discovery.rs means a shell and the systemd unit can resolve DIFFERENT directories, so the CLI gives a confident wrong answer about the hub it is standing next to. Rank 18 is owner: agenthuman and this task does NOT decide it — it measures and records the evidence so the human go/no-go has data.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T14:56:29Z
last_update: 2026-08-16T15:01:09Z
date_finished: 2026-08-16T15:01:09Z
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

# T-2766: Measured evidence for herdr rank 18: runtime_dir default divergence on this host

## Context

Filed to gather evidence for herdr backlog **rank 18** (default `runtime_dir`,
`owner: human`). The trigger was `termlink hub status` reporting **"Hub: not
running"** on a host where `systemctl` showed `termlink-hub.service` **active** and
accepting TLS connections.

### Result on rank 18: NEGATIVE — the hypothesis was wrong

The hypothesis was a resolution divergence: shell and unit landing on different
`runtime_dir`s. Measured, they do not.

| source | `TERMLINK_RUNTIME_DIR` |
|---|---|
| systemd unit (`systemctl show -p Environment`) | `/var/lib/termlink` |
| running hub process (`/proc/3093442/environ`) | `/var/lib/termlink` |
| this interactive shell (`env`) | `/var/lib/termlink` |

All three agree, and `substrate-preflight.sh` confirms
`runtime_dir=/var/lib/termlink on btrfs (disk-backed, not wiped by tmpfiles)`.
This host is correctly configured on the persistent path via the T-935 unit
migration, so it contributes **no** evidence that the four-step default in
`discovery.rs:10-26` is what bit here. Rank 18's argument stands on its own
merits; it gains nothing from this incident, and saying otherwise would be
manufacturing support for a human-owned decision. Recorded per AC 6.

### What was actually wrong — two findings, neither about defaults

**F1 — detached ghost hub (G-070 class), split-brain on one runtime_dir.**
Two hubs were running against `/var/lib/termlink`:

- PID `3093442` — the supervised unit, `termlink hub start --tcp 0.0.0.0:9100 --json`, up 2h33m
- PID `3869961` — `termlink hub start` (no `--tcp`), started 16:52:36, holding `hub.pid` **and** `hub.sock`

The ghost's parent is **another Claude session's shell** on this host (a different
`shell-snapshot` id from mine), which ran `termlink hub start` manually. Effect: unix-socket
clients reach the ghost while TCP clients reach the supervised hub — the same topic name
resolves to two different instances (the G-060 per-hub-state property, but *within one
host*, which no one expects).

My first hypothesis was that my own `cargo test --workspace` had spawned it, since my shell
exports `TERMLINK_RUNTIME_DIR` and cargo inherits it. **Checked and disproved** by
`/proc/<pid>/environ` (no `CARGO_*`) and the parent's cmdline. Recorded because the
plausible-and-wrong version would have blamed the test suite for another session's action.

*Preflight Check 6 (T-2358) detected this correctly*, unprompted, naming both PIDs and the
remediation. A guard fed a real, unplanned instance of its own condition and firing is the
evidence standard PL-328 asks for — worth more than any number of green runs.

**F2 — `hub.secret` and `hub.cert.pem` are ABSENT from the runtime_dir while the hub runs.**
`ls -la /var/lib/termlink/` shows `hub.pid`, `hub.sock`, `bus/`, `sessions/`,
`route-cache.json`, `rpc-audit.jsonl` — and **neither** `hub.secret` **nor** `hub.cert.pem`.
`fleet doctor` fails the local profile with
`Secret file not found: /var/lib/termlink/hub.secret`, which is why local reads
(`agent-chat-arc-recent.sh`) reported `workstation-107-public` and `local-test` as network
failures while the two remote hubs answered fine.

**The load-bearing consequence, stated because it is easy to get wrong:** the running hub
holds both in memory. **Restarting it regenerates both**, which is PL-021's "BOTH rotate"
case and forces a fleet-wide re-pin. So the ordinary G-070 remediation ("restart through the
unit") is *exactly wrong here* until the secret is recovered or a planned rotation is
accepted. Deliberately NOT actioned: stopping the ghost belongs to the session that started
it, and restarting the supervised hub is a fleet-affecting decision for the operator.

Root cause of the absence is NOT established — I could not inspect `/root/.termlink/` for a
cached copy because the T-559 project-boundary hook blocks it, and I did not route around it.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The divergence is MEASURED, not asserted: record the runtime_dir the running
      systemd unit uses, the runtime_dir an interactive shell resolves, and whether
      each is volatile — with the commands and their actual output
- [x] The resolution path is traced to `discovery.rs` so the evidence names the
      mechanism, not just the symptom
- [x] The observable operator-facing consequence is stated exactly (what the CLI
      reports vs what is true), since that is what makes it a Directive #2 issue
      (a confident wrong answer, not an error)
- [x] Evidence is written into `.context/upstream/herdr-adoption-backlog.md` under
      rank 18 so it reaches the human decision, not only this task file
- [x] The task does NOT change the default, add a migration, or decide the go/no-go —
      rank 18 is `owner: human` and this is evidence only
- [x] If the divergence turns out to be operator misconfiguration rather than a
      default-resolution property, that negative result is recorded just as plainly

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

# The negative result reached the human decision, not just this task file.
grep -q "Negative evidence, 2026-08-16 (T-2766)" .context/upstream/herdr-adoption-backlog.md
# The rank-18 entry still states the item is human-owned and undecided.
grep -q "18. Default .runtime_dir. — rank 18, HUMAN-OWNED" .context/upstream/herdr-adoption-backlog.md
# Preflight still detects the ghost condition (the guard that caught this).
grep -q "detached ghost serving outside supervision" scripts/substrate-preflight.sh
# This task changed no product code — evidence only, per AC 5.
test -z "$(git diff --name-only HEAD -- crates/)"

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

### 2026-08-16T14:56:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2766-measured-evidence-for-herdr-rank-18-runt.md
- **Context:** Initial task creation

### 2026-08-16T15:01:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
