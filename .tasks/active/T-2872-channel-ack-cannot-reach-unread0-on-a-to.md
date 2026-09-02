---
id: T-2872
name: "channel ack cannot reach unread=0 on a topic the acking identity also posts
  to"
description: >
  unread is computed as latest - receipt_up_to, and 'channel ack' posts the receipt
  as a normal envelope, which advances latest. So each ack manufactures exactly the
  one unread item it was meant to clear. Measured deterministically: three consecutive
  acks on dm:d1993c2c3ec44c94:deadbeefdeadbeef moved latest 7->8->9 and receipt_up_to
  6->7->8 with unread pinned at 1 throughout. Affects every topic the local identity
  posts to, including agent-chat-arc, so the T-2838 receiver-ack-lag guard decays
  back to non-zero after every broadcast.

status: started-work
workflow_type: build
owner: claude-code
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-08-31T19:46:32Z
last_update: 2026-09-02T06:45:13Z
date_finished:
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
bvp_scores_proposed:
  - ts: '2026-08-31T19:47:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2872: channel ack cannot reach unread=0 on a topic the acking identity also posts to

## Context

> **CONFIRMED 2026-09-01 — the defect is real. An intermediate "correction" posted
> on this task and broadcast to the fleet claimed it did not reproduce; that claim
> was wrong and has been retracted on the rail. Both wrong turns are left in the
> record below, because a task that quietly edits away its own bad calls teaches
> nothing.**

**What actually happened.** `channel unread` was measured first and *does* converge,
so the whole surface was declared clean. It is not the surface the filing was about.
Measured side by side, one topic, one moment, three consecutive acks:

```
ack #1   agent inbox -> unread=1     channel unread -> unread_count=0
ack #2   agent inbox -> unread=1     channel unread -> unread_count=0
ack #3   agent inbox -> unread=1     channel unread -> unread_count=0
```

Both are true simultaneously. That is PL-350's shape exactly — *"two verbs answering
'have I read this?' differently is not a difference in scope, it is a
contradiction."* Verifying one verb and generalising to the other is the same
adjacent-assertion error the filing warned about, committed while purporting to
correct the filing for it.

**Located.** `unread_verdict()` (`crates/termlink-cli/src/commands/channel.rs`)
returns `latest - frontier`: a **raw offset subtraction with no `msg_type`
awareness**. `channel ack` appends its receipt at `latest` and sets
`receipt_up_to = latest - 1`, so the difference is pinned at 1 forever. `latest`
arrives from the hub's `latest_offset` (T-2533) or is reconstructed from a count, so
at that layer there is no envelope type to filter on — which is why this cannot be
fixed by adding an exclusion the way `count_unread` has one.

**Why this surface is the one that matters.** `/check-arc` reads `agent inbox`, so
this is the unread number an operator actually sees. `channel unread` — the verb
that is correct — is not on the path they use.

**The class has been fixed three times already and this sibling was never
migrated.** `PL-317` (T-2589) named it: *"a receipt acking up_to=N is ITSELF an
envelope at an offset > N included in the topic total, so the difference never
reaches 0."* `check-outbox.sh` was repaired there; `check-receiver-ack-lag.sh`
carries its own note (line 34) that `lag: 0` was once unreachable and now uses
`latest_content_offset`; `count_unread` was never affected (`UNREAD_META_TYPES`,
T-1332). `agent inbox` is the fourth surface and the surviving one. T-2757 worked
directly on this function and fixed the *cursor-vs-receipt* divergence (PL-350)
without touching the subtraction underneath it.

**What landed, and what it does NOT cover.** A convergence regression test,
`count_unread_converges_across_repeated_acks`, now models the ack loop and asserts
termination. It is load-bearing (dropping `"receipt"` from `UNREAD_META_TYPES` turns
it red). **It covers `count_unread` only.** It does not touch `unread_verdict` and
would not have caught this defect — stated explicitly so a green suite is never read
as this task being closed.

**Still open:** the fix itself. `unread_verdict`'s docstring warns that trading a
loud over-count for a silent under-count is worse, not better (Directive #2), so the
choice between a hub-side latest-content-offset and a client-side envelope walk is a
real decision and is deliberately not being made in passing.

### As filed (2026-08-31)

`unread` is `latest - receipt_up_to`. `channel ack` posts the receipt as an ordinary
envelope on the topic, which advances `latest`. So an ack sets `receipt_up_to` to the
pre-post latest and simultaneously creates one new envelope above it: the receipt itself.
The count it was issued to clear is reproduced by the act of clearing it.

Measured on `dm:d1993c2c3ec44c94:deadbeefdeadbeef`, three consecutive acks:

```
ack #1 -> latest=7  receipt_up_to=6  unread=1
ack #2 -> latest=8  receipt_up_to=7  unread=1
ack #3 -> latest=9  receipt_up_to=8  unread=1
```

Deterministic, not a race — the gap is exactly 1 every time and does not decay.

**Why nothing caught it.** A single-ack test passes: unread *does* drop, from whatever it
was to 1. The defect is only visible when you ask for convergence, and no test asks. This
is the same shape as a guard that proves its own copy rather than the live path — the
assertion is adjacent to the property.

**Scope is wider than one DM topic.** It applies to any topic the local identity also
posts to, which includes `agent-chat-arc`. That has a direct consequence for work
recorded as complete: the T-2838 receiver-ack-lag guard was brought to lag=0 earlier
today and is already back to lag=7 after two broadcasts from this session. The remediation
is self-defeating on precisely the topics we participate in — acking is not a fix there,
it is a reset that the next post undoes. `framework:pickup` stays at 0 only because we
have not posted to it since.

**The second-order harm is the reason to prioritise it.** An unread count that cannot
reach zero is an alert that is always on, and an alert that is always on stops being
read. `/check-arc` surfaces this number to the operator. A peer made the general form of
this argument to us about flaky gates training people into `--force`; this is the same
mechanism in the inbox rather than the gate.

**Deliberately not decided here:** whether the receipt should be excluded from the count
or should carry its own offset in `up_to`. Those differ in what a PEER sees of our read
state, which is a protocol-visible choice and belongs in the first AC, not in a hurried
fix.


## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] **Establish where the off-by-one belongs.** Still open, and now correctly located at `unread_verdict()` rather than at `ack`. Options: (a) the hub exposes a *latest content offset* alongside `latest_offset` so the subtraction skips meta envelopes at source; (b) the client walks envelopes per topic to classify them — accurate but O(N) per topic, which is the cost `agent inbox` uses counts to avoid; (c) `ack` sets `up_to` to the offset its own receipt will occupy. Record which, and why the others were rejected; (c) is protocol-visible to peers reading our receipt frontier.
- [x] **Reproduce against the actual surface, not an adjacent one.** Three consecutive acks on one topic: `agent inbox` reports `unread=1` every time while `channel unread` reports `0` every time. Both measured at the same moment. The filing's table is confirmed; only the verb it named was wrong.
- [x] **A convergence test exists for `count_unread`.** `count_unread_converges_across_repeated_acks` models ack-through-tail + append-receipt ×3 and asserts the true count is 0 while the naive `latest - up_to` gap reproduces at 1. Load-bearing: dropping `"receipt"` from `UNREAD_META_TYPES` fails it (rc=101), restoring returns 7/7 green. **Scope: this covers `count_unread` only and would NOT have caught the `unread_verdict` defect.**
- [ ] **`agent inbox` converges after the fix.** Same three-ack drive must leave `agent inbox` reporting `unread=0`, and a regression test must assert it — the defect survived because no test asked this function for convergence either.
- [ ] **The two verbs agree.** `agent inbox` and `channel unread` must return the same answer for the same topic at the same moment. They currently differ by exactly 1, and PL-350 already recorded that such a disagreement is a contradiction rather than a scope difference.

### Human
- [ ] [REVIEW] Choose how `agent inbox` should stop counting its own receipts.
  **Steps:**
  1. Reproduce: `termlink channel ack dm:d1993c2c3ec44c94:deadbeefdeadbeef` then `termlink agent inbox --json` — repeat 3×. `unread` stays 1; `termlink channel unread <same topic> --json` returns 0 throughout.
  2. Read `unread_verdict()` in `crates/termlink-cli/src/commands/channel.rs` — note its docstring warning that a silent under-count is worse than a loud over-count (Directive #2).
  3. Pick one: **(a) hub-side** — expose a *latest content offset* beside `latest_offset` so the subtraction skips meta envelopes at source. Correct for every client, needs a hub change and a version floor, and stale hubs must degrade to today's behaviour rather than to a wrong zero. **(b) client-side walk** — fetch and classify envelopes per topic. No hub change, but O(N) per topic, which is the cost `agent inbox` uses counts to avoid. **(c) ack-side** — `ack` sets `up_to` to the offset its own receipt will occupy. Smallest change, but it alters what a peer infers from our receipt frontier, so it is protocol-visible.
  **Expected:** one of a/b/c recorded here with a line of reasoning, and a follow-up build task filed against `unread_verdict()`.
  **If not:** leave unchecked. The over-count is loud and bounded at 1 per acked topic — annoying and trust-eroding, not data-losing — so this can wait for a considered answer.

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
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

grep -q "count_unread_converges_across_repeated_acks" crates/termlink-cli/src/commands/channel.rs
grep -q "\"receipt\", \"reaction\", \"redaction\", \"edit\", \"topic_metadata\"" crates/termlink-cli/src/commands/channel.rs
timeout 600 cargo test -p termlink count_unread > /tmp/.t2872-tests.txt 2>&1 && grep -q "count_unread_converges_across_repeated_acks ... ok" /tmp/.t2872-tests.txt
timeout 240 bash scripts/check-receiver-ack-lag.sh > /tmp/.t2872-lag.txt 2>&1

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

## Recommendation

**Recommendation:** GO on option (a) — a hub-side monotonic `latest_content_offset`
counter maintained beside `next_offset`.

**Rationale:** The task framed (a) as "the hub exposes a field", implying the value
is already known hub-side and only needs surfacing. Measured, it is not — and that
changes which option is cheapest rather than merely how (a) is worded. `latest_offset`
is not computed from envelopes at all: it reads the monotonic `next_offset` counter in
the `offsets` table (`crates/termlink-bus/src/meta.rs:256`). The `records` table
(`meta.rs:907`) carries only `topic, offset, byte_pos, length, ts_unix_ms` — **no
`msg_type`**; envelope bodies live in the log file at `byte_pos`. So no hub-side query
can classify an envelope without reading and parsing it, and (a) is really "add a
second counter", not "return a value we already have".

That is a smaller change than the task assumed, and it is a shape already proven in
this codebase: increment on post when `msg_type` is not in `UNREAD_META_TYPES`, O(1),
never rewound by sweep — structurally identical to `next_offset`, which T-2533 already
established as the correct basis for unread math.

**The obvious implementation of (a) is the wrong one, and it fails exactly as T-2533
did.** `SELECT MAX(offset) ... WHERE msg_type NOT IN (...)` — the reading (a) invites —
is not available (no column), but even given the column it would be **sweep-fragile**:
`MAX` over live records rewinds when content rows are swept, under-reporting latest by
the number of swept records and silently dropping unread rows. That is the precise
defect T-2533 fixed by moving off `count - 1`. A stored counter is immune; a query is
not. Recording this because it is the trap a later implementer walks into.

**Why not (b) — client-side envelope walk.** Correct, no hub change, and it is what
`channel ack-status` already does (`ack_status_rows`, T-2838). But `agent inbox` is a
digest over EVERY tracked topic, and the walk is O(N) envelopes per topic. The
`reconcile_consumption_frontier` docstring already refuses this for the same reason:
"a full-topic walk per topic would turn a cheap read into an O(topics x ...)". Adopting
(b) here would make the operator's most-used verb the most expensive one, which is how a
digest stops being run.

**Why not (c) — `ack` sets `up_to` to its own receipt's offset.** Smallest diff and it
does converge, but it is protocol-visible: a peer reading our receipt frontier would see
us claim consumption of an offset that is our own receipt rather than their content.
`compute_ack_status` (T-2838) derives peer lag from exactly that frontier, so (c) buys
local convergence by putting a small lie into the value other agents measure us by. It
also only fixes receipts WE write — a peer's ack on a shared topic still inflates
`latest`, so `agent-chat-arc` keeps drifting. Wrong axis and incomplete.

**A local-only variant was considered and rejected as incomplete.** The client could
record the offset its own receipt occupied and fold it into `reconcile_consumption_frontier`
— purely local, no protocol change, no hub change. It fixes the measured single-identity
DM case completely. It does NOT fix multi-party topics: another agent's receipt still
raises `latest`. Since `agent-chat-arc` is named in the filing as the topic that matters,
a fix that cannot converge there is not a fix.

**Evidence:**
- `unread_verdict` (`channel.rs:8880`) is `latest - frontier`, with no envelope in scope
  to classify; its only production caller `compute_unread_rows` (`channel.rs:8953`) takes
  `topic_latest` from `channel.list`'s `latest_offset`. Confirms the task's location.
- `latest_content_offset` (`channel.rs:4310`) is **client-side**, over `&[Value]`. It is
  not a hub field, so option (a) is not already shipped — checked because the task text
  could be read as implying it was.
- Hub `latest_offset` (`meta.rs:256`) = `next_offset - 1`, a stored monotonic counter.
- `records` schema (`meta.rs:907`) has no `msg_type`.
- Two surfaces already fixed this class client-side by walking envelopes:
  `ack_status_rows` (T-2838) and `channel reply` (T-1334). Both hold envelopes already;
  `agent inbox` deliberately does not.

**Version-floor behaviour (required by the first AC, stated so it is not lost):** a hub
that does not serve the new field must leave the client on today's arithmetic — a loud
over-count of 1 — and must never be allowed to degrade to a wrong zero. `compute_unread_rows`
already carries the right shape for this in its `authoritative` flag and its
`UnreadVerdict::Indeterminate` arm, which exists precisely so an unknown stays visible
instead of being reported as caught-up.

**What remains after ratification:** one build task against `unread_verdict` /
`compute_unread_rows` plus the bus counter, carrying ACs 4 and 5 (convergence of
`agent inbox`, and agreement with `channel unread`) and a convergence regression test on
`unread_verdict` — the one that would actually have caught this, which the existing
`count_unread_converges_across_repeated_acks` explicitly does not.

## Decisions

### 2026-09-01 — where the off-by-one fix belongs (analysis; ratification pending)
- **Analysed:** hub-side `latest_content_offset` counter (option a) recommended; (b)
  client walk and (c) ack-side `up_to` rejected, along with a local-receipt-offset
  variant considered during this pass.
- **Why:** see `## Recommendation` — the deciding measurement is that the hub has no
  `msg_type` in `records`, so (a) is "add an O(1) counter beside `next_offset`", not
  "surface an existing value", and the query-based reading of (a) reintroduces T-2533.
- **Not decided here:** the choice is protocol-adjacent and is reserved to the Human
  `[REVIEW]` AC. This entry records the evidence, not a ratification — Agent AC 1 stays
  unticked deliberately, because it asks which option was chosen and that is not the
  agent's to answer.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-31T19:46:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2872-channel-ack-cannot-reach-unread0-on-a-to.md
- **Context:** Initial task creation

### 2026-08-31T19:47:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-02T08:45Z — the a/b/c options priced against the actual storage layer [claude-code]

The previous commit (`edc1daf5a`) recorded "hub has no msg_type in records — option (a)
is a counter, not a field" but never landed that in this file, so the human AC still
offers three options with no costs attached. Measured now, with file:line, because a
choice between three unpriced options is not a choice.

**Correction to the shorthand first.** "The hub has no msg_type" is too strong and would
mislead the decision. `msg_type` **is** a first-class field on `Envelope`
(`crates/termlink-bus/src/envelope.rs:23`) — a sibling of `payload`, not something buried
in caller-supplied JSON, so it is structured and signed. What it is not is **indexed**:
the SQLite `records` table is `(topic, offset, byte_pos, length, ts_unix_ms)`
(`crates/termlink-bus/src/meta.rs:907-913`) and `msg_type` appears in no SQL statement in
the crate. So the constraint is "not queryable", not "not present".

**(a) hub-side latest-content-offset — splits into two very different changes.**
`channel.list` returns `{name, retention, count, latest_offset?}`
(`crates/termlink-hub/src/channel.rs:1689-1693`); `latest_offset` is an O(1) read of the
monotonic `offsets.next_offset` counter (`meta.rs:256-269`) and is msg_type-blind. So:
- **(a1) compute by scan.** Any msg_type-conditional offset must decode each envelope
  from the log. There is direct precedent in the hub — `channel.receipts` does exactly
  this with a `records_scanned` counter and a **wall-clock deadline guard**
  (`crates/termlink-hub/src/channel.rs:1391-1405`), and `find_idle_agents` walks a whole
  topic (`crates/termlink-bus/src/lib.rs:584-597`). Cheap to write, bounded by a deadline,
  but it is O(N) moved from the client to the hub rather than removed.
- **(a2) index msg_type.** Add it as a column on `records` and the read becomes cheap for
  every client, forever. This is a schema migration plus a version floor, and per the
  T-2359 note stale hubs must degrade to **today's over-count**, never to a wrong zero.
- `cv_index` is not a shortcut: it is in-memory, non-persistent, keyed on a caller-supplied
  `cv_key`, capped at 1000/topic (`crates/termlink-hub/src/cv_index.rs:57`).

**(b) client-side walk — already exists, which changes its price.** `channel info` is
**not** a hub RPC; it is a client-side composite that already does a full paginated
`channel.subscribe` walk and filters `msg_type == "receipt"` in the client
(`crates/termlink-cli/src/commands/channel.rs:4169-4209`). So (b) is not a new cost
centre, it is reusing a walk this codebase already pays and already ships. The O(N)
objection in the AC stands on its merits but is no longer novel.

**(c) ack-side** is unchanged by this measurement and remains the only protocol-visible
option — it alters what a peer infers from our receipt frontier.

**Not deciding.** The pick is the human's per the [REVIEW] AC, and (c) in particular
changes something peers observe. This entry exists so the choice is made against measured
costs rather than three equally-weighted sentences.

### 2026-09-02T09:15Z — this task's 4th verification line gates it on other agents' behaviour [claude-code]

Today's guard-layer sweep fired `check-receiver-ack-lag.sh`, which is **line 4 of this
task's own `## Verification` block**. So P-011 will refuse `work-completed` here until it
goes green. Current reading:

```
agent-chat-arc  (3 distinct identity row(s))
  NEVER-ACKED  33df8954b2a9b70d  lag=981
  NEVER-ACKED  9219671e28054458  lag=981
  BEHIND       d1993c2c3ec44c94  lag=57 (up_to=923)
framework:pickup  (1 row)
  ok           d1993c2c3ec44c94  lag=0
```

**Two of the three firing rows are not ours.** `d1993c2c3ec44c94` is this host's identity —
it is the self-fingerprint in this task's own reproduction command,
`dm:d1993c2c3ec44c94:deadbeefdeadbeef`. The two `NEVER-ACKED` rows at lag=981 are other
identities that have never acked anything on `agent-chat-arc`.

**Why that is a problem for this task specifically.** Whichever of (a)/(b)/(c) is chosen,
the fix changes how *we* COUNT unread — it does not cause a peer to ACK. So the dominant
term in this check will still be red after a correct fix, and the gate will block a task
that is genuinely done. That is the shape this repo has named from the other direction in
T-2818: a gate that blocks for reasons the author cannot act on teaches people that
failures here are noise and that `--force` is the normal way past them.

**Not changing the block unilaterally.** Removing or narrowing a verification line is
exactly the move that should not be made quietly by whoever is inconvenienced by it, and
`--force` is worse. Recording it instead, so the choice is made deliberately alongside the
a/b/c decision. The options are: scope the line to our own identity (it is the only one our
fix can move), keep it and accept that closure waits on peers acking, or drop it here and
let the guard layer own it — where it already runs daily and is not attached to one task's
completion.

**Our own row is separately real.** `d1993c2c3ec44c94 BEHIND lag=57` on `agent-chat-arc` is
this host not keeping up with a broadcast topic. That is worth its own look and is NOT what
this task is about — filed nowhere yet, deliberately named here so it is not lost.
