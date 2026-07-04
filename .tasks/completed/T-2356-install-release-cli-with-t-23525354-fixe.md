---
id: T-2356
name: "Install release CLI with T-2352/53/54 fixes on .107 (rm-then-cp to /root/.cargo/bin)"
description: >
  Deploy follow-up: the installed /root/.cargo/bin/termlink is 0.11.296 (pre-fix); main now carries T-2352 (self-fp chain + peer-posted thread preference), T-2353 (agent-send --hub), T-2354 (bounded TCP RPC reads). CLI-only change — NO hub restart needed (client.rs/channel.rs are client-side). Path: cargo build --release -p termlink, backup existing binary, rm-then-cp (avoids ETXTBSY), verify termlink --version >= 0.11.313 and the T-2354 seam (TERMLINK_RPC_READ_TIMEOUT_SECS bounded error vs wedged .122 read). Closes the preflight Check 4 stale-binary WARN.

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
created: 2026-07-04T14:02:48Z
last_update: 2026-07-04T14:26:25Z
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

# T-2356: Install release CLI with T-2352/53/54 fixes on .107 (rm-then-cp to /root/.cargo/bin)

## Context

The installed `/root/.cargo/bin/termlink` on .107 is 0.11.296 — it predates T-2352 (self-fp resolution chain + peer-posted thread preference in agent-send.sh's underlying verbs), T-2353 (`--hub` explicit routing), and T-2354 (bounded TCP RPC read deadlines via `TERMLINK_RPC_READ_TIMEOUT_SECS`, default 30s). Until the release binary is swapped, any interactive `termlink channel info/unread --hub <tcp>` from this host can still hang indefinitely against a wedged hub (the exact G-157 class T-2354 fixed), and `/preflight` Check 4 WARNs stale-binary. CLI-only deploy: client.rs/channel.rs changes are client-side — NO hub restart needed, no re-pin, no auth impact. Install pattern: rm-then-cp (avoids ETXTBSY on a running binary's inode).

## Acceptance Criteria

### Agent
- [x] `cargo build --release -p termlink` completes cleanly on main at or after commit 998785dc (T-2354) — built at 30a4b559, exit 0, 11m29s, artifact mtime 16:38 (fresh, PL-209 checked)
- [x] Existing `/root/.cargo/bin/termlink` backed up (e.g. `termlink.0.11.296.bak`) before replacement — `/root/.cargo/bin/termlink.0.11.296.bak` present (33577280 bytes)
- [x] Binary swapped via rm-then-cp (not cp-over, which hits ETXTBSY); `termlink --version` reports >= 0.11.313 — installed reports 0.11.321
- [x] T-2354 seam present in installed binary: `TERMLINK_RPC_READ_TIMEOUT_SECS=5 termlink channel info agent-chat-arc --hub 192.168.10.122:9100` errored bounded at 5256ms with "RPC 'channel.subscribe' response timeout after 5s (hub accepted the connection but never replied — wedged record-walk or overloaded hub)" — vs pre-fix indefinite hang (.122 walk confirmed still wedged, live reinforcement for T-2355)
- [x] `/preflight` Check 4 (CLI version >= project VERSION) no longer WARNs stale-binary — `[PASS] binary termlink 0.11.321 >= project VERSION 0.11.321`; summary: 5 pass, 0 warn, 0 fail

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

termlink --version > /tmp/.t2356-ver.out 2>&1 && grep -q "termlink 0.11.321" /tmp/.t2356-ver.out
test -f /root/.cargo/bin/termlink.0.11.296.bak
bash scripts/substrate-preflight.sh > /tmp/.t2356-preflight.out 2>&1 && grep -q "0 warn, 0 fail" /tmp/.t2356-preflight.out

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

### 2026-07-04T14:02:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2356-install-release-cli-with-t-23525354-fixe.md
- **Context:** Initial task creation

### 2026-07-04T14:26:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-04T14:45:00Z — install-completed [agent]
- **Action:** Built release CLI at 30a4b559 (11m29s, exit 0), backed up 0.11.296 to `/root/.cargo/bin/termlink.0.11.296.bak`, swapped via rm-then-cp (termlink_run — T-559 boundary), chmod 755
- **Evidence:** installed `termlink --version` = 0.11.321; T-2354 seam live-proven (bounded 5.26s error vs .122's still-wedged walk); preflight 5 pass / 0 warn / 0 fail (Check 4 stale-binary WARN cleared)
- **Context:** CLI-only deploy — no hub restart, no re-pin, no auth impact, as scoped
