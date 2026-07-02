---
id: T-2296
name: "V6: direct transport-first + hub fallback + per-conversation journaling"
description: >
  ARC APEX (highest directive score). Dependency-gated on V1(identity auths socket)+V2(discovery resolves host:port)+V3(notify wakes recipient) — promote to 'now' the moment those land; 'later' is sequencing, NOT backlog. remote_call/remote_exec already direct P2P (TCP+TLS+HMAC, remote.rs:719); reachability spike GO (flat LAN 192.168.10.0/24, no NAT, 3/4 hubs directly reachable). Build: try-direct/fall-back-to-hub orchestration; 3-level confirm ladder (TCP-ack IGNORED as delivery / sidecar-journaled = delivered / read-receipt = consumed) — direct path confirms via sidecar journaled-receipt, hub receipts-frontier (T-2286) is FALLBACK-path only; durable messages move OFF the hub firehose into per-conversation journals (fixes 70.5%-heartbeat obfuscation, T-2250 Tier-0 pattern). ACs: 1:1 msg goes direct when peer reachable, falls back to hub when not; direct delivery confirmed via sidecar journaled-receipt (no hub frontier on direct path); durable msgs do NOT land in hub firehose; per-conversation journal is mineable; cross-host auth uniform (no T-2024 dependency for cross-host path).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:reliable-comms, arc-apex]
components: []
related_tasks: [T-2291, T-2292, T-2293, T-2294, T-2295]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-27T17:06:56Z
last_update: 2026-07-02T07:38:23Z
date_finished: 2026-07-02T07:38:23Z
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

# T-2296: V6: direct transport-first + hub fallback + per-conversation journaling

## Context

Arc-003 (reliable-comms) APEX — highest directive score; the destination of the arc. Horizon `later` is sequencing ONLY (dependency-gated on T-2292 identity / T-2293 discovery / T-2294-5 notify+confirm) — promote to `now` the moment those land; it is NOT backlog. `remote_call`/`remote_exec` are already direct P2P (TCP+TLS+HMAC, `crates/termlink-cli/src/commands/remote.rs:719`); the T-2291 V6 reachability spike returned GO (flat LAN 192.168.10.0/24, no NAT, 3/4 hubs directly reachable). Design trail: `docs/reports/T-2291-cross-agent-comms-inception.md` + `T-2291-V6-spike.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A 1:1 message goes DIRECT host-to-host when the peer is reachable, and falls back to the hub when it is not (try-direct/fall-back orchestration) — **S4 (T-2301)**: `agent-send.sh` default `auto`; reachable→DIRECT (post to peer hub), unreachable→loud local-hub store-and-forward FALLBACK. `test-agent-send-orchestration.sh` 5/5.
- [x] Direct delivery is confirmed via a sidecar journaled-receipt (3-level ladder: TCP-ack ignored as delivery / sidecar-journaled = delivered / read-receipt = consumed); no hub receipts-frontier on the direct path — **S3 (T-2300)**: `notify-sidecar.sh --auto-confirm` journals + posts a mechanism-A `stage=delivered` receipt; agent-send polls mechanism A (never the receipts-frontier on the direct path). `test-sidecar-auto-confirm.sh` 5/5.
- [x] Durable messages do NOT land in the hub firehose — they go to per-conversation journals (fixes the 70.5%-heartbeat obfuscation; T-2250 Tier-0 pattern) — **S5 (T-2302)**: `journal-reaper.sh` trims journaled `dm:` turns off the firehose (journal authoritative). Option-(b) land-then-reap; **live reaper activation is operator-gated** (same precedent as arc-002 R2 sweep) — the mechanism is shipped + tested (`test-journal-reaper.sh` 5/5), scheduling is the operator's enable step (recipe documented, deliberately not auto-installed to avoid trimming production data unprompted).
- [x] The per-conversation journal is mineable (queryable history per conversation) — **S1 (T-2298)**: `agent-journal.sh <topic|cid>` queries `~/.termlink/journals/journal.sqlite` (not the firehose). `test-journal-mirror.sh` 8/8; 1939 real dm rows journaled live.
- [x] Cross-host auth is uniform — the direct path needs no T-2024 dependency — the direct path posts `channel.post` against the peer's OWN hub with the same HMAC/TLS auth model as any hub post (no new auth primitive). Proven by S4 orchestration delivering direct with no T-2024 dependency.

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

# V6 apex capstone: the integrated slice suites (S1–S5) must all pass.
bash scripts/test-journal-mirror.sh
bash scripts/test-agent-send.sh
bash scripts/test-agent-send-transport.sh
bash scripts/test-agent-send-orchestration.sh
bash scripts/test-journal-reaper.sh

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

### 2026-06-28 — implementation design produced (started; not yet built)
- **What changed:** With V1/V2/V3a/V3b all shipped (V6's dependency gate is now
  open), produced a sliced implementation design:
  `docs/plans/T-2296-v6-direct-transport-first-design.md`. KEY FINDING: V6 is NOT
  greenfield — direct host-to-host transport already ships (a "direct" 1:1 message
  is just `channel.post` issued against the **peer's own hub** :9100 instead of the
  local one; `connect_remote_hub` remote.rs:719, TCP+TLS+HMAC all exist). The
  transport-select decision belongs in `agent-send.sh`, which already threads
  `--hub <peer_hub>` through every leg (lines 216-249).
- **Plan impact:** V6 sliced into 5 dependency-ordered sub-slices (one session can't
  do L→XL): **S1** per-conversation journal (read-side SQLite mirror, S→M, smallest
  safe first step, no peer needed) → **S2** reachability-probe + `--transport
  auto|direct|hub` seam (S; reuses `remote ping`) → **S3** direct confirm via sidecar
  journaled receipt (M) → **S4** try-direct/fall-back-to-hub orchestration (M,
  default) → **S5** firehose suppression for `dm:` (M→L, the "off the firehose"
  move; client-side journal-authoritative recommended over hub-side hot-path change).
  The 3-level ladder (delivered/read/acted) maps onto the 3 existing ack signals via
  ONE new `stage=` metadata key on the mechanism-A envelope — no new namespace
  (honors V3b's mechanism-A decision). Mechanism B stays fallback-only.
- **Triggered:** next session implements S1 first (committable, peer-free). Open
  questions for human in the design doc §Risks: (1) S2 on T-2293 self-report addr
  now vs block on T-2297; (2) S5 end-state hub-side vs client-side; (3) sidecar
  auto-ack "delivered" ≠ "cognitively present" — acceptable?

### 2026-07-02 — capstone: all 5 slices S1–S5 shipped; apex ACs met at mechanism level
- **What changed:** The full V6 slice sequence is built and closed — S1 (T-2298
  journal mirror/query), S2 (T-2299 transport-select seam), S3 (T-2300 sidecar
  journaled-receipt), S4 (T-2301 try-direct/fall-back orchestration), S5 (T-2302
  journal-authoritative reaper + firehose suppression). Each apex AC now maps to a
  shipped, tested slice (AC1←S4, AC2←S3, AC3←S5, AC4←S1, AC5←design premise proven by
  S4). All three design-doc §Risks open questions were resolved during build: (1) S2
  shipped on the T-2293 self-report addr — T-2297 hub-attested addr is a hardening
  follow-up, not a blocker; (2) S5 chose the CLIENT-SIDE journal-authoritative
  end-state (option b) over the hub-side hot-path change — smaller blast radius,
  reversible; (3) sidecar auto-ack is "delivered" (L2), distinct from "read" (L3) —
  the delivered-vs-read split the ladder encodes via the `stage=` metadata key.
- **Plan impact:** The design's "mechanism B fallback-only" gave way to the S3/S4
  finding that BOTH transports confirm via mechanism A (the journaled `stage=delivered`
  receipt); mechanism B (`--await-ack`) would double-post the turn, so it's not used on
  either path. AC3's "durables do NOT land in the firehose" is realized by option-(b)
  land-then-reap, so it holds at steady state only while the reaper runs — live reaper
  activation is operator-gated (arc-002 R2 sweep precedent), mechanism shipped + tested.
- **Triggered:** T-2297 (V2b hub-stamped observed source addr) remains the one open
  arc-tagged task — a Rust hot-path hardening (prefer hub-attested addr over
  self-report), consumed-by-V6 but not blocking (V6 works on self-report today).

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

### 2026-06-27T17:06:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2296-v6-direct-transport-first--hub-fallback-.md
- **Context:** Initial task creation

### 2026-06-28T09:42:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-07-02T07:38:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
