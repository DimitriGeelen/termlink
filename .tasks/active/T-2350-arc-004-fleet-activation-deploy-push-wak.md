---
id: T-2350
name: "arc-004 fleet activation: deploy push-wake binary to ring20 hubs + restart"
description: >
  arc-004 fleet activation: deploy push-wake binary to ring20 hubs + restart

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
created: 2026-07-04T11:16:45Z
last_update: 2026-07-04T11:19:36Z
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

# T-2350: arc-004 fleet activation: deploy push-wake binary to ring20 hubs + restart

## Context

arc-004 (push-transport) is closed=shipped, but the shipped dm-rail push-wake
(T-2323 hub `dm.queued` emit + T-2324 waker dm rail + T-2340 WS re-probe) is only
LIVE on the .107 hub (restarted 2026-07-03). Sweep evidence (this session):
ring20-management (.122) hub pid 2296932 runs a stale deleted binary
(`/usr/local/bin/termlink (deleted)`, grep dm.queued = 0) AND the on-disk
0.11.1367 binary ALSO predates T-2323 (grep dm.queued = 0) — the T-2184/PL-209
stale-binary class. ring20-dashboard (.121) is older still (governor_status RPC
fails with pre-T-2048 signature). laptop-141 unreachable (no route).
Local `target/release/termlink` (0.11.270, built 2026-07-04, zero source commits
since) carries the full arc: dm.queued grep = 1.

Plan: deploy the current binary to .122 (chunked base64 over remote exec, PL-096),
restart its hub (runtime_dir already /var/lib/termlink per T-1294 —
persist-if-present preserves secret/cert, no PL-021 exposure), verify dm.queued
served + TLS fingerprint unchanged + spokes healthy. Then .121 if a management
path exists (0 registered sessions — remote exec unavailable). PL-200 caveat:
agent-presence listeners on swapped hosts need re-registration.

## Acceptance Criteria

### Agent
- [ ] Current-code binary (dm.queued grep=1) installed at /usr/local/bin/termlink on .122, sha256 matches the local source binary
- [ ] .122 hub restarted and serving the dm-rail: running hub pid's /proc/<pid>/exe grep dm.queued = 1, exe NOT "(deleted)"
- [ ] No auth/TLS rotation caused: .122 hub.secret + hub.cert.pem unchanged (same mtime/hash), `termlink fleet verify` reports match for ring20-management, fleet doctor shows no auth-mismatch
- [ ] Post-restart hub functional: `termlink remote list 192.168.10.122:9100` succeeds; governor_status serves cv_index fields
- [ ] .121 (ring20-dashboard) dispositioned: upgraded via an existing management path, or explicitly documented as unreachable-for-deploy with the blocker named
- [ ] PL-200 follow-up recorded: presence-listener re-registration need on .122 either verified n/a or relayed to the ring20 agent via DM

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
out=$(timeout 40 termlink remote exec 192.168.10.122:9100 tl-dzbcxxka 'HP=$(cat /var/lib/termlink/hub.pid); grep -a -c dm.queued /proc/$HP/exe' --json 2>&1); echo "$out" | grep -q '"stdout": "1'
out=$(timeout 30 termlink fleet verify 2>&1); echo "$out" | grep -q "ring20-management.*match"

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

### 2026-07-04 — swap+restart mechanism for .122
- **Chose:** fleet-deploy-binary.sh for staging+probe ONLY; hand-rolled detached
  swap+restart script with hardcoded `TARGET=/usr/local/bin/termlink` and
  `TERMLINK_RUNTIME_DIR=/var/lib/termlink`.
- **Why:** two defects in `--swap-restart` for this host's state: (1) it resolves
  TARGET via `readlink /proc/PID/exe`, which returns `/usr/local/bin/termlink
  (deleted)` here (stale-binary state) — the swap would create a file literally
  named `termlink (deleted)` and leave the real path on the old build; (2) its
  relaunch falls back to `$HOME/.termlink/runtime` when the exec-channel env
  lacks TERMLINK_RUNTIME_DIR — silently rotating hub.secret/cert (PL-021 class).
- **Rejected:** `--swap-restart` as-is (above); systemd unit install (larger
  change than this task needs — hub is historically launched detached on .122;
  unit migration is T-935 playbook territory, separate task if wanted).
- **Recovery path verified BEFORE restart:** SSH root@192.168.10.122 works
  (BatchMode) — if the hub fails to relaunch, repair goes over SSH.

### 2026-07-04 — .121 (ring20-dashboard) disposition
- **Chose:** relay upgrade request to ring20 agent via DM; no direct deploy.
- **Why:** no management path from here: 0 registered sessions (remote exec
  impossible), SSH denied from .107 AND from .122 (BatchMode publickey), and
  the accessible hypervisor (.180 "proxmox") does not host the ring20 CTs.
- **Rejected:** password/interactive SSH (no credentials, not agent-appropriate).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-04T11:16:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2350-arc-004-fleet-activation-deploy-push-wak.md
- **Context:** Initial task creation
