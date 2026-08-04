---
id: T-2522
name: "Retention Days mode prunes on client-supplied timestamp (message loss + unbounded-growth vectors)"
description: >
  Retention::Days sweep deletes records WHERE records.ts_unix_ms < now - d*86400000 (bus/meta.rs:319). That column is populated from env.ts_unix_ms (bus/lib.rs:192), which channel.post takes from client params.ts (server_now only as fallback, channel.rs:617). So Days is the ONLY retention mode keyed off a client-supplied, non-monotonic value. Consequence (a) over-reclaim: a message posted now but carrying an old client ts is deleted on the next sweep before any consumer reads it — durability violation / message loss. (b) never-fires: a message with a FUTURE client ts is never < cutoff, survives forever, pins the topic (T-1991-class unbounded growth from one poison record). NEEDS HUMAN DECISION: does Days(N) mean keep-N-days-by-content-time (current) or by-hub-receive-time? Existing tests (bus/lib.rs:1306/1321/1356) encode content-time; agent-presence stale-eviction leans on it. See RCA for the durability argument for receive-time + the fix.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T10:12:58Z
last_update: 2026-08-04T10:30:10Z
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

# T-2522: Retention Days mode prunes on client-supplied timestamp (message loss + unbounded-growth vectors)

## Context

Campaign firing #41 (T-2468 subtract-and-deepen review, retention/sweep lens). Verified in
code, filed as turnkey because closing it needs a **human semantics decision** (below) that an
agent must not make unilaterally.

`Retention::Days(N)` sweeps records `WHERE ts_unix_ms < now - N*86_400_000`
(`bus/meta.rs:319`, cutoff computed at `bus/lib.rs:725`). The `records.ts_unix_ms` column is
written from `env.ts_unix_ms` (`bus/lib.rs:192`, `Bus::post`), and `channel.post` takes that
from the caller's `params.ts`, using `server_now` only as a fallback (`channel.rs:617-625`). So
`Days` is the **only** retention mode whose discriminator is a client-supplied, non-monotonic
value — Messages/Latest/LatestPerCvKey all key off hub-controlled monotonic offsets.

Blast radius confirmed contained: `records.ts_unix_ms` is read by exactly one thing — the Days
sweep DELETE (verified: no other reader across the bus crate uses the records-*index* ts; the
`env.ts_unix_ms` uses at lib.rs:552/572/652/672 are the decoded-envelope ts for presence dedup,
a separate value, and presence topics use `Messages`/`LatestPerCvKey`, not `Days`).

**The durability argument (for receive-time):** the charter promises *durable messages*. A
message posted now must be retrievable for N days from now, so a consumer that comes online
within N days can read it. Content-time retention lets a client-supplied `ts` silently delete a
durable message before any consumer sees it — which defeats the guarantee. This is the strong
reason to prefer hub-receive-time.

## Acceptance Criteria

### Agent
- [ ] (After the Decision below is made) `records.ts_unix_ms` — the column the Days sweep keys off — is populated from a hub-attested receive time, decoupled from `env.ts_unix_ms` (which stays the client/display ts in the envelope blob). A future-dated `ts` can no longer make a record immortal; a backfilled old `ts` no longer deletes an unread just-received message.
- [ ] Existing Days-sweep tests that encode content-time semantics (`bus/lib.rs:1306`, `1321`, `1356`) are updated to the chosen semantics, and the agent-presence stale-eviction path is re-verified unaffected (presence heartbeat ts ≈ receive ts, so no practical change).
- [ ] Regression test `sweep_days_uses_hub_receive_time_not_client_ts`: post one envelope with `ts = server_now - 10*day` and one fresh, both actually posted "now", to a `Days(7)` topic; sweep; assert BOTH survive. Also post one with `ts = now + 30*day`; assert it is pruned once wall-clock passes the window. Proven load-bearing.
- [ ] `cargo test -p termlink-bus --lib` passes; `cargo build --release -p termlink-hub` succeeds.

### Human
- [ ] [REVIEW] **DECIDE: does `Days(N)` retention mean "keep N days by hub-receive-time" or "by client content-time"?** This is the gating decision; the Agent ACs above assume receive-time (the durability-consistent default).
      **Context:** Current behavior is content-time (prunes on client `params.ts`). Two defects follow: (a) a backfilled/skewed old `ts` deletes an unread just-received message (durability violation); (b) a future `ts` is never `< cutoff` → immortal record → unbounded topic growth (T-1991 class). The charter's durability guarantee argues for receive-time (see Context). Counter-consideration: a topic where clients intentionally post historical events and want them pruned by *event* date would want content-time — is any such topic real in this fleet? (agent-presence eviction is unaffected either way: heartbeat ts ≈ receive ts.)
      **Steps:** 1. Read the Context + RCA. 2. Confirm no production topic relies on content-time Days pruning. 3. Choose: **receive-time** (recommended — build the Agent ACs as written), **content-time-but-reject-future-ts** (narrower: keeps content semantics, only kills the immortal-record/growth vector), or **keep-as-is** (accept both risks). 4. Record the choice in the Decision section.
      **Expected:** One of the three options recorded, unblocking the build.
      **If not:** Leave as-is; the daily topic-growth canary (T-2252) will still surface the unbounded-growth symptom if a future-ts poison record lands on a watched topic.

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
cargo test -p termlink-bus --lib sweep_days_uses_hub_receive_time_not_client_ts > /tmp/.t2522-test.out 2>&1 && grep -q "test result: ok" /tmp/.t2522-test.out

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

**Symptom:** On a `Days(N)` topic, (a) a message can be deleted on the next sweep even though a
consumer never read it (backfilled/skewed/old client `ts`), and (b) a message with a future
client `ts` survives forever, pinning the topic against its retention bound (unbounded growth).

**Root cause:** The Days sweep cutoff is compared against `records.ts_unix_ms`, which is written
from `env.ts_unix_ms` = the caller's `params.ts`. Days is the only retention mode keyed off a
client-controlled, non-monotonic value instead of a hub-controlled monotonic offset.

**Why structurally allowed:** The `ts` field serves two conflated purposes — *display/content
timestamp* (legitimately client-supplied) and *retention discriminator* (must be hub-attested).
One column does both. The tests encode the content-time behavior, so the sweep "passed" against
its own assumptions; nothing paired a client-supplied-old-ts post with a Days sweep to expose the
durability violation, and nothing posted a future ts to expose the immortality.

**Prevention:** Decouple the retention discriminator from the display ts (store a hub-receive
time in the records index), plus a regression test that posts old-ts + future-ts records and
asserts receive-time semantics. The daily topic-growth canary (T-2252) is the backstop for the
growth symptom but does not prevent the root cause. **Blocked on the Human decision above.**

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

### 2026-08-04T10:12:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2522-retention-days-mode-prunes-on-client-sup.md
- **Context:** Initial task creation
