# T-3003 — GO-propagation leak investigation (77 GO inceptions unpropagated)

**Task:** T-3003 (inception, arc-009, S-29a/C-35)
**Source finding:** `docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md` C-35 — 77 of 158
GO-recorded inceptions have empty `related_tasks` and no back-reference from any build task.
**Measurement script:** run 2026-09-20 over `.tasks/{active,completed}` (2,447 task files scanned,
236 inceptions). Reproducible; script archived in the run's job tmp, logic documented below.

## Question

When a human records GO on an inception, the framework's contract is that build work follows and is
traceable back to the inception (CLAUDE.md §Inception Discipline step 5). C-35 claims that for
roughly half of all GO inceptions no such propagation is discoverable. Is that accurate, what is the
leak mechanism, and what structural fix closes it?

## Method

1. Enumerate task files with `workflow_type: inception` and a recorded GO (`**Decision**: GO` block
   or `- **Decision:** GO` Updates entry).
2. **Strict predicate** (C-35's): forward link absent (`related_tasks` empty) AND no back-reference
   via any other task's `related_tasks`.
3. **Loose predicate** (new): additionally, no other task file mentions the inception's ID anywhere
   in its body.
4. Read the decide path (`lib/inception.sh`, `update-task.sh`) for any propagation write.

## Findings

**F1 — The C-35 number reproduces in shape: 80 of 167 GO-recorded inceptions are strictly
unlinked** (C-35 said 77/158; the tree gained tasks since the review ran). Distribution by created
month: 2026-03: 23, 2026-04: 25, 2026-05: 3, 2026-06: 6, 2026-07: 10, 2026-08: 13 — the leak is
**ongoing**, not legacy debt.

**F2 — The loose measure collapses to 4.** Only T-954, T-955, T-958, T-1698 have a GO decision and
zero mention by any other task file. For the other 76, follow-on work exists and names the
inception in prose or descriptions — it is the machine-readable link (`related_tasks`) that is
missing, in both directions. **C-35 is therefore primarily a traceability/metadata defect, not 77
approved-but-never-started decisions.** The severity framing in the consolidated report overstates
the work-loss half; the graph-blindness half is real (review queue, BVP components resolution, and
the update-task.sh decomposition gate all read `related_tasks`, not prose).

**F3 — Leak mechanism confirmed (vendored, G-062).** `lib/inception.sh` (1,041 lines) contains no
`related_tasks` write anywhere in the decide path. On GO it prints advisory text only
(`inception.sh:775` — "Next: Create build tasks for implementation"). `fw task create` has no flag
to declare a parent inception. So linkage depends entirely on the agent hand-editing frontmatter
after GO — undocumented as a step, unenforced by any gate. The one structural consumer
(`update-task.sh:917-1033`, decomposition-promise check) fires from the *build* side and only when
the build task already lists the inception — i.e., it can only see links that the missing write
would have created.

**F4 — The 4 genuine orphans:** T-954 (destructive-command auto-prompt pickup), T-955 (inception
decide UX review pass pickup), T-958 (auto-heal dormant — bootstrap_from not wired), T-1698
(partial-migration audit detection pickup). Each is a GO that authorized work nobody filed. T-958
is the operationally live one (auto-heal infrastructure dormant fleet-wide).

## Recommendation (GO — scoped build work)

1. **Local detection (survives re-vendor):** `scripts/check-go-propagation.sh` — deploy-time/ad-hoc
   check, T-2800 tier, firing on GO-recorded inceptions with the STRICT predicate after a grace
   period (default 7 days post-decision). The 80 existing instances go into a git-tracked baseline
   ledger (`.context/checks/go-propagation-allowlist`, T-2483 acknowledged-not-firing pattern) so
   the check is green-at-birth and fires only on NEW leaks — the T-2818 fatigue lesson applied.
   Counted-and-reported, never silently excluded.
2. **Upstream filing (G-062):** decide-time fix belongs in vendored `lib/inception.sh` — either a
   `--follow-on T-XXXX[,...]` flag writing `related_tasks` on both sides, or a loud warning at GO
   when `related_tasks` is empty. File at `framework:pickup` with F3's file:line evidence.
3. **The 4 orphans:** surface to the human (they are GO decisions awaiting filing, a human-priority
   call, not agent-initiated scope). Listed in F4; no retro-linking of the 76 metadata-only cases
   (cost exceeds value; the ledger records them).

**NO-GO condition tested:** none met — the fix is scoped (one script + fixtures + one upstream
filing), testable (fixture suite), reversible.
