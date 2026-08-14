# Audit remediation — status after pass 2 (2026-08-14)

**State: 224 PASS / 10 WARN / 0 FAIL.** Was 30 WARN before pass 1, 20 before
pass 2.

Pass 2 executed group A of the original plan. All ten inception research-artifact
warnings and both fabric-adjacent misreadings are resolved or correctly filed.
**The remaining 10 are at the honest floor** — see "Why zero is not reachable".

Everything is committed. Re-run `.agentic-framework/bin/fw audit` first to confirm
the list still matches before acting on anything below.

---

## What pass 2 did

### A1 — CTL-020 → filed as T-2715 / U-004, not fixable here

`.gitignore:54` excludes `.context/audits/cron/` by design, so a linked worktree
can never have it. **Both obvious fixes are wrong**, and the record says so: the
printed mitigation (`fw audit schedule install`) would aim a host cron entry at a
path deleted with the worktree, and `mkdir -p` would swap the warning for
"No cron audit files in last hour". The fix pattern already exists two sections up
in `audit.sh` (`fw_is_linked_worktree`, used at :1638 and :1708).

### A2 — C-002 → third symptom of T-2714, not a separate defect

`audit.sh:3022` greps the same concatenated `$PROJECT_ROOT/.git/hooks` path. The
installed hook **does** carry the `inception-research-warnings` marker (line 166)
and the C-001 enforcement block (lines 164-195). So C-002 is a false negative
about a **live safety gate** — worse than the sibling warnings, which merely claim
a file is missing.

### A3 — all ten research artifacts written (T-2716, completed)

Pass 1 declined these as a block. That was too coarse. Checked per task: T-2486 had
3/3 IW answered at confidence 3 with a shipped fix; T-2546 had A-1..A-4 verified
with `file:line` evidence; the six completed ones carried decision + rationale +
evidence. The trails existed — they just lived only in task files, and for the six
completed, in `.tasks/completed/` where they are least likely to be read again.

Each artifact declares itself a retrospective consolidation with its date and
source. Where a task file had empty Problem Statement and Assumptions sections
(T-1793, T-1830) the artifact says so rather than filling them in. No
`fw inception decide` was run and no Human AC ticked on the four `owner: human`
tasks.

### A4 / A5 — resolved

Uncommitted churn is committed. The 26 bypasses are **reviewed and clean**:
exactly 24 `FW_SWITCH_FOCUS` (framework's own handover flow) + 2
`FW_ALLOW_EMPTY_RECOMMENDATION`. No `--force`, `--no-verify`, `--skip-sovereignty`
or `FW_ALLOW_HUMAN_AC_TICK` in the 7-day window. It ages out unaided.

### Two findings the warnings did not show

**T-2717 / U-005 — inception decisions can contradict their own rationale.**
T-1793 records `Decision: GO` carrying the DEFER rationale verbatim; T-2288
records `Decision: GO` carrying the NO-GO rationale, including *"a
termlink-driven build now would violate the gate."* The four consistent cases all
had a GO recommendation already, so the divergence lands exactly where the
recommendation was **not** GO — two instances five weeks apart. `--rationale` is
mandatory (`lib/inception.sh:421`), so this is not tool auto-fill. Whether each was
a deliberate override or a mis-typed verdict is **unrecoverable**, which is the
defect: nothing detects a verdict contradicting the reasoning beside it. No
decision field was altered — that is a human-sovereignty record.

**T-2718 / U-006 — the fabric no-edges mitigation is inert.** `fw fabric enrich`
reports 0 enriched, 0 edges, **and 0 unresolved targets** — saturated, not
blocked. Across 149 files in `scripts/`: **0** source another repo script via the
standard idiom; **71** reference the `termlink` binary. The edge model is
file→file and cannot name a compiled binary, so no parser change closes this.

---

## The 10 remaining warnings, and why each stands

| # | Warning | Owner | Why it stands |
|---|---|---|---|
| 1 | Arc `arc-substrate-fitness` stale | **human** | T-2250 is its only open task; both offered mitigations are illegitimate (see D1) |
| 2 | Fabric 193/344 no edges | filed **T-2718** | mitigation inert; largely structural, not backlog |
| 3 | Fabric 4 cards uncovered | settled **T-2712** | deliberate; the 4 are real components but not source |
| 4 | Uncommitted changes | transient | session churn; clears on the next commit |
| 5 | Gate-bypass 26 in 7 days | reviewed | all benign; rolling window ages out |
| 6 | No commit-msg hook | filed **T-2714** | vendored; unfixable in a worktree |
| 7 | Learnings ready for promotion | **human** | curation decision (D2) |
| 8 | C-002 research-artifact check | filed **T-2714** | same root cause as #6 |
| 9 | CTL-020 cron audit dir | filed **T-2715** | vendored; unfixable in a worktree |
| 10 | CTL-011 pre-push hook | filed **T-2714** | same root cause as #6 |

Six of the ten (#2, #6, #8, #9, #10, and #3's rationale) are **framework defects
in vendored files**. A local edit is erased on re-vendor, so each is recorded
under `.context/upstream/` instead.

---

## Why zero is not reachable

Reaching 0 WARN from here requires either **lying** (#2, #3, #4, #5 — fabricate
edges, narrow the watch patterns, `mkdir` a directory to fake host state) or
**overstepping** (#1, #7 — close a human-owned inception, self-serve a promotion
decision). Avoiding exactly those two failure modes is what the remaining warnings
are for.

**If a future pass reports 0 WARN, check what it narrowed or faked to get there.**
This repo has hit the narrow-the-metric trap three times already — T-2680, T-2681,
T-2712 — recorded as PL-341.

---

## Next actions

### Ready to run — outward filing (needs operator authorisation)

Six upstream records are written and validated under `.context/upstream/`:

| | Task | Sev | Summary |
|---|---|---|---|
| U-001 | T-2711 | med | `revisit-due-scan.sh` exits 0 after scanning nothing |
| U-002 | T-2713 | med | hook telemetry counts exit-2 blocks as failures |
| U-003 | T-2714 | **high** | audit hook checks concatenate `.git/hooks` |
| U-004 | T-2715 | med | CTL-020 worktree-blind; its mitigation is harmful |
| U-005 | T-2717 | **high** | inception decision can contradict its own rationale |
| U-006 | T-2718 | med | fabric no-edges mitigation is inert |

This is Path 2 of `docs/guides/upstream-reporting.md` — the persistent record the
guide prescribes *before* delivery. **The outward post to the shared
`framework:pickup` topic was deliberately not made**: it is visible to peer
projects, so it is the operator's call. The `Filed to framework:pickup` AC on each
of the six tasks is correspondingly unticked and all six stay open.

**U-003 and U-004 share one root shape and one fix** — a fixture that runs the
audit inside a `git worktree add`. Each record says so, so they are not fixed twice
or half-fixed.

### Human decisions (D)

- **T-1898** — revisit fired 2026-07-06, now **39 days** overdue. Evidence needed:
  operator authorises the 5h-agent + 24h-observation spike, OR ring20-management
  goes silent >24h again.
- **T-2250** — revisit fired 2026-07-25, **20 days** overdue. Evidence needed: R7
  hygiene cleanup landed + R4 daily-aggregated-push validated live. **This is what
  keeps warning #1 alive.**
- **Five promotion candidates**: PL-213 (23 applications), PL-209 (18), PL-168
  (17), PL-206 (17), PL-172 (13).

Nothing calls `fw task revisit-due` automatically because T-1452 (cron + handover
banner) is still `started-work`, `owner: human` — the field is written and the verb
reads it correctly, but no one runs it.

---

## Operational note

Pass 2 ran to completion after `/compact` reset the budget gate, and hit the wall
again at ~289K while wrapping up. The gate counts cumulative session transcript
tokens, so it returns to critical roughly every pass. To run hands-off:

```
cd /opt/termlink/.claude/worktrees/charter-review-2026-0814 && claude-fw
```

`/compact` also resets it for one more pass, but without `claude-fw` the same wall
returns.
