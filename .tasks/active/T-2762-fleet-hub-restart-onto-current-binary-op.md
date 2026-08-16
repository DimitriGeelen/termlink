---
id: T-2762
name: "Fleet hub restart onto current binary (operator-authorized)"
description: >
  Operator authorized a fleet-wide hub restart to clear the T-2359 staleness warnings. Recon: only the local hub (.107, 0.11.720) has a usable foothold. .121 (0.11.588) refuses SSH publickey; .122 (0.11.679) answers SSH with a forced-command token broker and no shell; .141 is unreachable (no route). Scope is therefore: build+install current binary locally, restart the local hub THROUGH systemd per G-070, verify no PL-021 auth rotation, and report per-hub blockers for the three unreachable hubs.

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
created: 2026-08-16T12:19:55Z
last_update: 2026-08-16T12:25:58Z
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

# T-2762: Fleet hub restart onto current binary (operator-authorized)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

Operator authorized a fleet-wide hub restart onto the current binary, to clear the
T-2359 binary-staleness condition. Preflight reported the local binary and hub at
`0.11.720` against project VERSION `0.11.1411`.

**Fleet recon (2026-08-16).** `fleet doctor`: 5 profiles, 4 reachable.

| hub | addr | version | foothold |
|---|---|---|---|
| workstation-107-public / local-test | .107 / 127.0.0.1 | 0.11.720 | LOCAL — systemd unit, full control |
| ring20-management | .122 | 0.11.679 | SSH answers but yields a forced-command token broker, no shell |
| ring20-dashboard | .121 | 0.11.588 | SSH `Permission denied (publickey)` — matches CLAUDE.md "exempt for lack of an upgrade foothold" |
| laptop-141 | .141 | unknown | DOWN — `No route to host` |

SSH is therefore not a fleet deployment path. The supported path is
`scripts/fleet-deploy-binary.sh` (T-1420/PL-096), which streams the binary over
`termlink remote exec` rather than SSH and so does not need a shell — but it does
need a **registered remote session** on the target hub.

**PL-100 constraint.** The deploy script defaults to the musl-static target
precisely because `target/release/termlink` is dynamically linked against the
build host's glibc and fails on older targets. Any push must be the musl build,
and must be `--probe`d before swapping.

**PL-021 constraint.** A hub restart rotates the HMAC secret and TLS cert if its
`runtime_dir` is volatile. Local preflight reports
`/var/lib/termlink on btrfs (disk-backed)`, so the local restart should be
non-rotating — a claim to VERIFY after the fact, not assume.

**Cross-project boundary.** `.121` and `.122` are ring20 hosts carrying LIVE
ring20 agents (`ring20-concierge`, `ring20-dashboard-agent`). Restarting another
project's hub is not implied by "restart the fleet" and is not attempted without
that project's operator; the deliverable for those hubs is a precise blocker
report, not an improvised restart.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Current binary is built and installed locally; `termlink --version` reports the project VERSION
- [x] The local hub is restarted THROUGH its systemd unit (G-070), never as a detached process
- [x] Post-restart, the hub serves the new version (`fleet doctor` on the local profile)
- [x] NO PL-021 rotation: the TLS fingerprint and HMAC secret are identical before and after the restart, evidenced by captured values — not assumed from the runtime_dir being disk-backed
- [x] `substrate-preflight.sh` Checks 4 and 5 (binary / hub-binary staleness) both return to PASS
- [x] Remote-session availability is probed for `.121` and `.122` so the "no foothold" claim is measured, not inherited from documentation
- [x] Each unreachable hub has a named, specific blocker and the concrete action that would unblock it
- [ ] The fleet-binary canary floors are reviewed against the post-restart state so the canary reflects reality
- [ ] `.122` deployed and restarted onto the current binary (STOPPED at the budget gate — see below)

## Per-hub status and blockers

| hub | version | status | blocker / next action |
|---|---|---|---|
| workstation-107-public + local-test (.107) | **0.11.1411** | **DONE** | none — restarted, no auth rotation, preflight 6/0/0 |
| ring20-management (.122) | 0.11.679 | **READY, NOT DONE** | musl binary now built; run the deploy command below |
| ring20-dashboard (.121) | 0.11.588 | **BLOCKED** | no foothold by either route: SSH `Permission denied (publickey)` AND zero registered sessions. Needs either an SSH key installed on that host or a termlink session registered from it. Until then it cannot be upgraded remotely — this is what the CLAUDE.md floor exemption is really recording. |
| laptop-141 (.141) | unknown | **BLOCKED** | host down — `No route to host (os error 113)`. Nothing to deploy to; bring the host up first. |

## Where this stopped, and why

The budget gate blocked at ~286k tokens (95% of context) with the `.122` deploy
not yet started. That was a deliberate stop, not an interruption: the deploy is a
multi-step stage → probe → swap → restart → verify sequence against a LIVE hub
carrying three ring20 sessions, and beginning it without the headroom to verify
the result would risk leaving a half-staged binary on someone else's running hub.
Not starting is recoverable; a half-finished swap is not.

**Everything `.122` needs is already in place.** The musl-static binary finished
building (exit 0) at
`target/x86_64-unknown-linux-musl/release/termlink` — this is the PL-100-safe
artifact; do NOT push `target/release/termlink`, which is glibc-dynamic and will
fail on the target.

Resume with (from the worktree root):

```
bash scripts/fleet-deploy-binary.sh ring20-management --probe --swap-restart
```

`--probe` runs `<staged>--version` on the remote and aborts with exit 5 before any
swap if the binary cannot execute there — that is the guard against the PL-100 /
T-1422 failure mode, so do not drop it. The script auto-detects the remote session;
`tl-kufvjxsf` (`host=122,project=proxmox-ring20-management`) is the one used for the
read-only precondition probes.

**After the swap, verify the two things that matter:**
1. `termlink fleet doctor` shows `.122` at `0.11.1411`
2. `.122` did NOT rotate — its `hub.secret` / `hub.cert.pem` are on disk-backed
   `/var/lib/termlink` and `/tmp` is not tmpfs, so persist-if-present should hold;
   confirm rather than assume, because T-1294 recorded this exact host with a
   volatile `/tmp` and a rotation there breaks every client's pinning.

Note `.122` is watchdog-launched, NOT systemd — there is no `termlink-hub.service`
on it, so `systemctl restart` is not the mechanism there; `--swap-restart` is.

**Local hub — DONE, no rotation.** Captured either side of `systemctl restart termlink-hub`:

| | before | after |
|---|---|---|
| TLS fingerprint | `sha256:d1bd50f5cb03c4fd…` | `sha256:d1bd50f5cb03c4fd…` |
| `hub.secret` sha256 | `bce6f5f64fcc167ab7a0d12e` | `bce6f5f64fcc167ab7a0d12e` |

Identical, so persist-if-present held and **no client needs to re-pin**. Unit
`active`, `MainPID=3093442`, `NRestarts=0` (a clean start, not a crash-loop —
the G-070 signature). `substrate-preflight.sh` went from 4 pass / 2 warn to
**6 pass / 0 warn / 0 fail — substrate-ready**; `fleet doctor` shows both local
profiles at `0.11.1411`.

**The SSH-based "no foothold" reading was wrong for `.122`, and measuring corrected it.**
SSH to `.122` yields a forced-command token broker with no shell, which looks like
no foothold — but the supported deploy path is `fleet-deploy-binary.sh` over
`termlink remote exec`, which needs a registered session, not a shell. `.122` has
**three** (`tl-fj5gsdvb`, `tl-ucwfhj2o`, `tl-kufvjxsf`), so it is reachable.
`.121` has **zero** sessions AND refuses SSH, so it genuinely has no foothold by
either route — the CLAUDE.md exemption is accurate for `.121` only.

**`.122` restart preconditions verified before touching it** (T-1294 recorded this
exact host as having had a volatile `/tmp`, which would make a restart rotate both
secret and cert):
- `TERMLINK_RUNTIME_DIR=/var/lib/termlink`, directory present
- contains `hub.secret`, `hub.cert.pem`, `hub.key.pem` — persist-if-present has files to preserve
- `mount | grep " /tmp "` → not tmpfs
- no `termlink-hub.service`; only health/sweep/hygiene timers ⇒ watchdog-launched,
  so the restart path is the deploy script's `--swap-restart`, NOT `systemctl`

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

### 2026-08-16T12:19:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2762-fleet-hub-restart-onto-current-binary-op.md
- **Context:** Initial task creation

### 2026-08-16T12:20:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
