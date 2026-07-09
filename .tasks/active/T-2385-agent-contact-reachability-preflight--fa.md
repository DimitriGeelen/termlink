---
id: T-2385
name: "agent contact reachability preflight + fail-fast + structured delivery result — verify LIVE + agent-backed + waker + hub-read-health, return per-link status not silent offset-N (comms loud-contract centerpiece, T-2380 C2/C4/E4)"
description: >
  Before/around send, agent contact checks each delivery link and fails loud: (1) recipient is a LIVE agent not a bare --shell (F1), (2) recipient has a running push-waker or warn they wont be woken (E4), (3) for --ack-required probe target-hub read-health and fail fast instead of burning the timeout (E2). Return {delivered, recipient_live, waker_running, hub_targeted, hub_read_healthy, acked} + one-line diagnosis on any broken link. May slice.

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
created: 2026-07-09T09:29:09Z
last_update: 2026-07-09T11:52:29Z
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

# T-2385: agent contact reachability preflight + fail-fast + structured delivery result — verify LIVE + agent-backed + waker + hub-read-health, return per-link status not silent offset-N (comms loud-contract centerpiece, T-2380 C2/C4/E4)

## Context

T-2380 GO, the loud-delivery-contract **centerpiece** — attacks silent breakpoints
#1 (recipient not agent-backed, F1) and #2 (recipient live but no push-waker →
never woken, E4/PL-237). Depends on T-2384 (done: `agent contact` now addresses the
per-agent presence fp). The Explore map (2026-07-09) confirmed the seams:

- The dm send delegates `agent contact` → `cmd_channel_dm` → `cmd_channel_post`,
  which prints-and-drops the offset (returns `Result<()>`). A fully-unified
  delivery envelope with the real offset needs offset-plumbing up that chain.
- Recipient classification lives on `PresenceMatch` (`fleet_presence.rs:44-72`):
  `status` (Live/Stale/Offline), `pty_session` (**the waker-running signal** —
  be-reachable's PTY-bound heartbeat sets `metadata.pty_session`, T-1834; a bare
  `--shell` register or a waker without a bound PTY lacks it), `role`,
  `listen_topics`.
- `--require-online` (exit 9) today checks chat-arc *posting* activity in a window
  — a weak proxy that MISSES "live but no waker" entirely. No hub-read-health probe
  and no structured delivery-result type exist (both net-new).

**This task ships Slice 1** — the non-breaking, highest-value core:
a reachability preflight computed from the authoritative agent-presence heartbeat
that surfaces `waker_running` and fails loud on demand. **Deferred to later slices:**
hub-read-health fail-fast for `--ack-required` (#5, E2/PL-200) and the fully-unified
single delivery envelope carrying the real posted offset (needs the offset-plumbing
refactor). Reply-on-sender-hub is T-2386; the waker-liveness canary is T-2387.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] New pure fn `classify_reachability(presence: Option<&PresenceMatch>) -> ReachabilityReport { recipient_live, recipient_agent_backed, waker_running, presence_status, diagnosis }` in `agent.rs`, unit-tested for all four states: LIVE + `pty_session` set → all-green, no diagnosis; LIVE + no `pty_session` → `waker_running=false` + diagnosis naming the no-waker breakpoint ("recipient is live but has no push-waker — message will queue until they /check-arc"); STALE/OFFLINE → `recipient_live=false` + loud diagnosis; presence absent → `recipient_live=false` + "no agent-presence heartbeat" diagnosis. (Test `classify_reachability_all_states` → 1 passed.)
- [x] `agent contact` runs the reachability preflight BEFORE the post, keyed on the same agent-presence heartbeat T-2384 addresses on (resolve by target name when present, else by resolved fp via `fetch_recipient_presence`). On `--json` success the output includes a `reachability` object; human mode prints a loud `WARNING:` when `waker_running=false` OR `recipient_live=false`. Non-breaking — smoke on .107: LIVE-no-pty peer → `WARNING: recipient is LIVE but has no push-waker … will queue until they run /check-arc` printed to stderr, send still proceeded.
- [x] New opt-in `--require-reachable` flag hard-fails BEFORE the post when `recipient_live=false`: human-mode exit **11**, `--json` non-zero exit with `exit_code:11` + `reachability` in the body (mirrors the exit-9/10 `json_error_exit` convention). Smoke: `--require-reachable` to `--target-fp deadbeef…` (absent) → human exit=11, json body `exit_code:11`. Absent flag → unchanged send.
- [x] `--dry-run` surfaces the `reachability` block (smoke: dry-run of a LIVE-no-pty peer printed `recipient_live:true, waker_running:false, presence_status:LIVE` + no-waker diagnosis); `--target-fp`-only degrades gracefully (fp-keyed scan; presence-absent → `presence_status:ABSENT` loud diagnosis, no panic).
- [x] `cargo build --release -p termlink` succeeds (9m44s, exit 0); `classify_reachability` unit test passes (debug + release harness). `cargo check -p termlink` clean (no warnings).

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking. -->
- [ ] [REVIEW] The loud WARNING wording is clear and actionable in a real send
  **Steps:**
  1. From a host with a peer that is registered but NOT `/be-reachable` (no waker), run:
     `cd /opt/termlink && ./target/release/termlink agent contact <peer> --message "test" --json`
  2. Read the `reachability` block and the printed `WARNING:` line.
  **Expected:** The warning names the no-waker breakpoint and tells you the message will queue until the peer runs `/check-arc` — not a silent success.
  **If not:** Note the confusing wording; the diagnosis strings live in `classify_reachability`.

## Recommendation

**Recommendation:** GO (Slice 1 — the single Human AC is a wording-taste review, not a blocker)

**Rationale:** Slice 1 of the loud-delivery-contract centerpiece is code-complete,
non-breaking, and verified end-to-end on the live .107 hub. It converts the #1/#2
silent breakpoints ("recipient dead" / "recipient live but no push-waker") into a
visible per-link `reachability` block (+ loud human `WARNING`, + opt-in
`--require-reachable` hard-fail). Default behaviour is unchanged for existing
callers — the send still proceeds, only now annotated — so there is no regression
risk to merge. The only open item is subjective wording of the WARNING string,
which is exactly a `[REVIEW]` Human AC.

**Evidence:**
- Pure classifier `classify_reachability` + 4-state unit test
  (`crates/termlink-cli/src/commands/agent.rs`), P-011 gate PASS
  (`cargo test --release -p termlink --bin termlink classify_reachability` → ok).
- Live smoke on .107 (release binary): dry-run of a LIVE-no-PTY peer →
  `recipient_live:true, waker_running:false, presence_status:LIVE` + no-waker
  diagnosis; `--require-reachable` to an absent fp → human exit **11**, `--json`
  body `exit_code:11`; human-mode send to a LIVE-no-waker peer printed the loud
  `WARNING:` to stderr and still delivered.
- `cargo check -p termlink` clean; release build exit 0 (9m44s).
- Commit `c96e0b7c`. Non-breaking: no change to existing `{delivered}`/`{ack}`
  envelopes (reachability is an additive NDJSON line).

**Deferred (tracked in ## Evolution / ## Context):** hub-read-health fail-fast for
`--ack-required` (#5, E2/PL-200), the offset-plumbing unified delivery envelope, and
MCP `termlink_agent_contact` parity — later slices, out of Slice 1 scope.

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

# T-2385 Slice 1: classify_reachability unit tests live in the bin unit-test
# binary (`-p termlink` alone runs only the integration binaries). Capture-then-
# grep per L-387 (single pipe, no SIGPIPE).
out=$(cargo test --release -p termlink --bin termlink classify_reachability 2>&1); echo "$out" | grep -qE "test result: ok"

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

**Symptom:** `agent contact` (and its skill wrappers) returned a write success
(`offset N`) even when the recipient was gone, was a bare `--shell`, or was LIVE but
had no running push-waker — so a send "succeeded" while the message woke nobody. The
operator got no signal distinguishing "delivered + will be read" from "durably
written to a rail nobody is listening on." This is the E4/PL-237 field failure: 8
live sessions on .107, zero wakers, every DM a silent no-op.

**Root cause:** the send path optimised for durability (the hub append always
succeeds) but never verified — or surfaced — the *round trip*. The authoritative
signal for "will this recipient actually be woken" already existed on the
`agent-presence` heartbeat (`status` for liveness, `pty_session` for a bound
push-waker PTY), but `agent contact` never read it before posting. The only existing
preflight, `--require-online`, checked chat-arc *posting* activity — a weak proxy
that cannot see the "live but no waker" case at all.

**Why structurally allowed:** "delivery" was defined as "the append returned an
offset," never as "a live, awake recipient is listening on this topic." No layer
composed the presence signal with the send, so the gap was invisible: the happy
path and the dead-rail path produced identical output. Single-waker hosts and
interactive testing masked it (a human ran `/check-arc` manually), so it only bit in
the autonomous shared-host fleet — and silently.

**Prevention:** (1) the reachability preflight now reads presence before every send
and annotates the result — a broken link is loud (human `WARNING` / `--json`
`reachability` block) instead of silent; (2) `--require-reachable` upgrades it to a
hard fail (exit 11) for callers that want fail-fast; (3) `classify_reachability` is a
pure, unit-tested classifier so the four-state contract can't silently regress; (4)
the standing guard against recurrence — a waker-liveness canary so "shipped ≠
capability-live" surfaces on its own — is filed as T-2387.

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

### 2026-07-09 — Slice 1 boundary (reachability preflight shipped)
- **What changed:** The Explore map confirmed `pty_session` on the agent-presence
  heartbeat is the load-bearing `waker_running` signal — this made the #2 breakpoint
  ("live but no push-waker") cleanly detectable WITHOUT any hub-side change. It also
  revealed the send path drops the posted offset (`cmd_channel_post` returns
  `Result<()>`), so a *fully-unified* delivery envelope carrying the real offset
  needs an offset-plumbing refactor up `cmd_channel_post → cmd_channel_dm →
  cmd_agent_contact`.
- **Plan impact:** Split the original "verify LIVE + agent-backed + waker +
  hub-read-health + unified envelope" scope. Slice 1 (this task) ships the
  reachability preflight (links #1/#2) as a NON-breaking annotation + opt-in
  `--require-reachable` fail-fast, emitting a `reachability` NDJSON line rather than
  restructuring the existing `{delivered}`/`{ack}` envelopes. Deferred: hub-read-health
  fail-fast for `--ack-required` (#5) and the offset-plumbing unified envelope.
- **Triggered:** Deferred slices remain under T-2385 scope (not yet re-filed as
  separate tasks); MCP `termlink_agent_contact` parity for the preflight is a follow-up
  (this slice is CLI-first). Chose NDJSON annotation over envelope-restructure to keep
  the change non-breaking for existing `--json` consumers.

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

### 2026-07-09T09:29:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2385-agent-contact-reachability-preflight--fa.md
- **Context:** Initial task creation

### 2026-07-09T11:28:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
