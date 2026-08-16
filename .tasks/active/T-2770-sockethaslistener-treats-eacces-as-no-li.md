---
id: T-2770
name: "socket_has_listener treats EACCES as no-listener — T-2767 guard is blind to the uid-coupled split-brain path"
description: >
  crates/termlink-hub/src/pidfile.rs::socket_has_listener does UnixStream::connect(socket).is_ok(). A non-root agent probing a root:root 0755 hub.sock gets EACCES, is_ok() is false, the guard reports no-listener and PERMITS a second hub to start. So the T-2767 guard misses precisely the permission-coupled case that produces split-brain; it only catches probers that could have connected anyway. Fix: branch on io::ErrorKind — ConnectionRefused means a dead socket file (start, preserving the unclean-shutdown case), PermissionDenied means a socket exists this uid may not probe, which is positive evidence of another user's hub (REFUSE, naming the uid mismatch), any other kind refuses conservatively and names the error. Never treat cannot-look as nothing-there. Regression test must pin the EACCES case specifically (chmod fixture socket 0700 and probe as another uid, or inject the ErrorKind); stale_socket_file_with_no_listener_still_starts already pins the refused case. Origin: AEF agent analysis of local IPC uid-coupling, 2026-08-16.

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
created: 2026-08-16T16:41:47Z
last_update: 2026-08-16T17:52:47Z
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

# T-2770: socket_has_listener treats EACCES as no-listener — T-2767 guard is blind to the uid-coupled split-brain path

## Context

T-2767 added `socket_has_listener` so `hub start` refuses when a hub is LIVE on the
socket even though the pidfile does not say so (the split-brain path). It decides with:

```rust
std::os::unix::net::UnixStream::connect(socket).is_ok()
```

`.is_ok()` collapses every error into "no listener". Two of those errors mean opposite
things:

| `connect()` result | truth | current verdict |
|---|---|---|
| `ECONNREFUSED` | nothing is listening; the socket file is stale | no listener ✓ |
| `EACCES` | a listener may well exist — we simply may not reach it | no listener ✗ |

So the guard is blind on exactly the path it was written to guard: when the live hub
belongs to a **different uid**, or the socket's mode denies us write, we conclude
"stale" and start a rival hub — manufacturing the split-brain T-2767 exists to prevent.

**This is no longer hypothetical (2026-08-16, .107).** Three hubs were running on one
host, and T-2772 established the mechanism: local Unix access is gated first by socket
file mode and then by a same-uid `SO_PEERCRED` check. Both gates produce exactly the
`EACCES` that this function reads as "nobody home". An agent that cannot reach the hub
starts its own — which is the fragmentation observed. The guard would have been silent
through all of it.

Note the interaction with T-2772's second gate: if the mode permits `connect()` but the
uid differs, `connect()` SUCCEEDS (the hub refuses at the application layer afterwards),
so `is_ok()` is true and the guard already refuses correctly. It is specifically the
**kernel-level `EACCES`** case that is misread. Fixing it means the guard fails CLOSED —
the same posture T-2448 chose for the uid gate itself.

## Acceptance Criteria

### Agent
- [x] `socket_has_listener` distinguishes `ECONNREFUSED` (genuinely stale — safe to
      start) from every other error (cannot rule out a listener — refuse)
- [x] `EACCES` / `PermissionDenied` results in a REFUSAL to start, not a start
- [x] An unexpected error kind also refuses (fail-closed by default, so a future libc
      error kind cannot silently re-open the hole)
- [x] The refusal names WHY it could not probe, distinctly from the plain
      "socket is alive" case — an operator must be able to tell "another hub is running"
      from "I could not check whether another hub is running"
- [x] A genuinely stale socket file (no listener, connect refused) still starts —
      the T-2767 behaviour is preserved, not traded away for the new safety
- [x] Unit tests cover: refused-connection starts, permission-denied refuses, and the
      two refusal messages differ
- [x] `cargo test -p termlink-hub --lib pidfile` passes — 26 passed, 0 failed
- [x] `cargo test --workspace` passes — 3600 passed, 0 failed, exit 0 (2026-08-16)

### Live proof, both directions (2026-08-16)

Reproduced the exact split-brain condition on a scratch runtime_dir: hub started as
root (socket lands `root:root 0755`), pidfile then removed to recreate T-2767's
"lost pidfile in front of a live hub", and a second `hub start` attempted as
`dimitri-mint-dev` (uid 1000, which cannot write the root-owned socket → `EACCES`).

**With the fix — refuses, and says why:**

```
$ su - dimitri-mint-dev -c 'TERMLINK_RUNTIME_DIR=/tmp/t2770-proof ... hub start'
Error: Hub server error
Caused by:
    A socket exists at /tmp/t2770-proof/hub.sock but this process could not probe
    it (Permission denied (os error 13)), so whether a hub is already serving there
    is UNKNOWN. Refusing to start rather than risk taking over a live socket and
    splitting the substrate in two. ...
```

**With the old semantics temporarily restored — starts, and hijacks the socket:**

```
Starting hub server...
INFO termlink_hub::server: Hub listening on Unix path=/tmp/t2770-proof/hub.sock
thread 'main' panicked at channel.rs:151: failed to open channel bus: readonly database

$ ls -la /tmp/t2770-proof/hub.sock
srwxrwxr-x 1 dimitri-mint-dev dimitri-mint-dev   # was root:root
```

The rival hub bound the socket and took ownership of the path **before** crashing on
an unrelated permissions error — so even a hub that immediately dies leaves the
original hub's socket path hijacked. That is the .107 fragmentation, reproduced on
demand in one command.

**Load-bearing at the unit level too:** under the reverted classifier,
`socket_probe_permission_denied_is_unknown_not_stale` and
`socket_probe_unexpected_error_kind_is_unknown` both FAIL; restoring returns them to
green. The temporary revert was removed and grep-confirmed absent before commit.

Scratch hub stopped and `/tmp/t2770-proof` removed. The pre-existing hubs on this host
were not touched.

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

out=$(cargo test -p termlink-hub --lib pidfile 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-hub --lib socket_probe 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** `hub start` succeeds and takes over a socket that a live hub owned by a
different uid is already serving on, producing two hubs on one host — the exact
split-brain the T-2767 guard was written to prevent, with the guard silent throughout.

**Root cause:** `socket_has_listener` decided with `UnixStream::connect(..).is_ok()`.
That maps `EACCES` — "a listener may exist, I cannot reach it" — onto the same verdict
as `ECONNREFUSED` — "nothing is listening". The guard therefore only ever caught
listeners it could have connected to anyway, and was blind to every listener it could
not reach, which is the population it most needed to detect.

**Why structurally allowed:** T-2767 was written and tested against the incident that
motivated it — a *same-uid* hub with a lost pidfile — where `connect()` succeeds and
`is_ok()` is the right answer. Both of its socket tests (`live_socket_with_no_pidfile_refuses`,
`stale_socket_file_with_no_listener_still_starts`) exercise same-uid sockets, so the
suite was green and complete-looking while the cross-uid case was never expressed. The
deeper reason is that `is_ok()` is a *lossy* read of a three-valued outcome: yes / no /
cannot-tell. Collapsing "cannot tell" into "no" is an implicit fail-OPEN, and nothing
in the type or the test names made that visible. Note also that no test could have
caught it on a host running the suite as root, since root bypasses the permission that
produces the error.

**Prevention:** the outcome is now a three-valued `SocketProbe` (`Listening` / `Stale` /
`Unknown`), so "cannot tell" has to be handled explicitly at the call site and cannot be
silently absorbed. Classification is split into a pure `classify_connect` so the EACCES
policy is testable without producing a real EACCES — necessary because this suite runs
as root on some hosts. `socket_probe_unexpected_error_kind_is_unknown` pins the default
arm, so a future error kind that is not enumerated fails closed rather than re-opening
the hole. Verified load-bearing by reverting and observing both the unit failures and a
real socket hijack.

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

### 2026-08-16T16:41:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2770-sockethaslistener-treats-eacces-as-no-li.md
- **Context:** Initial task creation

### 2026-08-16T17:52:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
