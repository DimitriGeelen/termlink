---
id: T-2352
name: "agent-send.sh --to-session mints wrong-self-fp dm topic"
description: >
  Field-discovered in T-2350: explicit-routing send (--to-session tl-dzbcxxka --peer-fp 9219671e...) resolved self-fp as 06cd308242ef95bc (identity-file fp) instead of the registered/canonical d1993c2c..., minting NEW topic dm:06cd...:9219671e... instead of posting to the existing canonical thread. PL-236 class (identity show vs registered fp divergence). Fix: resolve self-fp via the T-1857 chain (be-reachable.state / registered session fp), or at minimum prefer an EXISTING dm topic containing peer-fp before minting a new one.

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
created: 2026-07-04T12:00:35Z
last_update: 2026-07-04T13:02:28Z
date_finished: 2026-07-04T13:02:28Z
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

# T-2352: agent-send.sh --to-session mints wrong-self-fp dm topic

## Context

Field failure (T-2350): `agent-send.sh --to-session tl-dzbcxxka --peer-fp 9219671e...` resolved self-fp via `channel info agent-presence | jq '.senders[0].sender_id'` — literally the FIRST sender on the topic, which on a shared/multi-agent hub is any co-resident or remote agent's fp (got 06cd308242ef95bc), minting a NEW topic `dm:06cd...:9219...` instead of posting to the existing canonical thread `dm:9219671e28054458:d1993c2c3ec44c94`. PL-236 class. Fix: resolve self-fp via the signing-path precedence chain, and prefer an EXISTING dm topic containing peer-fp before minting.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Self-fp resolution in the `--peer-fp` path follows the chain: `$TERMLINK_SELF_FP` env (test seam/override) → `be-reachable.state` `.self_fp` (T-2324) → `termlink agent identity --resolve` (signing-path precedence) → legacy senders-scan (last resort only) — replacing the senders[0]-first heuristic
- [x] Before minting a new dm topic, EXISTING `dm:*` threads with peer-fp as exactly one component are preferred, disambiguated by **peer participation**: exactly one thread the peer has posted in → use it (loud stderr NOTE if ≠ canonical mint); multiple peer-posted threads none matching the mint → refuse (exit 2) with candidate list + `--topic` hint; zero peer-posted → canonical mint if it exists, else single candidate reused with NOTE, else refuse ambiguous
- [x] Peer self-dm topics (`dm:<peer>:<peer>`) are excluded from candidate matching (a DM from us to the peer is never their self-thread)
- [x] `--dry-run` works with explicit routing (`--to-session` + `--peer-fp`/`--topic`), printing the RESOLVED line without posting/injecting — the regression seam for this fix
- [x] Regression proven: with `TERMLINK_SELF_FP=06cd308242ef95bc` (the field wrong-fp) and `--peer-fp 9219671e28054458`, the resolved topic is the existing canonical `dm:9219671e28054458:d1993c2c3ec44c94`, NOT a fresh `dm:06cd...` mint

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
# env-override seam resolves the canonical topic (no hub dependency on fp choice)
out=$(TERMLINK_SELF_FP=d1993c2c3ec44c94 bash scripts/agent-send.sh --to-session tl-seam-test --peer-fp 9219671e28054458 --message seam --dry-run 2>&1); echo "$out" | grep -q "topic=dm:9219671e28054458:d1993c2c3ec44c94"
# field regression: wrong self-fp is redirected to the EXISTING canonical thread instead of minting dm:06cd...
out=$(TERMLINK_SELF_FP=06cd308242ef95bc bash scripts/agent-send.sh --to-session tl-seam-test --peer-fp 9219671e28054458 --message seam --dry-run 2>&1); echo "$out" | grep -q "topic=dm:9219671e28054458:d1993c2c3ec44c94"

## RCA

**Symptom:** Explicit-routing send (`--to-session` + `--peer-fp`) minted a brand-new dm topic `dm:06cd308242ef95bc:9219671e...` instead of posting to the existing canonical thread `dm:9219671e28054458:d1993c2c3ec44c94` — the peer never saw the turn on the thread it watches.

**Root cause:** Self-fp was resolved as `channel info agent-presence --json | jq '.senders[0].sender_id'` — the FIRST sender on a shared topic, which on a multi-agent hub is arbitrarily some other agent's fingerprint. Nothing tied the resolved fp to what this sender actually signs with.

**Why structurally allowed:** The dm-topic mint is client-side and unvalidated — any `dm:a:b` name is accepted by `--ensure-topic`, so a wrong-fp mint succeeds silently; no check compares the minted topic against existing threads with the peer.

**Prevention:** (1) resolution chain anchored to the signing path (`TERMLINK_SELF_FP` → be-reachable.state → `agent identity --resolve`), senders-scan demoted to last resort; (2) existing-topic preference guard — a peer-fp-matching existing thread wins over a fresh mint, ambiguity refuses loud; (3) `--dry-run` seam on the explicit path makes the resolution testable without posting.

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

### 2026-07-04 — Existing-thread disambiguation discriminator
- **Chose:** peer participation (`channel info` senders: does peer_fp have >0 posts in the candidate?) as the primary discriminator among existing dm threads; post-count and canonical-mint-existence only as fallbacks when no thread has peer posts.
- **Why:** The field incident left BOTH the canonical thread (110 posts, peer active) and the wrong-mint `dm:06cd...:9219...` (1 post, only us) on the local hub. "Canonical mint exists → use it" would have PERPETUATED the bug whenever the resolved self-fp is wrong (the wrong mint exists precisely because of the earlier failure). "A thread the peer has spoken in" is the semantically correct definition of the live conversation and self-heals around stale wrong-mint cruft.
- **Rejected:** highest-post-count heuristic (correlates but is not the semantic property; ties are unresolvable); refuse-on-any-ambiguity (would make every send to ring20 fail until the 1-post cruft topic is manually deleted).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-04T12:00:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2352-agent-sendsh---to-session-mints-wrong-se.md
- **Context:** Initial task creation

### 2026-07-04T12:58:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-043c2b16
- **Timestamp:** 2026-07-04T13:02:29Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-04T13:02:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
