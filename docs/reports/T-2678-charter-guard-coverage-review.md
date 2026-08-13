# T-2678 — Charter Guard-Coverage Review

**Type:** inception (exploration → go/no-go)
**Date:** 2026-08-14
**Predecessor arcs:** T-2419 (1st purpose review), T-2468 (2nd purpose review, P1–P6),
T-2470 (P1 charter), T-2471/T-2478 (P4 surface reduction), T-2483/T-2484 (charter canaries)

## The question

The two prior purpose reviews asked *"is TermLink's stated purpose right, and does the
product match it?"* Both answered by measuring the **product against the charter**.

This review asks the next question up the stack: **is the charter itself load-bearing?**
Not "is the purpose correct" but "what mechanically happens if the product drifts from
it?" A charter that nothing enforces is a wish, and the T-2468 arc's own closing lesson
(PL-271) was that *a recurring human "review X purpose/scope" mandate is itself the symptom
of a missing structural check*. If the fifth such review still has to be run by hand, the
first four did not close the loop.

## Method

1. Read `docs/CHARTER.md` and decompose it into its two enforceable halves — the four
   **verbs** (what TermLink must do) and the five **non-goals** (what it must refuse).
2. For each half, inventory what structurally enforces it: an affirmative **prover**
   (proves it works on demand), a **canary** (detects when it broke, daily), or a
   **guard/test** (blocks the drift at source).
3. Where a guard exists, test whether it is *load-bearing* — feed it the violation it
   claims to catch and confirm it fires.
4. Look for shipped surfaces the charter does not account for at all.

Step 3 is the part prior reviews skipped, and it is where the sharpest finding came from.

## Finding F1 — The charter's verbs are guarded 4/4; its non-goals 2/5, and nothing tracks the matrix

The verb half of the charter is in excellent shape. Every one of the four verbs has both
an affirmative prover and a daily canary:

| Charter verb | Prover | Daily canary |
|---|---|---|
| 1. discover | `comms-selftest.sh` (T-2482) | waker-liveness (T-2387) |
| 2. exchange durable messages | `comms-selftest.sh` (T-2482) | dead-letter (T-2558), unconfirmed-delivery (T-2295) |
| 3. claim work | `substrate-smoke.sh` (T-2151) | stuck-claims (T-2556) |
| 4. control terminal sessions | `session-selftest.sh` (T-2485) | session-control (T-2557) |

That matrix was deliberately completed — T-2556 and T-2557 exist *because* someone
noticed verbs 3 and 4 had no canary. The completeness of the verb axis is proof that this
project knows how to close a coverage matrix once it can see one.

The non-goal half has had no such pass:

| Charter non-goal | Structural guard | Status |
|---|---|---|
| 1. Not an inter-hub federation layer | *none* | **UNGUARDED** — T-2569 filed 2026-08-09, `captured` / `horizon: later` ever since |
| 2. Not a durable database | forever-archival canary (T-2562) | guarded |
| 3. Not a social / engagement platform | charter-drift canary (T-2483) | guarded **but false-assuring** — see F2 |
| 4. Not a workflow/orchestration engine | *none* | UNGUARDED — T-2570 open, `owner: human` (out of agent authority) |
| 5. Not a security boundary | *documented only* (README § Trust Model) | stated clearly; not mechanically guardable — acceptable |

**Coverage: 2 of 5, and one of the two is weaker than it reports.**

The root problem is not any individual missing guard — it is that **no artifact asserts
this matrix exists**. Both tables above had to be reconstructed by hand for this review.
Because nothing enumerates "every charter non-goal shall declare a guard", T-2569 could be
filed, correctly scoped, agent-owned, and then sit untouched at `horizon: later` with
nothing surfacing it. That is precisely the G-019 shape the project applies everywhere
else: *fix the symptom, then ask why the framework was blind.* Filing T-2569 was the
symptom fix. Nothing did the second half.

Non-goal 1 deserves specific note as the most violable of the five. G-060 exists because
peer projects have repeatedly *assumed* federation; `docs/operations/channel-topic-semantics.md`
exists to explain that assumption away; a live ring20 RCA (T-2229) named cross-hub
"federation" as a root cause. It is the non-goal most likely to be violated by a
well-meaning future change, and it is the one with zero tests.

## Finding F2 — The charter-drift canary reports a full-surface clean bill it cannot actually measure

This is the sharpest finding, and it inverts the reassurance the project currently has.

`scripts/check-charter-drift-freshness.sh` (T-2483) is the guard for non-goal 3. Its own
header states its purpose as detecting when *"the tool surface has drifted from the
charter's four verbs"*. `CLAUDE.md` records its result as **"214 live tools scanned, 0
off-charter"**. Run today it emits:

```json
{"ok":true,"firing":[],"checked":214,"live_off_charter":0}
```

That reads as: *the entire live MCP surface was examined and every tool traces to the
charter.* It does not. Mechanically the canary applies a **fixed six-family name regex**
(reactions / emoji / stars / pins / typing / polls) — the exact families P4 happened to
delete. It has no model of charter-traceability at all. `checked:214` counts tools the
regex was *run against*, not tools whose purpose was *assessed*.

The blind spot is not hypothetical. 28 tools are LIVE right now in categories the binary
itself names as analytics:

| Category | Live | Examples |
|---|---|---|
| `agent_rankings` | 5 | `top_replied`, `top_repliers`, `top_thread_starters`, `first_post_by`, `first_responders` |
| `agent_stats` | 10 | `stats`, `response_latency`, `topic_stats`, `user_summary` |
| `agent_thread_health` | 8 | `thread_health`, `busiest_threads`, `idle_threads`, `quiet_threads` |
| `channel_engagement` | 5 | `mentions`, `mentions_of`, `digest`, `snippet` |

`termlink_agent_top_repliers` is a social leaderboard. It is the same class of surface as
`termlink_agent_top_reacted` and `termlink_agent_top_pinners`, which P4 deprecated as
charter-untraceable. The only difference is that "reacted" is in the regex and "repliers"
is not.

**Demonstrated with the canary's own PL-213 test hook:**

```
$ TERMLINK_CHARTER_DRIFT_TEST_JSON=a.json ...   # a.json: top_reacted, live
{"ok":false,"firing":[{"name":"termlink_agent_top_reacted", ...}],"checked":1,"live_off_charter":1}
exit=1

$ TERMLINK_CHARTER_DRIFT_TEST_JSON=b.json ...   # b.json: top_repliers, live
{"ok":true,"firing":[],"checked":1,"live_off_charter":0}
exit=0
```

Identical class, opposite verdict.

Two consequences, and the second is worse than the first:

1. **Under-detection.** A new `termlink_agent_top_<anything>` ships without firing anything.
2. **False assurance.** The 28 tools are the *precise* family that **T-2548**
   ("Charter non-goal #4: conversation-analytics MCP tool family subtract-vs-keep",
   `started-work`, `owner: human`) is currently incepting to remove. So the project has an
   open, human-owned decision about ~30 off-charter tools, and simultaneously a daily
   canary reporting that surface as 0-off-charter. An operator reading `/canaries` sees
   green. This is a Directive #2 (no silent failures) violation *in the guard layer itself* —
   the place it is most costly, because a guard reporting green is why nobody looks.

The fix is **not** "delete the 28" — that decision is T-2548's and belongs to the human.
The fix is that the canary must stop claiming to have measured what it did not, and must
make the pending decision *visible* instead of invisible.

## Finding F3 — Unbounded per-session `kv` store in the daemon that owns real PTYs

`SessionContext.kv` is a plain `HashMap<String, serde_json::Value>`
(`crates/termlink-session/src/handler.rs:29`). `handle_kv_set` does:

```rust
let replaced = ctx.kv.insert(key.clone(), value.clone()).is_some();
```

No cap on key count. No cap on value size. No eviction. `kv.set` requires only
**Interact** scope (`crates/termlink-session/src/auth.rs:191`) — in a cooperating fleet,
every authenticated peer. The map lives in the **session daemon**, the process that owns
the real PTYs backing charter verb 4. Growing it without bound OOM-kills the user's
terminal sessions.

This is not a novel class for this repo — it is the *same* class closed twice in the
immediately preceding session:

- T-2675 — bounded `PresenceTracker`
- T-2676 — bounded circuit-breaker map

Both were "unbounded peer-influenced map in a long-lived daemon". `kv` is the third
instance, and it sits in the most consequential process of the three.

Charter-wise, `kv` is also one of the surfaces the charter does not name. It is defensible
as session-scoped coordination state under verb 4 — it is in-memory, per-session, and dies
with the session, so it does **not** violate non-goal 2. Flagging it as a charter question
is not the point; bounding it is.

## What is NOT wrong (recorded so the next review does not re-litigate)

- **Non-goal 5 is well handled.** README § Trust Model states the UDS-vs-TCP anchor split
  and says plainly "treat every authenticated peer as trusted … not yet for adversarial
  multi-tenancy". Clear, user-facing, honest. No action.
- **The cron layer is live, not shipped-dark.** `check-cron-install-drift.sh` reports 22
  installed-and-matching, **0 MISSING**. Two content-drift warnings
  (`fleet-doorbell-mail-canary`, `substrate-preflight-canary`) are worth reconciling but
  are not a coverage hole.
- **The canonical sentence is genuinely in sync** across CHARTER/README/ARCHITECTURE
  (T-2484 canary passing).
- **Verb coverage is complete and was completed deliberately.** No action.

## Prioritized gap register

| # | Gap | Charter half | Severity | Authority |
|---|-----|--------------|----------|-----------|
| **G1** | charter-drift canary reports a full-surface clean bill it cannot measure; 28 live analytics tools invisible; T-2548's open decision reported as green | non-goal 3 | **HIGH** | agent — honesty fix only; deletion stays T-2548/human |
| **G2** | Non-goal 1 (no federation) has zero regression test; most-violable non-goal, unguarded | non-goal 1 | **HIGH** | agent (T-2569 is `owner: agent`) |
| **G3** | Unbounded `kv` HashMap in the PTY-owning session daemon at Interact scope | verb 4 | **MED-HIGH** | agent |
| **G4** | No artifact asserts the non-goal↔guard matrix, so coverage gaps are invisible and filed guards rot at `horizon: later` | meta | **MED** | agent |
| **G5** | Two installed crontabs drift from git source | ops hygiene | **LOW** | agent |
| **G6** | Non-goal 4 (orchestration) unguarded | non-goal 4 | MED | **human** — T-2570 / T-2540 / T-2549 open |

## Recommendation — GO on the in-authority subset

**GO**, staged, on G1–G5. **G6 stays with the human** (T-2570), and nothing here pre-empts
the T-2548 subtract-vs-keep decision — G1 is deliberately scoped to making the canary
*honest about what it measured*, not to removing anything. An acknowledgement allowlist is
the right instrument precisely because it renders the pending human decision visible rather
than resolving it.

The through-line of this review is that TermLink's charter is well written and half
enforced: rigorous about proving what it does, thin about refusing what it says it
refuses — and, in the one place a refusal-guard exists, that guard over-reports. Fixing
the over-report is worth more than adding any new guard, because a guard reporting green
is the reason the gap survived five purpose reviews.

Per inception discipline, no build artifacts are written under T-2678. On GO each item
becomes its own build task.

## Dialogue Log

**Human (2026-08-14):** "please ultra critically review termlink's purpose and goals and
identify gaps or needed adjustment, incept these and build these and test these, drive to
completion."

**Agent:** Read the charter; found that both prior reviews had measured product-vs-charter
and neither had tested whether the charter's own guards fire. Pivoted the review to
guard-coverage. Established the verb matrix (4/4) and the non-goal matrix (2/5) by hand,
then stress-tested the one non-goal guard that exists (T-2483) with its own test hook and
found it structurally unable to see the family an open human-owned inception is trying to
remove. Recommendation GO on the in-authority subset; the deletion decision left with the
human.
