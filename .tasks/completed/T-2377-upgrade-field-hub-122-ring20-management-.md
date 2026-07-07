---
id: T-2377
name: "Upgrade field hub .122 ring20-management to 0.11.399 via musl fleet-deploy; assess .121 fork-lineage before any action"
description: >
  Upgrade field hub .122 ring20-management to 0.11.399 via musl fleet-deploy; assess .121 fork-lineage before any action

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
created: 2026-07-07T10:10:46Z
last_update: 2026-07-07T10:10:46Z
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

# T-2377: Upgrade field hub .122 ring20-management to 0.11.399 via musl fleet-deploy; assess .121 fork-lineage before any action

## Context

Follow-on to T-2376 (.107 upgrade). `fleet doctor` shows the field: .122
(ring20-management) serves **0.11.377** (same lineage, 22 commits behind our
0.11.399) — a genuine upgrade. .121 (ring20-dashboard) serves **0.11.806** from
its OWN FORK (per CLAUDE.md fleet-version-floors: "numerically newest while
lacking our commits") — pushing our 0.11.399 would be a numeric DOWNGRADE and a
lineage replacement, destroying its fork commits. .141 is unreachable (conn=error).
This task upgrades ONLY .122 via `scripts/fleet-deploy-binary.sh --probe
--swap-restart` (musl-static, base64-over-remote-exec, no SSH). .121 is
explicitly HELD for an operator decision (see Decisions) — not touched.

## Acceptance Criteria

### Agent
- [x] musl-static binary rebuilt to 0.11.400 (restamped from stale 0.11.377; 400 = current HEAD after T-2376 docs commit)
- [x] `scripts/fleet-deploy-binary.sh ring20-management --probe --swap-restart` completes: 730/730 chunks, sha verified, `--probe` OK (0.11.400 executes on target), swap+restart done, hub UP at t=15s
- [x] Post-deploy `fleet doctor` shows .122 (192.168.10.122:9100) serving **0.11.400**, conn=ok
- [x] PL-021 check: .122 auth valid after restart — deploy verify [PASS] connected in 42ms (client authenticated w/ existing profile secret → secret preserved)
- [x] .121 NOT modified — only read-only probed (tofu verify + hub probe); held for operator decision (see Decisions)
- [x] fleet-binary-canary re-checked: healthy — `ring20-management served=0.11.400 >= floor=0.11.324`; `.121 exempt (not firing)`

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

# Target is current HEAD = 0.11.400 (0.11.399 VERSION + the T-2376 docs-only commit 5832d7ae → git bumped commits-since-tag 399→400)
test -x target/x86_64-unknown-linux-musl/release/termlink && target/x86_64-unknown-linux-musl/release/termlink --version 2>&1 | grep -q "0.11.400"

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

### 2026-07-07 — .121 ring20-dashboard: HELD, not upgraded
- **Chose:** Do NOT deploy 0.11.400 to .121; surface a decision to the operator instead.
- **Why:** .121 serves **0.11.806** — genealogically divergent from our mainline (our HEAD is 400 commits past v0.11.0; .121 is 806 past the same tag, so it carries ~400 commits we don't have and cannot be a fast-forward of ours). It is marked **exempt** in `fleet-version-floors.conf` (`ring20-dashboard -`) per the documented rule "never set a floor / deploy to a hub whose binary you don't build". Overwriting it with our binary would be a numeric downgrade AND a lineage replacement (destroys its fork commits). Additionally, its live TLS fingerprint `sha256:1389a831…` differs from the recorded `53de15ec` (memory reference_ring20_dashboard), so the inherited "own fork" characterization is anchored to a stale identity — I cannot even confirm the current lineage from here (no session registered on .121 → git history not RPC-inspectable).
- **Rejected:** (a) blind `fleet-deploy-binary.sh ring20-dashboard` — destructive, would replace an uncharacterized lineage; (b) asserting it's a fork without evidence — the identity anchor drifted, so this needs a shell session on the dashboard project (cross-project, T-559) to answer "what are those 806 commits".
- **Operator options:** SKIP (leave .121 on its own lineage), or file a separate reconcile-then-deploy task if .121 is meant to be on mainline. Requires operator input.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-07T10:10:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2377-upgrade-field-hub-122-ring20-management-.md
- **Context:** Initial task creation

### 2026-07-07T~10:20Z — .122 upgraded, .121 held [agent]
- **Fleet baseline (pre):** fleet doctor → .122=0.11.377, .121=0.11.806, .107/localhost=0.11.399, .141=error.
- **musl build:** `cargo clean -p termlink --release --target x86_64-unknown-linux-musl` + rebuild → `0.11.400` static (HEAD moved 399→400 via T-2376 docs commit 5832d7ae; functionally identical, docs-only).
- **.122 deploy:** `scripts/fleet-deploy-binary.sh ring20-management --probe --swap-restart` → 730/730 chunks 0 failures, sha verified, probe OK (0.11.400 runs on target), G-070 guard confirmed NO systemd unit on .122 (bare-process → detached relaunch correct), swap+restart pid 306862, hub UP t=15s, verify [PASS] 42ms.
- **.122 verify (post):** fresh fleet doctor → .122=0.11.400 conn=ok; auth held (authenticated connect = profile secret preserved across restart, PL-021 OK). fleet-binary-canary healthy (.122 0.11.400 >= floor 0.11.324).
- **.121 read-only probe:** tofu verify OK (pin matches wire), hub probe fingerprint `sha256:1389a831…` (differs from recorded `53de15ec` → identity drift, inherited fork-claim is stale-anchored), version 0.11.806, NO sessions registered → git lineage not RPC-inspectable. HELD for operator decision (see Decisions).
- **Fleet after:** .107=0.11.399, .122=0.11.400 (1 docs-commit skew, noise), .121=0.11.806 (untouched), .141 unreachable.
