# Merge plan — `worktree-t2687-pickup-failopen` → `main`

**Prepared 2026-08-20 by trial merge in a scratch worktree.** Nothing here was applied to
`main` or to the branch. Every resolution below was actually applied in the scratch tree, and
the result **builds (`cargo build` clean) and passes 37/37 fixture suites**.

| | |
|---|---|
| branch | `worktree-t2687-pickup-failopen` @ `c8607a501` |
| merge base | `19ba70a33` (2026-08-13) |
| commits landing | 78 |
| files changed | 377 |
| conflicts | **37** |
| needs your judgement | **1** (see §Flagged) |

---

## The one thing to decide first

Everything else is mechanical. **§Flagged** is a real fork in the road and it changes which
files survive, so read it before starting.

---

## Resolution table

### take MAIN — vendored framework (9 files)

```
.agentic-framework/agents/context/revisit-due-scan.sh
.agentic-framework/lib/branch-hygiene.sh
.agentic-framework/lib/hook_paths.py
.agentic-framework/lib/ollama_thin_loop.py
.agentic-framework/lib/verification-port.sh
.agentic-framework/lib/version-relation.sh
.agentic-framework/lib/worktree.sh
.agentic-framework/web/blueprints/bvp.py
.agentic-framework/web/templates/bvp.html
```

**Why.** This inverts what I expected. These are files T-2806/T-2807/T-2811 recovered off the
main checkout's *disk* — but main's git already tracks **newer** copies of all nine
(`worktree.sh` 810 lines vs our 497; `revisit-due-scan.sh` 162 vs 93). Main is 230 commits
ahead and someone committed them there after this branch diverged. **Ours are older, not
additive.** Taking ours would be a silent downgrade of nine framework files.

The recovery work is not wasted — it is what made this branch's tooling run — but for the
merge, main wins every one.

**Does this undo T-2810?** No. Main's `revisit-due-scan.sh` still has the marker-collision bug
(line 54: `.framework.yaml` **or** `FRAMEWORK.md`). The mitigation lives in
`.context/cron-registry.yaml`, which does **not** conflict and merges cleanly.

### take MAIN — session scratch + trivia (14 files)

```
.context/working/.budget-status        .context/working/.compact-log
.context/working/.gate-bypass-log.yaml .context/working/.hook-counter
.context/working/.loop-detect.json     .context/working/.pre-compact.last-run
.context/working/.session-metrics.yaml .context/working/.session-start-ts
.context/working/.tool-counter         .context/working/focus.yaml
.context/working/session.yaml          .context/handovers/LATEST.md
.termlink-task                         VERSION
```

Counters, timestamps and per-session state. No content decision in any of them.

### take OURS — episodics (4 files)

```
.context/episodic/T-2025.yaml  .context/episodic/T-2229.yaml
.context/episodic/T-2303.yaml  .context/episodic/T-2677.yaml
```

Ours carry the hand-written summaries from T-2804 (78–83 lines). Main's are the generator's
output with `summary:` blank (73 lines) — the exact "an episodic with no summary is a file, not
a memory" case T-2804 existed to fix.

### take OURS — enriched fabric cards + one task (5 files)

```
.fabric/components/crates-termlink-hub-src-retention_sweeper.yaml
.fabric/components/crates-termlink-session-src-fleet_presence.yaml
.fabric/components/crates-termlink-session-src-identity_dir.yaml
.fabric/components/crates-termlink-session-src-ws_consumer.yaml
.tasks/active/T-1452-revisit-due-scansh-cron--handover-banner.md
```

Main's cards are auto-generated stubs — `purpose: 'TODO: describe what this component does'`,
`type: script`, `tags: []`. Ours are filled in. T-1452 carries this session's updates.

### UNION — `CLAUDE.md`

**Neither side is a superset**, so both take-ours and take-main lose work:

- **main has 7 sections ours lacks:** platform-lock check (T-2693 — *main's* T-2693, a
  different task from our renumbered one), error-code emission check (T-2699),
  version-derivation check (T-2746), MCP/CLI parity census (T-2747), release-artifact drift
  (T-2751), `run-guard-layer.sh` (T-2684), canary log hygiene (T-2685)
- **ours has 8 main lacks:** episodic-store readability (T-2805), task-ID collision (T-2800),
  stranded pickup envelope (T-2801), pipefail auditor (T-2818), framework recoverability
  (T-2814/T-2817), vendor divergence (T-2812), revisit-cron PROJECT_ROOT (T-2810), BVP
  estimator caveat (T-2809)

Resolution applied: start from ours, splice main's 7 in before `## Project-Specific Rules`.
Result: **112 headings, 0 lost from either side** (verified by comparing heading sets both
ways, not by eyeballing size).

### UNION — the three project registers

This is where a naive merge quietly destroys records, and it is worth understanding before you
trust any tooling here.

`learnings.yaml` and `decisions.yaml` allocate IDs by max+1 against the branch's own copy —
**the same defect as the task IDs (T-2800), in registers nothing checks.** Measured:

| register | main | ours | same id, DIFFERENT content |
|---|---|---|---|
| `learnings.yaml` | 371 | 342 | **1** (`PL-328`) |
| `decisions.yaml` | 121 | 154 | **10** (`PD-094`–`PD-103`) |

Deduping on ID — the obvious implementation, and my first one — silently drops **11 records**.
`PL-328` means "a guard's green result is not evidence" on main and "a cron installed and
firing can still be doing nothing" here. Both real, both worth keeping.

Applied resolution: keep both sides; on a same-ID-different-content clash, keep main's ID and
renumber **ours** above the combined ceiling, stamped `renumbered_from`:

```
PL-328 -> PL-358                      (main max PL was 357)
PD-094..PD-103 -> PD-137..PD-146      (combined max PD was 136)
```

Result: learnings **372**, decisions **164**, both parse. `metrics-history.yaml` is a pure time
series with no ID field and main holds the far longer run (6104 lines vs 304) — **take main**.

---

## Flagged — needs your judgement

### `scripts/check-cron-install-drift.sh`

Both branches changed the same checker in **incompatible directions**.

**Main (T-2682)** added `UNINSTALLED_JOBS` as its own firing class — *present, but the installed
crontab is missing job lines git declares* — and demoted plain DRIFT (comment churn, env
tweaks) to a warning that fires only under `--strict`.

**Ours (T-2821)** made DRIFT fire by default, with a `.cron-drift-allowlist` to acknowledge
known-good divergence. The argument recorded at the time: 21 of 24 installed crontabs had
diverged from source, carrying a real fix nobody had committed, and "a warning nobody must act
on is indistinguishable from no warning once it scrolls past."

**My recommendation: take MAIN.** `UNINSTALLED_JOBS` is a more precise version of what ours was
reaching for — it fires on the substantive case (a job line actually missing) rather than on
any byte difference, which avoids the fire-on-cosmetics failure that teaches people to skip a
check. It also needs no allowlist machinery to stay quiet.

**The cost, and it is not free:** ours also removes `tests/cron-drift-firing-fixtures.sh` (its
13 assertions test the allowlist behaviour main's version does not have). Main ships
`tests/cron-install-drift-fixtures.sh`, **24 assertions, passing**. The trial merge was
verified with our suite deleted — that is the configuration that gives 37/37.

If you prefer ours, keep our script **and** our fixtures **and** `.context/working/.cron-drift-allowlist`,
and expect main's 24-assertion suite to need reconciling instead.

---

## Verification of the resolved tree

Everything below was run against the fully-resolved scratch merge, not against the branch:

```
cargo build                          clean
tests/*fixtures.sh                   37 suites, 37 passed, 0 failed
learnings.yaml / decisions.yaml      parse; 372 / 164 records
```

**Known red, and not caused by this merge:** `cargo test --workspace` still fails
`parity_topics`. Pre-existing since 2026-08-12 (T-2624), tracked as **T-2824**. T-2824 landed a
partial fix on this branch — the MCP tool now reports partial inventory instead of silently
swallowing probe failures — but a reachability asymmetry between the two clients remains. The
suite is no redder than main's already is.

---

## Order

**This branch → charter-review → re-vendor.** Not any other way round.

Main still carries the blanket `.gitignore:21`, tracks **0** `policy/` files and 165 `lib/`
(vs 234 here), so `fw bvp` is broken in any clean clone of main until this lands.

### Pre-re-vendor checklist

Also at the top of `.vendor-divergence.yaml`; inlined so you do not have to go and find it.

1. `bash scripts/check-vendor-divergence.sh` — nothing unclassified
2. Confirm the three `filed-upstream` fixes are carried upstream, **or accept losing them**:
   **T-2813** (pickup fail-open), **T-2304** (`update-task.sh` sys.path — already cost this
   project twice), **T-2469** (budget-gate wrap-up deadlock)
3. **`cp CLAUDE.md CLAUDE.md.prevendor`** — `fw upgrade` replaces everything from
   `## Core Principle` to EOF; that is **844 lines** here, including the whole Quick Reference
   table
4. Run the re-vendor
5. `diff CLAUDE.md.prevendor CLAUDE.md` — re-apply what was dropped
6. Re-check the `mode-only` entry (PL-205: a non-executable scanner fails **open**)
7. Update `last_vendor_event:` in `.vendor-divergence.yaml`

### After merging

From `/opt/termlink`, **not from a worktree** (`fw cron install` derives the `/etc/cron.d`
filename from the checkout basename — T-2815):

```bash
cd /opt/termlink && fw cron generate && fw cron install
```

That is what activates the T-2810 `PROJECT_ROOT` fix in the live crontab. Until it runs, the
G-053 revisit scan keeps finding nothing.

---

## Reproducing this

The scratch worktree at `/root/.claude/jobs/d638a35c/tmp/merge-trial` holds the fully-resolved
merge, uncommitted. Scripts that produced it, in order:

```
trial-merge.sh        merge --no-commit origin/main; capture the 37
classify-conflicts.sh how each side differs (sizes + content hashes)
resolve.sh            the mechanical take-ours / take-main calls
union-registers.py    collision-safe register union (--apply)
union-claudemd.py     heading-set union, refuses if anything would be lost (--apply)
verify-trial.sh       fixture suites + YAML sanity on the result
```
