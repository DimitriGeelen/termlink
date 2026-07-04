---
id: T-2353
name: "agent-send.sh doorbell inject ignores peer hub"
description: >
  Field-discovered in T-2350: with --to-session targeting a session on a REMOTE hub (tl-dzbcxxka on 192.168.10.122:9100), the doorbell inject ran against the LOCAL hub ('session missing?' WARN x2) — the mail posted but the doorbell never rang the peer. Workaround used: direct 'channel post --hub <peer-hub>' + 'termlink remote inject <hub> <session>'. Fix: agent-send.sh explicit-routing path must carry/accept the peer hub (--hub flag or resolve from presence) and use remote inject when the session is not local.

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
created: 2026-07-04T12:00:55Z
last_update: 2026-07-04T13:02:53Z
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

# T-2353: agent-send.sh doorbell inject ignores peer hub

## Context

Field failure (T-2350): with `--to-session` targeting a session on a REMOTE hub (tl-dzbcxxka on 192.168.10.122:9100), the doorbell inject ran against the LOCAL hub (`session missing?` WARN ×2) — the mail posted locally and the doorbell never rang the peer. The remote plumbing (post `--hub`, `remote inject`, hub-scoped receipt polling) already exists but is only wired to the `--to` auto-discover path; explicit routing never sets `peer_hub`. Fix: accept `--hub <addr>` on the explicit-routing path.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `agent-send.sh` accepts `--hub <addr>` with explicit routing (`--to-session` + `--topic`/`--peer-fp`), setting `peer_hub` so the existing remote plumbing applies: mail posts to the peer hub, doorbell rings via `termlink remote inject`, receipt/reply polling targets the peer hub
- [x] `--hub` is mutex with `--to` (auto-discover resolves its own hub) — combining them exits 2 with a clear message
- [x] The dm-topic existence scan (T-2352) runs against the destination hub when `--hub` is set (dm topics are per-hub state, G-060) — and every scan call is time-bounded (`TERMLINK_SCAN_TIMEOUT`, default 8s) with loud degradation to the canonical mint, so a wedged remote hub can never hang the send (T-2354 discovered here)
- [x] `--dry-run` with explicit routing + `--hub` prints `hub=<addr> routing=remote` in the RESOLVED line (regression seam)
- [x] Usage text documents `--hub` under explicit routing

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

bash -n scripts/agent-send.sh
# explicit routing + --hub resolves remote routing in the RESOLVED line
out=$(TERMLINK_SELF_FP=d1993c2c3ec44c94 bash scripts/agent-send.sh --to-session tl-seam-test --peer-fp 9219671e28054458 --hub 192.168.10.122:9100 --message seam --dry-run 2>&1); echo "$out" | grep -q "hub=192.168.10.122:9100 routing=remote"
# --hub is mutex with --to
out=$(bash scripts/agent-send.sh --to some-agent --hub 1.2.3.4:9100 --message seam --dry-run 2>&1); echo "$out" | grep -qi "mutex"

## RCA

**Symptom:** `--to-session tl-dzbcxxka` (a session on hub 192.168.10.122:9100) posted the mail to the LOCAL hub and rang the doorbell via local `inject` — `WARN ... (session missing?)` ×2; the peer was never woken and never saw the turn.

**Root cause:** `peer_hub` is only populated by the `--to` auto-discover path (T-2273 fleet lookup). The explicit-routing path (`--to-session` + `--topic`/`--peer-fp`) has no way to name the peer's hub, so all remote plumbing (post `--hub`, `remote inject`, hub-scoped polling) silently degrades to local.

**Why structurally allowed:** The explicit path predates cross-hub support (pre-T-1834 form) and was never extended when T-2273 added the hub-aware plumbing; inject failure is deliberately non-fatal (best-effort doorbell), so the misroute surfaced only as a WARN while the send "succeeded" against the wrong hub.

**Prevention:** `--hub` flag on the explicit path reuses the exact same plumbing as auto-discover (one `peer_hub` variable, no second code path); `--dry-run` RESOLVED seam prints `hub=... routing=remote` so the routing is assertable in verification; mutex with `--to` prevents conflicting hub sources.

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

### 2026-07-04 — Bounded scan calls after discovering T-2354
- **Chose:** wrap the T-2352 dm-topic scan (`channel list` + per-candidate `channel info`) in `timeout ${TERMLINK_SCAN_TIMEOUT:-8}`; on list failure/timeout emit a loud stderr NOTE and fall back to the canonical mint.
- **Why:** verification against the live .122 hub exposed that `channel info`/`channel unread` over `--hub <tcp>` hang indefinitely (while `channel list` works) — an unbounded scan would have hung every remote send, a worse regression than the wrong-mint bug being fixed. Filed as T-2354 (root-cause fix in the CLI/hub).
- **Rejected:** reverting the destination-hub scan (loses the G-060 correctness — topics are per-hub); waiting for the T-2354 root-cause fix (blocks this shipped fix on an unscoped one).

### 2026-07-04 — `--hub` flag vs presence auto-resolve for explicit routing
- **Chose:** explicit `--hub <addr>` flag, mutex with `--to`.
- **Why:** the explicit-routing form exists precisely for cases where presence discovery is unavailable or wrong (the T-2350 field case: session known, presence rail incomplete); auto-resolving from presence there re-introduces the dependency the operator was routing around. One `peer_hub` variable engages ALL existing remote plumbing — no second code path to maintain.
- **Rejected:** resolving hub from presence when `--to-session` is given (fails exactly when explicit routing is needed most); trying local inject first then falling back to remote probing (slow, and inject failure is non-fatal by design so the misroute would stay silent).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-04T12:00:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2353-agent-sendsh-doorbell-inject-ignores-pee.md
- **Context:** Initial task creation

### 2026-07-04T13:02:53Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
