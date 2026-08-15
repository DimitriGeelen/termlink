---
id: T-2719
name: "agent inbox unread count mixes retention-relative index with absolute cursor"
description: >
  termlink agent inbox subtracts an absolute cursor from a retention-window-relative
  latest index, over-reporting unread by 19x on a topic past its retention cap;
  agent unread reports the true count from absolute offsets
status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-14T21:20:00Z
last_update: 2026-08-15T05:19:25Z
date_finished: null
---

# T-2719: agent inbox unread count mixes retention-relative index with absolute cursor

## Context

Reported by the 999-AEF peer session (uds:/tmp/cc-socks/124667.sock) on
2026-08-14, homed here because the fix lives in this repo. Reproduced on
consecutive invocations with no posts in between.

**Two verbs disagree about the same topic in the same second:**

```
$ termlink agent inbox  | grep agent-chat-arc
  agent-chat-arc — 389 unread (latest=2000, cursor=1611)

$ termlink agent unread | grep agent-chat-arc
Topic 'agent-chat-arc': 20 unread for d1993c2c3ec44c94 (first new offset 11838, last 11857, last receipt up_to=11836)
```

389 vs 20 — a 19× disagreement, with nothing indicating which to trust.

**Diagnosis (peer's, and it is self-consistent from the numbers alone).**
`termlink agent info` reports:

```
Topic: agent-chat-arc
Retention: messages:2000
Posts: 2001
```

`latest=2000` in the inbox line is a **retention-window-relative** index — it is
the window size, not an absolute offset. The cursor `1611` is then subtracted from
it as though both were in the same coordinate space: `2000 - 1611 = 389`, exactly
the printed figure. `agent unread` uses **absolute** offsets (11838 / 11857 /
11836) and arrives at 20, the true count.

The arithmetic reproducing the printed number exactly is strong evidence on its
own — this does not need a live hub to be credible, though it does need one to be
confirmed (see the prediction below, which is the actual test).

**Falsifiable prediction to run BEFORE patching** (peer's, and the right shape —
it can disprove the diagnosis rather than merely illustrate it):

> `inbox`'s number should be wrong only on topics whose post count has **exceeded**
> the retention cap, and should agree with `unread` on any topic still under it.

If that holds, the fix is one coordinate conversion. If it does not hold, the
diagnosis is wrong and the cause is elsewhere. **Run this first** — patching on an
unconfirmed read is how a coordinate bug gets moved rather than fixed.

**Why this matters more than the arithmetic.** The peer hit it while diagnosing a
cross-session coordination failure: another agent believed it had sent ten
unanswered messages and asked whether the peer's read cursor had wedged. That
question could not be answered by inspection, because the two verbs that exist to
answer exactly it disagreed 19×.

`inbox` is the summary view — the one an agent reaches for **first** — and it is
the one that is wrong. It over-reports, so it fails toward alarm rather than
silence. That is the better failure direction, but a permanently-alarming unread
count is functionally equivalent to no count at all, because you stop reading it.
This is the same class as the guard-layer findings from this session's audit
(T-2680, T-2709): a signal whose value does not correspond to the state it claims
to describe, which trains its reader to ignore it.

The underlying coordination failure turned out to be neither party's cursor — the
sending peer was writing into a DM topic the recipient's fingerprint is not a party
to. **So this bug did not cause that incident**; it made it undiagnosable for
longer than it should have been. Worth stating plainly so the fix is not
over-credited.

**Environment:** hub PID 3080, runtime `/var/lib/termlink`, local hub only,
identity `d1993c2c3ec44c94`, topic `agent-chat-arc` (Senders 3, Posts 2001,
Retention messages:2000).

**Status of verification at filing:** NOT yet reproduced in this repo. This session
hit the context-budget gate, which blocks Bash, so neither the live verbs nor the
source could be inspected. Everything above is the peer's evidence plus arithmetic
that checks out on its face. First action next session is the prediction test, then
locating the subtraction site.

---

## VERIFIED 2026-08-15 — prediction passed, diagnosis half-right, cause already fixed

The prediction test was run first, as the ACs required. **It passed.** But following
it to the source showed the peer's diagnosis is right about one operand and wrong
about the other, and that the code half of this bug **was already fixed a week ago
and never deployed**.

### 1. The prediction holds

Two topics, opposite sides of the retention cap, same second:

| Topic | Retention | Posts | `agent inbox` | `channel unread` | Verdict |
|---|---|---|---|---|---|
| `agent-chat-arc` | `messages:2000` | 2000 (**at cap, swept**) | **388** (latest=1999, cursor=1611) | **30** (offsets 11838..11867, receipt 11836) | 12.9× divergence |
| `dm:9219671e…:d1993c…` | `forever` | 172 (**never pruned**) | **127** (latest=171, cursor=44) | **125** (offsets 46..171, receipt 44) | agrees within 2 |

The clean proof is in the `latest` column: **`latest = Posts − 1` in both rows.**
It is a count-derived index, always. It coincides with the true last offset only
while nothing has been pruned — 171 on the `forever` topic (correct), 1999 on the
swept topic where the true last offset is 11867 (wrong by 9868).

### 2. The `latest` operand: already fixed, at T-2533, 7 days ago

This is not a new bug. `T-2533` (`ac859d321`, 2026-08-08, **v0.11.871**) —
*"expose `Bus::latest_offset` — fix silent unread/ack data-loss on swept topics"* —
fixed exactly this, and the tests at `channel.rs:16402-16435` name the failure mode
in a comment: *"Empty latest map = pre-T-2533 hub → count-1 path"*.

The installed binary here is **0.11.720**, i.e. **151 commits before the fix**.
`latest_offset` is served hub-side, so the client silently takes the documented
`count-1` fallback.

**One thing T-2533 did not document:** its tests only cover the fallback
*under*-reporting (cursor ≥ count-1 → row dropped → 0 unread, silent data loss).
With the cursor *below* count-1 the same fallback **over**-reports instead — 388 vs
30. Both directions are wrong; only one was named and tested.

### 3. The `cursor` operand: a different defect, NOT what the peer diagnosed

`channel.rs:8800` — `agent inbox` reads cursors from **`~/.termlink/cursors.json`**,
a local file written by `subscribe --resume`. `channel unread` / `agent unread` read
the **hub-side ack receipt**. Two independent stores that are never reconciled:

```
~/.termlink/cursors.json →  "agent-chat-arc::d1993c2c3ec44c94": 1611
hub receipt              →  d1993c2c3ec44c94 up to 11836
```

The peer read `cursor=1611` as an absolute offset being subtracted from a
window-relative `latest`. It is neither — it is a **stale local subscribe-cursor**,
frozen since whenever `subscribe --resume` last ran, while the hub receipt advanced
to 11836.

**This matters for the fix order.** Upgrading to ≥0.11.871 alone makes the printed
number *worse*, not better: `latest` becomes the correct 11867 while `cursor` stays
1611, giving **10256 unread** against a true 30. The two verbs are structurally
incomparable until the cursor sources are reconciled — the upgrade fixes one operand
of a subtraction whose other operand comes from a different book.

### 4. Why nothing caught the stale binary — the guard was told to be quiet

`scripts/check-fleet-binary-freshness.sh` run ad-hoc today:

```
fleet-binary-freshness: healthy — all floored reachable hubs at/above floor
  ✓ local-test:              served=0.11.720 >= floor=0.11.679
  ✓ workstation-107-public:  served=0.11.720 >= floor=0.11.679
  ✓ ring20-management:       served=0.11.679 >= floor=0.11.679
```

Every reachable hub in the fleet is **below the T-2533 fix (0.11.871)** — by 151,
151, and 192 commits — and the canary reports healthy. It is behaving correctly:
`.context/cron/fleet-version-floors.conf` was last bumped 2026-07-27 to 0.11.679,
and its own header states the convention that was not followed:

> **BUMP THE FLOOR WHEN HUB-SIDE RAILS SHIP:** that is the operator's declaration
> that "shipped" must mean "capability-live".

T-2533 shipped a hub-side rail on 2026-08-08 and did not bump the floor. So a known
silent-data-loss fix sat undeployed for 7 days with the guard built to catch exactly
that reporting green daily. Split out as **T-2720** (G-019: the framework blindness,
distinct from this bug).

This is the **fifth** instance this session of a guard whose verdict rests on an
assumption that no longer holds — after T-2680, T-2709, T-2714, T-2715, T-2718.

## Acceptance Criteria

### Agent
- [x] The prediction is tested first: `inbox` and `unread` agree on a topic UNDER its retention cap, and disagree on one OVER it — recorded with both outputs *(§1 — passed; 127 vs 125 under cap, 388 vs 30 over)*
- [x] If the prediction fails, the task is re-scoped to the real cause rather than patched to match the peer's read *(prediction passed, but the `cursor` half of the diagnosis was wrong — re-scoped in §3 rather than patched to match)*
- [x] The subtraction site is located and cited by `file:line`, showing which operand is retention-relative and which is absolute *(`channel.rs:8749` compute_unread_rows; `latest` = count−1, `cursor` from `channel.rs:8800` local store — §2/§3)*
- [x] The code fix for the `latest` operand is identified *(T-2533 / `ac859d321` / v0.11.871 — already merged, not deployed)*
- [ ] The `cursor`-source divergence (`~/.termlink/cursors.json` vs hub ack receipt) is fixed or filed as its own task — it is NOT covered by T-2533 and upgrading alone makes the printed count worse
- [ ] T-2533's test suite gains the over-report direction of the `count-1` fallback (cursor below count−1), which its current tests do not cover
- [ ] `agent inbox` and `channel unread` are shown agreeing on the reporter's exact case, on a binary >= 0.11.871
- [x] The peer (999-AEF) gets the outcome: prediction passed, `latest` half already fixed in T-2533 and undeployed, `cursor` half is a separate defect their diagnosis did not cover *(sent 2026-08-15, msg 94180c21)*

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Fill in once the fix lands — the regression test is the load-bearing entry.
cargo test --workspace

## RCA

**Symptom:** `agent inbox` reports 389 unread where `agent unread` reports 20, on
the same topic in the same second, with no posts in between.

**Root cause (VERIFIED 2026-08-15):** `agent inbox` computes `latest - cursor`
where the two operands come from different books entirely.

- `latest` is `count - 1` (`compute_unread_rows`, `channel.rs:8749`), taken because
  the hub does not serve `latest_offset`. Correct only while nothing has been swept.
  **Fixed in T-2533 at v0.11.871; the fleet runs 0.11.720.**
- `cursor` is a **local** value from `~/.termlink/cursors.json` (`channel.rs:8800`),
  written by `subscribe --resume`. The verb it is compared against reads the
  **hub-side ack receipt**. Nothing reconciles the two. **Not fixed by T-2533.**

So the peer's "window-relative minus absolute" reading is right for `latest` and
wrong for `cursor`. The real shape is worse than a coordinate error: it is two
independent cursor systems, one local and one hub-side, presented through two verbs
as though they measured the same quantity.

**Why structurally allowed:** three compounding gaps.
1. Offsets, counts, and local cursors are distinct concepts but not distinct
   *types*, so `count - local_cursor` is well-formed and silently wrong.
2. No test compares `inbox` against `unread` on the same topic. Each verb is tested
   against its own model, so both pass while disagreeing 13×.
3. T-2533 fixed the operand and shipped the fix behind a **version floor that was
   never bumped**, so the guard that exists to catch undeployed hub rails reported
   healthy for 7 days. (→ T-2720)

**Prevention:** a test asserting `inbox` and `channel unread` agree on a topic
seeded past its retention cap — the cross-verb comparison neither suite makes today.
Plus the T-2533 fallback's over-report direction, which its tests omit. The stronger
form is a newtype separating offset-space from count-space so the subtraction cannot
compile; worth assessing, since the same `count-1` idiom appears in the MCP tool
(`tools.rs:31517`) too.

## Separate findings — NEED THEIR OWN TASKS

The peer reported two usability items alongside the bug. Recorded here so they are
not lost, but they are **distinct deliverables** and compounding them into this
task would violate the one-bug-one-task rule. File as their own tasks next session:

1. **`termlink agent recent <topic>` resolves its positional as a session.**
   Passing a DM topic yields `Session '<topic>' not found`, which reads as "that
   topic does not exist" rather than "wrong argument kind". Cost the reporter a
   hypothesis. Same class as T-2663 (a refusal that does not name what happened) —
   the message is not wrong, it is answering a different question than the one the
   operator asked.

2. **No verb maps a fingerprint to a name.** `agent who` and `agent who-is` both
   reject a fingerprint argument. The reporter ended up identifying a peer by
   reading a DM and recognising the sign-off. Needs confirming that no such verb
   exists before building one.

## Updates

### 2026-08-14T21:20:00Z — filed from peer report
- **Source:** 999-AEF session, cross-session message
- **Action:** filed without local reproduction; context-budget gate blocked Bash
- **Next:** run the falsifiable prediction before any patching
