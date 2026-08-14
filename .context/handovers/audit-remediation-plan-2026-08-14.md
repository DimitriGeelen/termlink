# Audit remediation — next-pass plan (2026-08-14)

Written at the end of pass 1. Pass 2 was requested but the budget gate was at
critical (~290K session tokens), which blocks every `fw` and Bash call the loop
needs. This file carries the analysis forward so the next pass executes instead
of re-deriving.

Lives in `.context/handovers/` deliberately: `.context/working/` is gitignored,
so a plan written there would not survive — the same untracked-state trap as
T-2681.

State at handoff: **224 PASS / 20 WARN / 0 FAIL**, down from 30 WARN.
All pass-1 work is committed and pushed (through `0a34485b6`).

**Start by re-running `.agentic-framework/bin/fw audit`** — confirm the list still
matches before acting on it.

---

## A. Believed mechanically fixable — NOT attempted in pass 1

Pass 1 classified these as "can't clean". That was too coarse; each deserves a
check before being written off.

### A1. CTL-020 — "Cron audit directory missing"
Evidence path was `<worktree>/.context/audits/cron` (truncated in the report).
Possibly a plain `mkdir -p`. **Read the CTL-020 site in `audit.sh` first**: the
audit already skips two other cron checks with *"linked worktree — cron is
host-level, managed from the main checkout"*. If CTL-020 belongs to that class it
should be filed alongside T-2714, not conjured into existence with a mkdir that
makes a real gap look closed.

### A2. C-002 — "commit-msg hook missing research artifact check"
Read the **installed** hook via `git rev-parse --git-path hooks/commit-msg`
(never `.git/hooks/...` — see T-2714) and check whether it genuinely lacks the
C-001 research-artifact clause. If it does, `fw git install-hooks` may regenerate
it. If the installed copy is current and the warning persists, it is a detection
bug in the T-2714 family.

### A3. The 10 "inception has no research artifact" warnings
Plain: T-1692, T-1693, T-1793, T-1830, T-2054, T-2288.
C-001: T-2486, T-2546, T-2548, T-2549.

Pass 1 declined all ten as a block, reasoning that writing an artifact after the
fact fabricates a thinking trail. **Right for some, wrong for others — triage per
task.** The audit source scopes C-001 to *"Active inception tasks with
started-work"*, so:

- `captured`, never started → no trail exists to record. Investigate whether the
  status or the check is mis-scoped. **Do not invent content to silence it.**
- `started-work` with real findings already in the task body → a
  `docs/reports/T-XXXX-*.md` consolidating that existing recorded work is
  legitimate; it relocates a trail that exists rather than inventing one. State
  in the artifact that it was consolidated retrospectively, and on what date.
- Several are `owner: human`. Adding a reference line is fine; deciding an
  inception is not.

### A4. "Uncommitted changes present"
Session-state churn under `.context/working/`. The handover normally sweeps it.
One of the 20, and cheap.

### A5. "Gate-bypass log: 26 safety bypasses in last 7 days"
24 of 26 are `FW_SWITCH_FOCUS=1` emitted by the framework's own handover flow
(13 on T-1452 alone); the rest are `FW_ALLOW_EMPTY_RECOMMENDATION` from inception
filing. Rolling 7-day window, so it ages out unaided. Worth one look to confirm no
genuine `--force` / `--no-verify` entry is hiding in the set, then leave it.
Prefer `fw context focus T-XXX` over `FW_SWITCH_FOCUS=1` — the bypass path is what
feeds this warning.

---

## B. Settled — do NOT "fix" these by narrowing a scope

### B1. Fabric: 193/344 cards have no edges
**This is the honest number.** It read 31/150 (18%) only because
`watch-patterns.yaml` excluded the 188 guard-layer scripts that already carried
cards. Widening moved the same metric to 56%; nothing got worse — those files
always had no edges, they simply were not counted. See T-2712 and PL-341.

Reducing it legitimately means giving those scripts real `depends_on` edges.
`fw fabric enrich` is already saturated (0 new edges on re-run), so that needs
better shell-dependency parsing or hand-authored edges. Real work, not
housekeeping. **Narrowing the patterns to make the percentage look better is the
exact trap this repo has now hit three times (T-2680, T-2681, T-2712).**

### B2. Fabric: 4 cards point at files no watch pattern covers
Deliberate, and verified to be exactly: `install.sh`,
`systemd-templates/termlink-substrate-worker@.service`,
`docs/guides/upstream-reporting.md`, `.claude/commands/capture.md`. Real
components, not source. Inventing globs for four singletons would make the
pattern list meaningless. Documented in T-2712.

---

## C. Filed defects — these will NOT clear until upstream fixes them

All three are vendored framework files; a local edit is erased on re-vendor, so
the deliverable is an upstream report on `framework:pickup`, not a patch.

| Task | Warning it causes | One-line |
|---|---|---|
| T-2711 | (none directly) | `revisit-due-scan.sh` exits 0 when it cannot find the tasks dir — reports success after scanning nothing |
| T-2713 | `Hook threshold` (intermittent) | telemetry counts exit-2 blocks as hook failures; every blocking gate is exposed |
| T-2714 | `No commit-msg hook`, `CTL-011` | hook checks concatenate `.git/hooks`, ignoring `core.hooksPath`; structurally unfixable in a linked worktree |

**T-2714 matters for the loop.** `fw git install-hooks` succeeds, the hooks
genuinely fire (proven — the pre-push gate blocked a push, and post-commit ran on
every commit), and both warnings persist regardless. Do not spend a cycle
re-running install-hooks; pass 1 already did, and reported it fixed before
re-checking. It was not.

**T-2713 is intermittent by construction** — it fires when 2/N crosses 0.10 and
clears as N grows. Seeing it PASS is not evidence it is fixed.

---

## D. Requires the human — do not clear unilaterally

### D1. Two overdue revisit decisions (also the cause of the stale-arc warning)
`fw task revisit-due` reports both correctly. Nothing calls it automatically
because T-1452 (cron + handover-banner integration) is still `started-work`,
`owner: human` — the field is written and the verb reads it, but no one runs it.

- **T-1898** fired 2026-07-06 — 39 days overdue.
  Evidence needed: operator authorizes the 5h-agent + 24h-observation spike, OR
  ring20-management goes silent >24h again.
- **T-2250** fired 2026-07-25 — 20 days overdue.
  Evidence needed: R7 hygiene cleanup landed + R4 daily-aggregated-push validated live.

`arc-substrate-fitness` is flagged stale because T-2250 is its only open task.
**Neither mitigation the audit offers is legitimate:** `fw arc close` decides a
human-owned inception; bumping `last_update` fakes activity *and* buries the
overdue date. Leave the WARN until the human rules on T-2250.

### D2. "Learnings ready for promotion"
Five candidates: PL-213 (23 applications), PL-209 (18), PL-168 (17), PL-206 (17),
PL-172 (13). Promotion permanently names a project practice
(`fw promote PL-XXX --name "..." --directive D1`) — a curation decision, and the
warning itself says "review". Surface, do not self-serve.

---

## Expected end state

With A1–A5 resolved, roughly **8–10 warnings** remain: the three filed defects
(C), the two fabric entries that are correct as they stand (B), the stale arc plus
its two overdue revisits (D1), and the promotion review (D2).

**Zero is not reachable without either lying or overstepping**, and avoiding
exactly those two failure modes is what the remaining warnings are for. If a
future pass reports 0 WARN, check what it narrowed or faked to get there.

---

## Operational note — why pass 2 could not run

The budget gate counts cumulative session transcript tokens, so it returned to
critical shortly after the pass-1 `/compact` reset it. The gate itself flags the
cause: *"Unsupervised session (not under claude-fw): the budget auto-restart loop
will NOT fire."*

To run the loop hands-off:

```
cd /opt/termlink/.claude/worktrees/charter-review-2026-0814 && claude-fw
```

`/compact` also resets it for one more pass, but without `claude-fw` the same wall
returns at ~290K.
