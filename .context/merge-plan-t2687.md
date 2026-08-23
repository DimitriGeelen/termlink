# Merge plan — `worktree-t2687-pickup-failopen` → `main`

> **2026-08-23 — SUPERSEDED BY A REAL BRANCH. This is no longer a plan.**
>
> The merge described below has been **performed** on `integration/t2687-trial`, pushed to
> OneDev at `a25afc4b6`. All 39 conflicts are resolved, the tree builds, and every suite
> passes. What follows is kept as the record of how each class was decided.
>
> **To land it:**
>
> ```
> https://onedev.docker.ring20.geelenandcompany.com/termlink/~pulls/new?target=30:main&source=30:integration/t2687-trial
> ```
>
> Merging that PR brings the branch's 91 commits onto main with the conflicts already
> settled. `main` has not been touched.

## Proof on the merged tree

| check | result |
|---|---|
| `cargo build --release` | clean |
| `cargo test --release` | **2,944 passed, 0 failed** (10 suites) |
| `tests/*.sh` fixture suites | **60 passed, 0 failed** |
| register union (new check) | 4 registers, **no entry lost from either parent** |
| conflict markers in tree | 0 |
| `tools.rs` vs `origin/main` | byte-identical |

## Divergence at merge time

| | |
|---|---|
| branch | `worktree-t2687-pickup-failopen` @ `4d577b308` |
| merge base | `19ba70a33` (2026-08-13) |
| main | `447b8b638` — 230 commits ahead of base |
| commits landing | **91** |
| conflicts | **39** — all resolved |
| merge commit | `a25afc4b6` on `integration/t2687-trial` |

## How each class was resolved

| class | n | resolution |
|---|---|---|
| session churn | 14 | ours; `VERSION` took main's higher stamp |
| append-only logs | 4 | **union** — see below |
| episodic records | 4 | ours (hand-enriched vs generator one-liners) |
| fabric cards | 4 | ours **plus** main's topology edges grafted in |
| vendored `.agentic-framework/**` | 9 | main (6 strict supersets; 3 keep our logic and add features) |
| `tools.rs` | 1 | main — T-2687 ≡ our T-2824, ours is the duplicate |
| `CLAUDE.md` | 1 | **union** — disjoint sections, zero heading overlap |
| canary scripts + task file | 2 | merged by hand, both designs kept |

**Nothing was resolved by picking a side where both sides held real content.** The two
places that would have looked like clean side-picks were the ones that cost most:

- **Fabric cards.** A plain `--ours` looks right (ours are the enriched cards; main's are
  `purpose: 'TODO'` stubs) and would have silently dropped **6 real `depended_by` topology
  edges** that only main's copies carried. Kept ours, grafted the edges.
- **`.gate-bypass-log.yaml` / registers.** These are append-only logs. Picking either side
  compiles, tests green, audit passes, and deletes history — nothing else in the tree would
  notice. Hence `scripts/verify-register-union.sh`, which asserts every id from **both**
  parents survives. It goes **red before the merge and green after**, so it is a check that
  can actually fail.

## Two register ID collisions — the T-2800 class, outside `.tasks/`

Both branches allocated the same ids to **different records**:

| id | ours | main's |
|---|---|---|
| `PL-328` | revisit-due-scan silent-nothing (T-2810) | a guard's green result is not evidence (T-2678) |
| `PL-329` | duplicate-work detector blind to fixes (T-2827) | — |
| `PD-094`–`PD-103` | T-2805-era decisions | T-2746-era decisions |

Main's numbering is published, so **ours moved**: `PL-358/359`, `PD-139`–`PD-148`. The one
prose reference (`T-2197`, "learning PL-328") was repointed. This is exactly the concurrent
allocation race T-2800 documents for task ids — it applies to every counter in
`.context/project/`, not just `.tasks/`, and nothing checks those.

## The one genuine policy collision — `check-cron-install-drift.sh`

Both branches rewrote the same check to fix the same complaint, incompatibly:

- **Ours (T-2821):** all DRIFT fires; an allowlist acknowledges deliberate host variations.
- **Main (T-2682):** drift split by *direction* — a git-declared job line absent from the
  host becomes its own always-firing `UNINSTALLED_JOBS` class; cosmetic drift stays a
  warning behind `--strict`.

**Main's default won**, because its discriminator is sharper: it fires on the difference
that actually means "shipped but dark" rather than on any byte change. T-2821's mechanism
was **not** discarded — the allowlist and `--lenient` are grafted onto main's
implementation.

Keeping our fixture suite paid for itself immediately: it caught main's script summarising
a tree with known drift as **"healthy"** — the precise T-2815 wording defect main's own
header claims to build on. Not firing is a policy choice; calling it healthy is a false
statement. Fixed. Both suites now pass (13 ours + 24 main's).

## Still needs your judgement

1. **The drift direction neither policy covers.** T-2821's motivating evidence was *21 of
   24 host crontabs carrying a real fix that had never been committed to git*. That is the
   **opposite** direction from `UNINSTALLED_JOBS`, and main deliberately treats an extra
   host-local job as the operator's prerogative. So **neither** policy fires on it, and the
   21-crontab backlog that started all this is still invisible. Worth its own class.

2. **Episodic records embed an ephemeral worktree path.** Every conflicting episodic carried
   `source_file: /opt/termlink/.claude/worktrees/<name>/...`. Both sides were wrong; the
   path names a directory that gets deleted. Cosmetic today, misleading forever.

3. **`.fabric` cards are generated with `created_by: unknown` and `purpose: 'TODO'`** and
   only become useful when someone enriches them by hand. The stub/enriched split is what
   made these conflict at all.

---

*Everything below this line is the original 2026-08-20/22 plan, kept as the decision record.*

# Merge plan — `worktree-t2687-pickup-failopen` → `main`

**Prepared 2026-08-20 by trial merge in a scratch worktree.** Nothing here was applied to
`main` or to the branch. Every resolution below was actually applied in the scratch tree, and
the result **builds (`cargo build` clean) and passes 37/37 fixture suites**.

**What was re-measured on 2026-08-22 and what was carried forward.** Re-measured against
the current heads: the conflict set (38, listed below), both sides' content for the new
conflict, and main's position. **Carried forward from the 2026-08-20 trial: the build and
the 37/37 fixture run.** That carry-forward is sound rather than lazy —
`crates/termlink-mcp/src/tools.rs` is the *only* Rust file touched by the nine commits
added since, and it resolves to main's copy, so the tree that compiles is byte-identical to
the one that was built and tested. The fixture count will read 38 after the merge, not 37,
because this branch adds `tests/canary-status-firing-fixtures.sh`; it is a new file on one
side and does not conflict.

| | |
|---|---|
| branch | `worktree-t2687-pickup-failopen` @ `fdb70a144` |
| merge base | `19ba70a33` (2026-08-13) |
| main | `447b8b638` — 230 commits ahead of base, **unchanged since the 2026-08-20 trial** |
| commits landing | 87 |
| files changed | 395 |
| conflicts | **38** |
| needs your judgement | **1** (see §Flagged) |

**Refreshed 2026-08-22.** The trial was run at `c8607a501` / 78 commits / 37 conflicts;
nine commits have landed on this branch since, so those figures are superseded and the
numbers above were re-measured with `git merge-tree --write-tree` against the current
heads. Main has **not** moved (still exactly 230 ahead of the same base), so every
resolution below still applies to the same content on main's side — only the branch grew.

Exactly one conflict is new (`crates/termlink-mcp/src/tools.rs`) and none of the previous
37 went away. Its resolution is in §Resolution table under *take MAIN — the duplicate fix*.

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

### take MAIN — the duplicate fix (1 file)

```
crates/termlink-mcp/src/tools.rs
```

**This is the one conflict that is new since the trial, and it is not a merge inconvenience
— it is duplicated work.** Both branches fixed the same defect in `termlink_topics`: a
chained `if let` that swallowed a timeout, a transport error, an error response and a
missing `topics` array identically, so a caller received a partial topic inventory with no
way to know it was partial. Main fixed it as **T-2687**; this branch fixed it again as
**T-2824**, eight days later.

I compared the two rather than assuming the larger diff was better. They are **functionally
identical** — same four JSON keys (`sessions_unreachable`, `sessions_bad_result`,
`sessions_skipped`, `sessions_probed`), same semantics, same three-way probe
classification. The difference is structural: main extracts a pure
`aggregate_topics_probes_mcp` helper that is unit-testable and mirrors the CLI's
`aggregate_topics_probes`; ours classifies inline in the handler. **Main's is the better
shape, so main wins.**

Ours is not additive in any respect I could find, so nothing is lost by dropping it.

**One thing to re-check after merging.** T-2824 closed with two ACs deliberately left
unchecked, because `parity_topics` still failed on a reachability delta it said "cannot be
fixed from the MCP side alone" — specifically the empty-registrations early return. Main's
version *does* populate that early return with explicit zeros. So main's implementation may
already satisfy what T-2824 could not. Run `cargo test -p termlink-mcp --test parity` after
the merge: if it is 24/24, T-2824's two open ACs are satisfied by main's code and the task
can be closed honestly instead of carrying permanent unchecked boxes.

**Why nothing caught this.** `check-task-id-collisions.sh` axis C exists precisely to catch
duplicated work, and it did not see this one. Two reasons, both structural:

- Axis C runs `git diff --diff-filter=A`, so it only considers files **added** on a branch.
  `tools.rs` existed at the merge base and both sides *modified* it — that class is never
  examined. It catches duplicated new files; it cannot catch duplicated **fixes to existing
  files**.
- The branch list is `[b for b in git branch ... if b != BASE]`, so **main is excluded**.
  Duplication between a feature branch and main is invisible — and main is where the other
  230 commits live.

The cost here was one task's effort, which is small. But this is the second instance of the
same class in this repo (the first being the two `check-verification-pipefail.sh`
implementations in §Flagged), and that one *was* caught only because it happened to be a new
file on two feature branches. Captured as a task rather than fixed here — the fix is a new
axis, not a tweak, and it needs a false-positive story before it can fire on "two branches
touched the same file".

Practical mitigation until then: before implementing a fix on a long-lived branch, run
`git log origin/main -- <path>` on the file you are about to change.

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

The scripts that produced this are committed at **`scripts/merge-t2687/`** — see its README
for the order and for the two worth keeping past this merge. They were written in a scratch job
directory that is deleted with the session, and a plan pointing at files that will not exist is
the same failure this branch spent the day documenting.

They hardcode a scratch path; point it at your own:

```bash
git worktree add --detach /tmp/merge-trial HEAD
```

A fully-resolved tree also still exists at `/root/.claude/jobs/d638a35c/tmp/merge-trial`,
uncommitted — **ephemeral**, so treat it as a convenience rather than the record.
