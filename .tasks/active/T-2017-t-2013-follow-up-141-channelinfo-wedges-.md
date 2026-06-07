---
id: T-2017
name: "T-2013 follow-up: .141 channel.info wedges over network despite fix"
description: >
  T-2013 fix landed on .141 (binary swapped to v0.11.806, hub restarted, LOCAL channel info instant proving worker starvation cured). But .107 client → .141 hub channel.info on agent-presence + agent-chat-arc both wedge at 15s timeout. Same client, same network: channel list (~100ms) and fleet doctor (~113ms) work fine. Hub log shows TCP accept + token authenticated (scope=execute) then handler stalls. Inside .141 the same channel info on the same topic completes in <500ms (instant). This is NOT the T-2013 worker starvation bug (which is structurally cured on the same binary running locally). It's a separate latent issue — possibly: WSL-on-Windows /mnt/c DrvFs filesystem latency on a code path that channel.list avoids, or response-size handling regression in v0.11.806 specific to channel.info envelope. Reproduce: from .107: timeout 15 termlink channel info agent-presence --hub 192.168.10.141:9100 → exit 124. Then from .141 agent-1 session: /home/dimitri/bin/termlink channel info agent-presence → instant. Investigate hub.rs::handle_channel_info_with: is there a code path that walks envelopes for sender summary on agent-presence (2412 posts, 1 sender)? Compare to handle_channel_list_with which works fine. .122 + .121 (same v0.11.806 binary, both LXC ext4) work end-to-end so it's NOT a generic regression — laptop-141 specific.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:T-2013, host:141, network]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-06T12:20:15Z
last_update: 2026-06-06T12:43:43Z
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

# T-2017: T-2013 follow-up: .141 channel.info wedges over network despite fix

## Context

T-2013 fix is structurally cured on .141 (local channel info instant; cured worker starvation). The residual surfaced today: .107 client → .141 hub wedges in a flaky, response-size-correlated pattern. Initial T-1991 framing was "0.11.473 channel info concurrency regression" — the post-T-2013 evidence shows the framing was incomplete: there is a SECOND, host-environmental wedge specific to .141 (WSL on Windows DrvFs/9p mount). Captured here so it doesn't get lost or mis-attributed to T-2013.

## Forensic Evidence (2026-06-06)

### Reproducer
```
# from .107 (192.168.10.107, native LXC client, v0.11.472):
for c in 0 1000 2000 3000; do
  termlink channel subscribe agent-presence --hub 192.168.10.141:9100 --limit 1000 --cursor $c --json | wc -l
done
# Result: cursor=0/1000 → 1000 lines @ 300ms; cursor=2000 → TIMEOUT @ 10s; cursor=3000 → 0 lines @ 100ms
```

### Bisection (limit-axis at cursor=2000)
| limit | result |
|---|---|
| 100 | OK 35KB @ 118ms |
| 200 | OK 71KB @ 135ms |
| 300 | OK 107KB @ 152ms |
| **400** | **WEDGE 10s timeout** |
| 500 | WEDGE 10s timeout |
| 800 | WEDGE 10s timeout |

Wedge threshold: response size ≥ ~140KB at any non-end cursor.

### Bisection (cursor-axis at limit=50)
All cursors 1900..2360 work at limit=50 (~17KB response). Cursor position is not the trigger — response size is.

### Flakiness signal
Same `cursor=0 limit=1000` call returned 1000 lines / 355890 bytes in 280ms in one test, then wedged at 10s on a later retry. **Non-deterministic** — matches T-1991's "~45% probability under repeated sequential use" exactly.

### Environmental
- .141 hub binary: `/mnt/c/ntb-acd-plugin/termlink/target/release/termlink` (Windows-mounted 9p DrvFs)
- 9p mount: `cache=5, msize=65536` — 64KB chunk size
- Bus data: `/home/dimitri/.termlink/runtime/bus/` (native ext4 — NOT involved)
- .121 + .122 (same v0.11.806 binary, native LXC ext4 binary location): 5/5 channel info sub-second, never flaky

### Local-vs-network on .141
- From INSIDE .141 (`/home/dimitri/bin/termlink channel info agent-presence` via Unix socket): instant
- From .107 over TCP → .141 hub: 15s wedge on large response

## Acceptance Criteria

### Agent
- [ ] Hypothesis tested: copy hub binary to /home/dimitri/bin/ (native ext4) + restart hub from local-fs path. If .107→.141 channel info now works deterministically at ≥1000-envelope responses, the wedge is WSL 9p-related (binary text segment paging on /mnt/c via 9p)
- [ ] If hypothesis confirmed: document the runbook ("WSL hosts: deploy binary to ~/bin not /mnt/c") in `docs/operations/wsl-host-deployment.md` + cross-reference from CLAUDE.md
- [ ] If hypothesis disproven: capture strace -p $(pgrep -f 'termlink hub') during the wedge from inside .141 (look for read on /mnt/c vs futex_wait vs network syscalls), narrow further
- [ ] Update T-1991 / T-2012 cross-reference: the "~45% probability" was a TWO-bug confound — T-2013 cured the LXC wedge; this task cures the WSL wedge
- [ ] PL-XXX learning registered: "WSL DrvFs/9p hosts behave differently from LXC ext4 hosts for TermLink hub binary placement"

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

**Symptom:** From .107 (LXC client), `channel info` and large `channel subscribe` calls (≥400 envs / ≥140KB response) targeting .141 hub wedge at 10-15s timeout. Same calls work fine targeting .121 / .122 hubs (same v0.11.806 patched binary). Same calls work fine from INSIDE .141 via Unix socket. Pattern is FLAKY — sometimes succeeds, sometimes wedges.

**Root cause (hypothesis pending validation):** The .141 hub binary lives on /mnt/c (WSL 9p DrvFs mount, msize=65536). Tokio's TCP/TLS response path includes compiled code in the binary's text segment that the kernel demand-pages from 9p. When the response payload is large (>140KB), the code path executed touches enough of the text segment that 9p page-fault latency cascades into the response stream, presenting as a flaky 10-15s wedge. .121 and .122 don't exhibit this because their binaries run from native ext4 (no on-demand 9p page-faults).

**Why structurally allowed:** TermLink ships fleet-deploy-binary.sh that swaps the binary in-place wherever it's currently running — it doesn't move it to a native-fs path on WSL hosts. The runbook for WSL host deployment didn't exist (operator placed binary on /mnt/c for convenience since the project root is also on /mnt/c via Cargo). T-1991 first observed the wedge but attributed it solely to a hub-binary version regression (0.11.473) because .121/.122 had the SAME problem (cured by T-2013); T-2013 cured the LXC-side worker starvation but exposed that .141's wedge has a SECOND root cause.

**Prevention:** Runbook for WSL hub deployment + lint in fleet-deploy-binary.sh that warns when target is /mnt/* path on WSL hosts + cross-reference learning. The actual fix (move binary to ~/bin) is operator-side.

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

### 2026-06-06T12:20:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2017-t-2013-follow-up-141-channelinfo-wedges-.md
- **Context:** Initial task creation

### 2026-06-06T12:43:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
