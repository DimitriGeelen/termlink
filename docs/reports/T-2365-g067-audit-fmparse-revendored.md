# T-2365 — G-067 audit slowness: fm-parse fork batched (re-vendored T-2297)

**Date:** 2026-07-05
**Gap:** G-067 (pre-push structure audit intermittently kills `git push` — exit 143 / "Terminated")
**Host:** workstation-107

## What G-067 actually was

The pre-push structure audit (`agents/audit/audit.sh`, T-2067 frontmatter-parse
check) fork-exec'd a **separate `python3 -c` per task file** — importing
`web.shared.parse_frontmatter` and parsing one file each. On termlink's corpus
(**2127** task files under `.tasks/active` + `.tasks/completed`) that is 2127
interpreter startups.

Measured before-cost on this host: **200 forks = 13.3s → ~142s** extrapolated
for 2127 files. In a **consumer project** (`web/` absent — termlink has no
`web/shared.py`) each fork immediately hits `ImportError`, exits 0, produces
**zero** useful output — ~142s of pure waste on every push.

## Fix already existed upstream — this was a stale-vendor problem

AEF `origin/master` already carries the fix as **T-2297** (commit `06041f9b`,
"audit: batch T-2067 fm parse — --section structure 6.7min → 132s"): a single
batched `python3` invocation that streams file paths via stdin and emits
`rc<TAB>path` lines, bash aggregates. Per-file rc semantics preserved exactly
(0=ok, 2=False/None, 3=empty-dict yaml.ScannerError).

termlink's **vendored copy** (`.agentic-framework/agents/audit/audit.sh`)
predated T-2297 — it still had the per-file fork. So T-2365 is a **re-vendor**,
not a new AEF fix.

**AEF was NOT mutated.** The AEF checkout was found on a dirty feature branch
(`t2416-fw-safe-mode-hook-timing`, large uncommitted working tree, active
worktrees) — an in-use multi-agent workspace. Rather than disturb it, the fix
was applied to termlink by **surgical block replacement**: the exact T-2067→
T-2297 block was extracted from AEF `origin/master` (`git show
origin/master:agents/audit/audit.sh`, byte-identical, verified durable on both
origin/master and local master) and spliced into termlink's vendored copy,
with a `# T-2365 (G-067):` provenance marker. A future `fw vendor` from master
reinforces (does not clobber) this change.

## Result

| Metric | Before | After |
|---|---|---|
| fm-parse component (2127 files) | ~142s | **0.07s** |
| Full `--section structure` (pre-push path) | ~206s | ~64s |

`bash -n` clean; `fw audit --section structure` runs end-to-end (exit 1 =
pre-existing WARNs only: fabric edges/drift). The dominant push-killing
bottleneck is removed; a 206s→64s audit is far less likely to be killed by
push-time / cron contention.

## Residual finding (NOT G-067 — separate follow-up)

The full structure section retains **~64s** from two bottlenecks unrelated to
the fm-parse fork (surfaced via timestamped audit run):

- **arc-commit-recency check** — ~20s ("in-progress arc(s) had task commits
  within 30 days"; git-log per arc).
- **large-file gate** — ~26s ("Large-file gate: tracked tree clean"; tree scan).

These are distinct optimization targets (git-log fan-out + tree walk), each
deserving its own task per "one bug = one task". They are logged here so the
next optimizer starts from the measured profile rather than re-discovering it.
Whether either is already batched in a newer AEF revision is unverified — a
full audit.sh re-vendor (vs. this surgical fm-parse-only splice) would pull any
such upstream perf work but carries a wider review surface.

## Verification

- `grep -q 'T-2365 (G-067)'` and `grep -q 'T-2297: single batched'` present in
  the vendored copy.
- Batched block forks `python3` exactly once (line: `} | python3 -c "`); the
  other `python3` occurrence is the T-2297 explanatory comment.
- Consumer fast path: single fork → `ImportError` → exit 0 → 0 emitted lines
  (identical to the old per-file behavior, which also recorded zero fails in a
  `web/`-absent project).
