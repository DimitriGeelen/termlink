# T-2848 — the 75 finished-and-waiting tasks, triaged

**Date:** 2026-08-28 · **Task:** T-2848 · **Tree:** `worktree-charter-review-2026-0814`

## What this is

75 tasks in `.tasks/active/` have **every agent AC ticked and none open**, and still carry
at least one open `### Human` AC. They are finished work waiting on a human. This document
turns that pile into a decision list with evidence, so the queue can be worked in sittings
rather than task-by-task.

**Nothing here ticks a Human AC.** Only the human may do that (CLAUDE.md, Human Task
Completion Rule). Nor can any of them be closed from this worktree: closing moves the file
into `.tasks/completed/`, which is a task-corpus change, and the T-3110 gate refuses those
from a linked worktree. Every action below happens at the authority (`/opt/termlink`).

Population selected by predicate, not by `owner:` alone — the same correction
832-Workflow-designer made on agent-chat-arc @607, where keying on `owner: human` alone
inflated their count from 16 to 26 by sweeping in unstarted backlog that nobody was
waiting on.

## The shape of it

| | count |
|---|---|
| total finished-and-waiting | **75** |
| `[REVIEW]` — genuine human judgement | 63 |
| `[RUBBER-STAMP]` — mechanical, no judgement | 12 |
| already `status: work-completed` (pure partial-complete) | 59 |
| still `status: started-work` | 16 |

**63 `[REVIEW]` items collapse to ~22 decisions, and two of those cover 30 tasks.**

## Group 1 — the CLI output-taste family (30 tasks, 2 sittings)

`T-1482..T-1506` (21) and `T-1529..T-1537` (9) are one shipped feature family each. Sampled
Human ACs are the same sentence repeated: *"Verify the verb reads naturally"*, *"Verify
offset rendering reads naturally"*, *"Verify text-mode table is scannable"*. Pure output
taste on the `agent presence / recent / timeline / on-thread / forward / edit / redact /
threads / pin-history / edits-of / relations` surface.

**Checked, because it would have changed the answer:** P4 (T-2471/T-2478) deleted 52 tools
from the surface, so some of these verbs could have been removed — which would make their
ACs moot and close them as superseded rather than needing review. **All 11 probed verbs are
alive.** None close as superseded; the review is real. This is a clean negative and it also
confirms the 30 are not stale cruft.

`[REVIEW]` is the correct marker here — "reads naturally" is genuine taste and is not
grep-able, so the T-1811/T-1878 conversion to `[REVIEWER]` does not apply.

**Suggested handling:** one terminal sitting per family. Run the verbs, look at the output,
tick or file a follow-up. 30 of 75 clear in two sittings.

## Group 2 — rubber-stamps with evidence already gathered

| task | Human AC | evidence | verdict |
|---|---|---|---|
| **T-1696** | Cron entry installed in `/etc/cron.d` on .107 | `/etc/cron.d/termlink-release-mirror-canary` **present** | **satisfied** — evidence supports closing |
| **T-1723** | Cron entry installed so the meta-canary actually fires | 9 `check-canary-aliveness` job lines installed, including on `termlink-release-mirror-canary` | **satisfied** — evidence supports closing |
| **T-2408** | Close arc `mcp-slimming` with demo evidence | `.context/arcs/mcp-slimming.yaml` → `status: in-progress`, anchor `T-2406` | **not yet** — closing the arc *is* the action |
| **T-2711** | Decide whether U-001 is filed to `framework:pickup` | 65 envelopes scanned, **no U-001** | **not yet** — an undecided call, not a missed step |
| **T-2723** | Decide whether U-008 is filed to `framework:pickup` | 65 envelopes scanned, **no U-008** | **not yet** — same |
| **T-1420** | Binary deployed on .141 | .141 is the 1 FAILED hub in `fleet doctor`; unreachable all session | **blocked** — cannot be satisfied until .141 returns |
| **T-2013** | Deploy fixed binary to .122/.121/.141, confirm 5/5 | fleet is version-skewed (0.11.1196 ×2, 0.11.1411, 0.11.588, 1 unknown) and .141 is down | **blocked** — same dependency |
| **T-1722** | Upstream landed on `/opt/999-AEF` `origin/master` | cross-project path; T-559 boundary gate refuses inspection from here | **unverifiable here** — ask 999-AEF on the rail |
| **T-1691** | GitHub Release published with macOS + Linux binaries | not probed (outward-facing; deliberately left to the operator) | **unverified** |
| **T-2297** | Live end-to-end after installing rebuilt hub binary | depends on the same fleet upgrade as T-2013 | **blocked** |
| **T-2194** | Batch-click ripe partial-completes | this document is the evidence refresh it asks for | **ready** |
| **T-1885** | mixed `[REVIEW]` + `[RUBBER-STAMP]` | — | needs its own read |

**Two are closeable on evidence today: T-1696 and T-1723.** Four (T-1420, T-2013, T-2297,
and partly T-1691) share a single blocker: **the fleet is version-skewed and .141 is down.**
That is one operator action gating four tasks.

## Group 3 — singles needing individual judgement (14)

`T-1442 · T-1453 · T-1570 · T-1673 · T-1695 · T-1795 · T-1799 · T-2014 · T-2197 · T-2385 ·
T-2409 · T-2470 · T-2644 · T-2709 · T-2790`, plus the small clusters `T-1415..T-1419` (3),
`T-1426..T-1435` (4), `T-1557..T-1559` (3), `T-1632..T-1635` (3), `T-2209..T-2213` (5).

Two worth pulling forward:

- **T-1799** — *"Purge leaked GitHub PAT from local git object store."* A leaked credential
  is the one item here that does not age gracefully. Recommend reading this one first
  regardless of BVP.
- **T-2470** — *"Reconcile TermLink purpose into one canonical charter"* — sits on the
  charter arc this worktree is named for.

## The honest caveat

This triage reads AC **text and status**; it does not re-run each task's original
verification. A task whose agent ACs were ticked in error is invisible to it. The value is
routing — turning 75 opaque items into ~22 decisions with the blockers named — not a claim
that all 75 are correctly finished.
