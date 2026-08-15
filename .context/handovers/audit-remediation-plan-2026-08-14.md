# Audit remediation — status after pass 3 (2026-08-15)

**Trajectory: 30 WARN (pre-pass-1) → 20 → 10 → 6.** 0 FAIL throughout.

Pass 3 rejected pass 2's conclusion that 10 was "the honest floor". Four of those
ten were **false findings** — the checks were structurally unable to pass in a
linked worktree — and clearing a false warning by fixing its detector is not
cosmetic, it is the only correct response.

Re-run `.agentic-framework/bin/fw audit` before acting on anything below. Note it
takes **>15 minutes**: CTL-013 re-runs the verification suites of the three most
recently completed tasks, which includes `cargo test`. Budget for that or run
`--section <name>` for a targeted check.

---

## What pass 3 did

### Four false findings fixed at the detector (T-2721)

| check | why it was false | now |
|---|---|---|
| No commit-msg hook | concatenated `$PROJECT_ROOT/.git/hooks/…`; in a worktree `.git` is a **file**, so that path can never exist | PASS |
| C-002 research gate | same concatenation — asserted a **live safety gate was missing** while it was installed and enforcing | PASS |
| CTL-011 pre-push | same concatenation | PASS (hook proven executable) |
| CTL-020 cron audits | `.context/audits/cron/` is gitignored, so it cannot exist in a worktree | INFO skip |

Three are fixed by resolving through `git rev-parse --git-path hooks/<name>`, which
handles a normal checkout, a worktree, **and** `core.hooksPath` in one call — so
those checks now genuinely *verify* rather than being suppressed. Only CTL-020 is
skipped, matching the `fw_is_linked_worktree` idiom already used by the cron-drift
and cron-misload checks in the same file.

**This edits vendored code and will be erased on the next re-vendor.** That is
recorded in T-2705 §Updates — the re-vendor task itself — because a warning written
inside `audit.sh` cannot survive the event it warns about. U-003 and U-004 remain
the durable upstream path and are deliberately still open.

**A bug in the fix, caught by verification.** The helper was first defined *inside*
the `enforcement` section block. A full run masked this (enforcement defines it
before the later sections use it), but `--section oe-daily` alone called an
undefined function and resolved to the empty string — producing a false WARN with a
blank Evidence line. Found only by testing each section in isolation. Hoisted to top
level.

### Two corrections to pass 2's own claims

Both were cases of a confident conclusion resting on a measurement narrower than the
claim it supported — the exact defect this whole review has been cataloguing.

**1. "0 of 149 scripts depend on another repo script" (T-2718) was wrong.** That
held only for the three `${SCRIPT_DIR}` / `${REPO_ROOT}` / `${PROJECT_ROOT}` idioms
that were grepped. Re-measured including *invocation*:

```
$ grep -lE '(bash|sh|source|\.) +[^ ]*scripts/[a-z0-9._-]+\.sh' scripts/*.sh | wc -l
18
```

`run-guard-layer.sh` executing every `check-*.sh` is a dependency by any reading. So
"no parser improvement can close this" is false — a parser change closes *part* of
it; the binary-dependency remainder is structural. A measurement returning exactly
zero deserves more suspicion than one returning a few.

**2. "The 26 bypasses are the framework's own handover flow" was wrong.** Verified:
`FW_SWITCH_FOCUS=1` is a mechanism a **caller types** to override the focus-drift
gate. `bin/fw:6639` only *suggests* it in an error message; no framework script
sets it. All 24 are deliberate agent-initiated Tier-2 bypasses. **Warning #5 is a
true signal and should not be dismissed** — 24 focus-gate overrides in 7 days is
exactly the "safety-bypass-as-pattern" the check exists to surface.

### Verified from the peer report (T-2719 / T-2720 / U-007)

Ran the peer's falsifiable prediction before touching code. It passed, but the
source disagreed with half the diagnosis, and the code half was **already fixed and
undeployed**:

- `latest` operand → T-2533 (`ac859d321`, 2026-08-08, **v0.11.871**), a silent
  data-loss fix. Fleet runs **0.11.720**.
- `cursor` operand → a *different* defect: `agent inbox` reads local
  `~/.termlink/cursors.json`, `agent unread` reads the hub-side ack receipt. Two
  stores, never reconciled. **Upgrading alone makes the printed count worse** (a
  correct `latest` of 11867 minus a stale cursor of 1611 = 10256, against a true 30).
- Nothing caught the stale binary because the fleet-binary canary's floor is a
  hand-written constant last touched 2026-07-27. Floors raised to 0.11.871; the
  unchanged script now fires and names all three hubs.

---

## The 6 remaining warnings

| # | Warning | Clearable by an agent? | What actually clears it |
|---|---|---|---|
| 1 | Arc `arc-substrate-fitness` stale | **No — sovereign** | Human resolves the 20-day-overdue T-2250 revisit, or approves drivers via `fw arc approve-driver … --i-am-human` |
| 2 | Fabric 193/344 no edges | **Partly** | A parser change would add ~18 scripts' invocation edges; the rest is structural (binary deps a file→file model cannot express) |
| 3 | Fabric 4 cards uncovered | **No — deliberate** | `.fabric/watch-patterns.yaml` documents this on purpose |
| 4 | Uncommitted changes | Yes, transient | A commit. Note the audit *writes its own report*, so a run dirties the tree it then warns about |
| 5 | Gate-bypass 26 in 7 days | **No — and it is correct** | Rolling 7-day window; ages out only if agents stop overriding the focus gate |
| 6 | Learnings ready for promotion | **No — curation** | Human runs `fw promote PL-XXX --name "…" --directive DN` |

### Why #3 cannot be cleared honestly

The four cards are `.claude/commands/capture.md`, `docs/guides/upstream-reporting.md`,
`install.sh`, `systemd-templates/termlink-substrate-worker@.service`. Two could be
covered cheaply. The other two are markdown — and globbing `docs/**/*.md` would
newly register *hundreds* of uncarded files, trading one warning for a much larger
one. That is the "inventing globs makes the pattern list meaningless" case
`watch-patterns.yaml` already documents.

### Prepared work for #6 (human decides; this is only the proposal)

The check fires on **application count alone**, so it cannot distinguish a reusable
principle from an incident note that happens to be referenced often. Triaged:

**Practice-shaped — worth promoting:**
- `PL-168` (17) — *a detector without a trigger is not prevention*. The strongest of
  the set; it generalizes past its origin incident (G-058). Suggest **D1**.
- `PL-213` (23) — *proof scripts must assert the property they claim, not the
  happy-path outcome*. Suggest **D2**.
- `PL-206` (17) — *fenced code in docs is inert and lapses silently*. Suggest **D2**.

**Not practice-shaped — recommend NOT promoting:**
- `PL-209` (18) — an incident observation carrying an open "Next: investigate…".
  That is a task, not a practice.
- `PL-195` (11) — a specific bug, already filed as T-1874.
- `PL-172` (13) — a narrow MCP-parity recipe; useful, but a technique note.

---

## Why "zero" is still not the target

Reaching 0 from here requires **fabrication** (#3 fake globs, #4 impossible while the
audit dirties its own tree) or **overstepping** (#1 closing a sovereign arc, #6
self-serving a curation decision, #5 silencing a signal that is telling the truth).

The audit itself offers a fabrication as a mitigation: warning #1 suggests
`fw task update T-XXX --last-update $(date -u +%FT%TZ)` — touching a timestamp so
the staleness check stops firing. That was not done.

**If a future pass reports 0 WARN, check what it faked or narrowed to get there.**
This repo has hit the narrow-the-metric trap four times now — T-2680, T-2681, T-2712,
and (twice, in its own analysis) pass 2 — recorded as PL-341.

---

## Open items

### Outward filing — needs operator authorisation

Seven upstream records under `.context/upstream/`, validated:

| | Task | Sev | Summary |
|---|---|---|---|
| U-001 | T-2711 | med | `revisit-due-scan.sh` exits 0 after scanning nothing |
| U-002 | T-2713 | med | hook telemetry counts exit-2 blocks as failures |
| U-003 | T-2714 | **high** | audit hook checks concatenate `.git/hooks` |
| U-004 | T-2715 | med | CTL-020 worktree-blind; its mitigation is harmful |
| U-005 | T-2717 | **high** | inception decision can contradict its own rationale |
| U-006 | T-2718 | med | fabric no-edges mitigation is inert |
| U-007 | T-2720 | **high** | a shipped hub-side rail never moved the deploy floor |

The post to the shared `framework:pickup` topic is **deliberately not made** — it is
visible to peer projects, so it is the operator's call. The `Filed to
framework:pickup` AC on each task stays unticked.

### Operator actions

- **Upgrade the three hubs** to ≥0.11.871 (install binary, restart *through* the
  systemd unit per G-070). Until then the fleet runs a known silent-data-loss bug and
  the canary will name it daily — which is now the correct behaviour.
- **T-1898** revisit — 40 days overdue.
- **T-2250** revisit — 21 days overdue. This is what keeps warning #1 alive.

### Still open for an agent

- **T-2719** — reconcile the two cursor stores. Do *not* treat the binary upgrade as
  the fix.
- Two peer-reported usability items, captured in T-2719 and needing their own tasks:
  `agent recent <topic>` resolving its positional as a session, and no verb mapping a
  fingerprint to a name.
