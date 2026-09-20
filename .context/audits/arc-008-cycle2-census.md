# arc-008 cycle 2 — complete finding census

Captured: 2026-09-20T10:30Z  |  VERSION 0.11.1973  |  commit 76f5f18c6  |  task T-3014
Cycle 1 for comparison: `.context/audits/arc-008-cycle1-census.md` (2026-09-09, commit 2ea686b22)

Verbs run: `fw audit` (full, all sections) and `fw doctor`.

```
                 cycle 1 (09-09)      cycle 2 (09-20)      delta
fw audit  pass          368                  386           +18
fw audit  warn           77                   94           +17
fw audit  fail            5                    4            -1
fw doctor warn           14                    1           -13
fw doctor fail            0                    0            0
```

This file is the durable record, on the same terms as cycle 1: every raw FAIL line below is
reproduced verbatim, whether or not it has become a task. Nothing here may be dropped without
an explicit decision.

**Scope note (T-2680 discipline).** This census records what the two verbs emitted on
2026-09-20 and how that compares to cycle 1. It does **not** assert that any finding is
remediated — arc-008's rule is that the re-run *is* the verification, and for two findings
below the re-run says the opposite of what their task files say.

## Audit FAIL (4)

- [FAIL] cron(substrate-smoke-canary): USER-field syntax but no install in /etc/cron.d
- [FAIL] D2: Human review queue — 58 task(s) waiting >30d: T-1417(143d) T-1419(143d) T-1435(142d) T-1442(141d) T-1482(138d) … T-2873(18d) T-2878(18d)
- [FAIL] D8: Handover quality — LATEST.md has 5 [TODO] sections
- [FAIL] D8b: Handover archive rot — 10/10 recent handovers have unfilled [TODO]s

## Doctor FAIL (0) / Doctor WARN (1)

- [WARN] Repo root: 1 untracked binary file(s) — 327MB `voxtype_0.7.5-1_amd64.deb`

Doctor went 14 warn → 1 warn between cycles. The single survivor is a stray 327MB Debian
package sitting untracked in the repo root. It is untracked, so the T-1845 pre-commit
large-file gate (10 MiB BLOCK) never sees it and it cannot enter history by accident — but
it is exactly the shape G-058 was about, and doctor is right to name it.

## Cycle-1 to cycle-2 diff

Each cycle-1 FAIL, with the verdict the re-run supports:

### 1. Cron drift: agentic-audit.crontab differs from deployed copy — **CLEARED**
Task T-2938. `diff -q .context/cron/agentic-audit.crontab /etc/cron.d/agentic-audit-termlink`
exits 0 and the line is absent from the cycle-2 audit. This is the one cycle-1 FAIL that is
genuinely gone. Note the resolving event was T-2870's registry regeneration plus a host
install, not T-2938 itself — T-2938 measured and verified, it did not remediate.

### 2. cron(substrate-smoke-canary) not installed — **PERSISTS**
Task T-2939, parked to human review with a GO verdict. `/etc/cron.d/termlink-substrate-smoke-canary`
is still absent. The remediation is a single `sudo cp` the human owns; the agent verified the
source, the self-declared install path and that the job itself exits 0 healthy. Persisting
here is the sovereignty boundary working as designed, not a stalled fix.

### 3. D2: human review queue >30d — **PERSISTS, WORSENED (57 → 58)**
Task T-2940 (parked, GO) and T-2194 (open since 2026-08-20). The oldest entry is now
T-1417 at 143 days. See the D2 list defect below — the printed list is still unfiltered.

### 4. D8: Handover quality — **PERSISTS, unchanged at 5 [TODO] sections**
Task T-2941, closed `work-completed` 2026-09-09. Eleven days later the audit line is
identical. See "Arc rule violation" below.

### 5. D8b: Handover archive rot 10/10 — **PERSISTS, unchanged**
Task T-2942, closed `work-completed` 2026-09-09. Same shape as D8.

### WARN delta (77 → 94): +18 new, −1 cleared
- **+14** `C-001`/`C-006` inception warnings — research artifact missing / template-only
  Recommendation block. Almost entirely tasks created *after* cycle 1 (T-2898…T-2945,
  T-3003…T-3012). The warning class is not degrading; the corpus it scans grew.
- **+3** `CTL-029` "all Agent ACs ticked but status='started-work'" — now 30 tasks.
- **+1** `CTL-013b` review-queue verification re-run: 1 of 3 red; `CTL-003` budget file
  stale; `Fabric drift` 1 source file with no card.
- **−1** `Gate-bypass log: N safety bypasses in last 7 days` — cleared.

## Arc rule violation — D8 and D8b were closed against self-assertion

arc-008's stated rule: *"Verification of a task is the re-run of the audit in the next cycle,
not self-assertion."* Cycle 2 is that re-run, and it contradicts two closed tasks:

- **T-2941** ("D8: handover LATEST.md ships with 5 unfilled [TODO] sections") — closed
  `work-completed` 2026-09-09. Audit still reports **5** [TODO] sections.
- **T-2942** ("D8b: 10 of 10 recent handovers carry unfilled TODO sections") — closed
  `work-completed` 2026-09-09. Audit still reports **10/10**.

The distinction that matters, because it changes the remedy: both tasks fixed an **instance**
(they filled the handover that existed that day) while the **mechanism** that produces
unfilled handovers was untouched. `handover.sh` emits five `[TODO]` sections and the
PreCompact hook auto-generates and auto-commits a handover with nobody obliged to fill them.
Every compaction since has minted a fresh violation; `fw doctor` emitted
`HANDOVER STALE: Last handover has 5 unfilled [TODO] sections (61min old)` during this very
session, unprompted.

This is **not** T-2943's floor defect. T-2943 established that D8's `grep -c` counts the
generator's own instructional comment, so D8's floor is 1 and its PASS is unreachable — and
T-2943 is scrupulous about saying its own green does not mean D8 passes. But D8 **FAILs** at
threshold `>3`, and the current count is 5. Four of those five are genuinely unfilled
sections, not the floor artifact.

Per the arc rule — findings sharing a root cause are **linked, never merged**, and T-2941/2942
are already the governed tasks for their individual FAIL lines — the mechanism is filed as its
own task rather than by reopening either: **T-3015**.

## Second finding — CTL-029 advises a close the sovereignty gate refuses

CTL-029 emits, for 30 tasks including T-2938, T-2939 and T-2940:

```
[WARN] CTL-029: T-2940 has all Agent ACs ticked but status='started-work' — completable, not closed
```

Measured this session, not inferred: `fw task update T-2940 --status work-completed` returns

```
ERROR: Cannot complete human-owned task
Sovereignty gate (R-033): owner is human.
```

These tasks are `owner: human`, so T-193 partial-complete never applies and an agent can
never close them. Their correct terminal state is *parked to review* — agent AC ticked,
Recommendation filled, review marker and `/review/<id>` URL emitted — which is exactly the
state CTL-029 is flagging as a problem. One framework surface is recommending an action
another framework surface structurally forbids.

The cost is not cosmetic: CTL-029 is 30 of the 94 warnings, and a warning class that cannot
be acted on is the attention-exhaustion shape T-2818 and T-2556 both document. Recorded as
learning **PL-376**. Filed as **T-3016**.

## Third finding — the D2 list still prints 10 sub-threshold IDs

The D2 header count is correctly filtered (`58 task(s) waiting >30d`) but the printed list
carries 68 IDs, including ten under the threshold: T-2409(27d) T-2706(23d) T-2709(24d)
T-2711(24d) T-2822(27d) T-2836(27d) T-2839(24d) T-2861(20d) T-2873(18d) T-2878(18d). A
reader trusting the list over the header over-counts the backlog by 10. Unchanged since
cycle 1, where it was recorded in T-2940's Updates but never filed as its own finding.
The check is vendored (G-062), so the fix is upstream, not local. Filed as **T-3017**.

## FAIL to task mapping

Every cycle-2 FAIL is attributed. No FAIL is unmapped.

| # | FAIL line | Task | Task state | Notes |
|---|---|---|---|---|
| 1 | cron(substrate-smoke-canary) not installed | **T-2939** | parked to review, GO | awaits human `sudo cp`; R-033 blocks agent close |
| 2 | D2 review queue 58 >30d | **T-2940** (+ T-2194) | parked to review, GO | list defect filed separately as T-3017 |
| 3 | D8 handover quality 5 [TODO] | **T-2941** (closed) → mechanism **T-3015** | instance closed 09-09; mechanism open | linked, not merged (arc rule) |
| 4 | D8b handover archive rot 10/10 | **T-2942** (closed) → mechanism **T-3015** | instance closed 09-09; mechanism open | shares root cause with #3 |

Doctor's single WARN (327MB untracked .deb in repo root) is **not** filed: it is a WARN, not
a FAIL, and the remedy is an operator judgement about a file that cannot enter git history
while it stays untracked. Recorded here so the next cycle can see whether it persisted.

## What cycle 3 should check first

1. Did T-2939's install land — does the cron FAIL clear?
2. Did T-3015 change the handover generator such that D8b's count moves off 10/10? That is
   the single highest-signal number in this census, because it is the one FAIL whose task was
   already marked done once.
3. Is CTL-029 still 30, and does it still name human-owned tasks?
