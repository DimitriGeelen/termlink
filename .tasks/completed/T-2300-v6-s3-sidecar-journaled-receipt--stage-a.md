---
id: T-2300
name: "V6-S3 sidecar journaled-receipt + stage-aware confirm"
description: >
  V6-S3 sidecar journaled-receipt + stage-aware confirm

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:reliable-comms]
components: []
related_tasks: [T-2291, T-2296, T-2298, T-2299]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-01T21:03:51Z
last_update: 2026-07-01T21:14:20Z
date_finished: 2026-07-01T21:14:20Z
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

# T-2300: V6-S3 sidecar journaled-receipt + stage-aware confirm

## Context

arc-003 reliable-comms V6 (apex, T-2296) **slice S3** — the recipient-side
auto-confirm. Design: `docs/plans/T-2296-v6-direct-transport-first-design.md`
§2-S3 + §3 (the 3-level confirm ladder). Builds on S1 (T-2298, journal) and S2
(T-2299, transport seam).

**Deliverable:** the V3a notify sidecar (`scripts/notify-sidecar.sh`) gains an
opt-in `--auto-confirm` mode that, per cycle, for every `dm:<self>:*` topic with
unread mail, (a) mirrors that topic into the S1 journal
(`~/.termlink/journals/journal.sqlite` via `scripts/journal-mirror.sh`) and
(b) auto-posts a **mechanism-A** receipt `--msg-type receipt --metadata
stage=delivered --metadata up_to=<latest_offset>` — the recipient confirms
delivery with NO LLM turn. This is the direct path's **L2-delivered producer**
(design §3 ladder row L2, direct column). `agent-send.sh`'s receipt poll
(agent-send.sh:259-266) is made **stage-aware**: it surfaces the receipt's
`stage` in the DELIVERED line and still treats an un-tagged receipt (pre-S3,
V3b) as delivered — backward compatible.

**Scope boundary (what is NOT in S3):** the sender's try-direct/fall-back
routing branch AND the "doorbell becomes optional on the direct path" change are
**S4** — S3 only adds the recipient's journaling auto-acker + the `stage`
semantic + the sender's stage-aware *recognition*. No routing change. Mechanism
B (hub `channel.receipts` frontier) is untouched — it stays the fallback-only
producer (design §3, AC2 of the apex).

**Ladder key (design §3):** one new `stage` metadata key on the existing
mechanism-A envelope — `stage=delivered` (sidecar, auto, no LLM) < `stage=read`
(agent at yield point, future) < reply turn = `acted` (mechanism C, already
covered by `--await-reply`). No new receipt namespace.

Key code anchors: sidecar arg-parse notify-sidecar.sh:96-109; probe_mail
enumerates dm topics + unread notify-sidecar.sh:145-190; write_cycle
notify-sidecar.sh:192-225; agent-send receipt poll (make stage-aware)
agent-send.sh:259-274; journal mirror `scripts/journal-mirror.sh --topic <T>`
(S1). Test hooks: `TERMLINK_NOTIFY_TEST_UNREAD` short-circuits the real probe
(so the auto-receipt path — which needs a REAL topic+offset — is tested via a
real self-post to a loopback dm: topic, the test-journal-mirror.sh pattern).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `notify-sidecar.sh` accepts `--auto-confirm` (default OFF → V3a behavior byte-for-byte: no receipt posted, no journal write). Invalid combinations rejected with exit 2. `--help` documents the flag and its store-and-forward semantics — flag parsed (`--auto-confirm` case), gated behind `[ "$auto_confirm" -eq 1 ]` in the unread loop; T1 proves default-off is a no-op (receipts=0 journal=0); `--help` block added
- [x] With `--auto-confirm`, each cycle: for every `dm:<self>:*` topic carrying unread mail, the sidecar (a) mirrors the topic into the S1 journal (`journal-mirror.sh`) and (b) posts a mechanism-A receipt `--msg-type receipt --metadata stage=delivered --metadata up_to=<latest_offset>` to that topic — verified live against a loopback self-posted dm: turn — `_auto_confirm_topic` calls `journal-mirror.sh --topic` + `channel post --msg-type receipt --metadata stage=delivered --metadata up_to=<content-watermark>`; T2 journal rows=3, T3 one stage=delivered receipt up_to=2
- [x] Re-post guard: the sidecar records the last-acked offset per topic in a durable local state file under `notify_dir` and posts a NEW receipt only when the latest offset advances — a second `--once` on an unchanged topic posts NO duplicate receipt (no ack spam; survives restart) — guard file `.$agent_id.<topic>.acked`; watermark computed over CONTENT offsets only (meta excluded) so our own receipt does not bump it; T4 second `--once` posts no dup (receipts still 1)
- [x] `agent-send.sh` receipt poll is stage-aware: it surfaces the receipt's `stage` (e.g. `DELIVERED (stage=delivered)`) when present, and an un-tagged receipt (pre-S3/V3b shape) still counts as DELIVERED — backward compatible (A–G tests still pass) — poll now captures the whole receipt (`recv_json`), extracts `.metadata.stage`; DELIVERED line = `DELIVERED${stage:+ (stage=$stage)}`; T5 surfaces `(stage=delivered)`; A–G ALL PASS (un-tagged → plain DELIVERED)
- [x] Tests prove all of the above hub-independently (peer-free, loopback): `scripts/test-sidecar-auto-confirm.sh` (default-off no-op, journal row written, `stage=delivered` receipt posted, offset-guard no-dup, agent-send reads the receipt as DELIVERED with stage surfaced); existing `scripts/test-agent-send.sh` (A–G) and `scripts/test-journal-mirror.sh` still pass — `test-sidecar-auto-confirm.sh` 5/5; `test-agent-send.sh` A–G ALL PASS; `test-journal-mirror.sh` 8/8; `test-agent-send-transport.sh` 7/7; both `bash -n` clean

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
# (S3 commands — the auto-confirm test file is created during the build.)
bash scripts/test-sidecar-auto-confirm.sh
bash scripts/test-agent-send.sh
bash scripts/test-journal-mirror.sh
bash -n scripts/notify-sidecar.sh
bash -n scripts/agent-send.sh

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

### 2026-07-01 — `channel unread` semantics drove three design corrections

- **What changed (unread is frontier-relative, not sender-relative):** the design
  assumed a self-posted turn would show as "unread" and carry a usable
  `latest_offset`. Live probing disproved both: `channel unread --sender <fp>`
  counts *content past `<fp>`'s own latest `m.receipt.up_to`* — a fresh reader with
  no receipt is treated as caught-up (0 unread), and `--json .latest_offset` is
  `null` even when `unread_count>0`. So genuine unread had to be manufactured as
  turn1 → receipt(up_to=0) → turn2, and the ack watermark had to be read from
  `channel subscribe` (max offset), not from `unread`.
- **Plan impact / offset-guard fix:** the watermark must exclude meta types
  (receipts/reactions/…) — taking the max over ALL offsets made the sidecar's own
  posted receipt bump the watermark, so every re-run re-acked (caught live: run1=1,
  run2=2 receipts). Excluding meta types (mirroring `channel unread`'s own rule)
  fixed the guard (run1=1, run2=1). This is the load-bearing detail of the slice.
- **Triggered — new test seam `TERMLINK_NOTIFY_TEST_TOPICS`:** the real enumeration
  matches *every* dm: topic this fp participates in (121 live on this host), so a
  test could not use it without auto-acking production conversations. Added an
  explicit-topic-list test hook (mirroring `TERMLINK_NOTIFY_TEST_UNREAD`) to scope
  the enumeration — isolates the test AND is a reasonable operator affordance.
- **Scope held:** the sender routing branch + doorbell-optional-on-direct stayed in
  S4 as planned; S3 delivered exactly the recipient auto-acker + `stage` semantic +
  sender-side stage recognition. Mechanism B untouched (fallback-only).

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

### 2026-07-01T21:03:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2300-v6-s3-sidecar-journaled-receipt--stage-a.md
- **Context:** Initial task creation

### 2026-07-01T21:14:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
