---
id: T-2698
name: "Blanket .context/working gitignore makes static-check allowlists untrackable"
description: >
  .gitignore:113 blanket-ignores `.context/working/` while 115 files under it are tracked.
  Every file added there since is silently untrackable — the four static-check allowlists
  among them, so alloc-sink and drain-sink both FIRE in any clean clone or worktree despite
  CLAUDE.md documenting both as clean. Narrow the rule to contents-plus-re-include.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, gitignore, clean-clone, static-checks]
components: []
related_tasks: [T-2694, T-2689, T-2692, T-2527, T-2531, T-2666, T-2672, T-2697]
created: 2026-08-20
last_update: 2026-08-20T07:07:33Z
date_finished: null
---

# T-2698: Blanket `.context/working` gitignore makes static-check allowlists untrackable

## Context

`.gitignore:112-113`:

```
# Session-local working state (large vector index, etc.) — never commit
.context/working/
```

The intent is right — a large vector index does not belong in git. The *form* is the bug,
and it is the same one T-2694 just fixed for `.agentic-framework`: a blanket directory rule
under which files are nevertheless tracked. **115 files under `.context/working/` are in
git** right now. Ignore rules do not apply to already-tracked files, so those keep working
and nothing looks broken, while everything added afterwards is silently untrackable —
`git add -A` skips it and `git status` never mentions it, because ignored files are not
reported.

### Confirmed casualties: four static-check allowlists

The alloc-sink (T-2527), drain-sink (T-2531), silent-exit (T-2666) and busy-spin (T-2672)
checks all follow the same convention: fire on a candidate, and let a confirmed-safe site be
acknowledged in `.context/working/.<name>-allowlist` with a cited reason. CLAUDE.md documents
the resulting state as clean:

> The current tree is clean (98 sink calls scanned, 5 confirmed-safe sites allowlisted with
> cited reasons …)

> The current tree is clean: 6 `.output()` sink calls scanned, all 6 allowlisted …

Neither allowlist is in git:

```
$ git check-ignore -v .context/working/.alloc-sink-allowlist
.gitignore:113:.context/working/	.context/working/.alloc-sink-allowlist
```

So in this worktree — and in any clean clone, and on any other machine — both checks fire:

```
$ bash scripts/check-alloc-sink-clamps.sh   → exit 1
$ bash scripts/check-drain-sink-caps.sh     → exit 1
```

Eleven sites that a human already reviewed and consciously acknowledged are presented as
unreviewed findings to the next reader. The acknowledgements — which are governance
decisions, complete with cited reasons — exist only on one host's disk.

### Why this is worth fixing rather than tolerating

Two distinct harms, and the second is the one that matters.

The first is recoverability: the same class as T-2689/T-2692 (`lib/bvp.sh` running but
untracked) and T-2696 (the canary stderr-split deployed but never committed). This is now the
**third** instance found in as many sessions, all sharing one mechanism — a rule that hides
files so well that nobody notices they are being hidden.

The second is what it does to the checks. A reviewer in a clean clone sees eleven findings on
a tree CLAUDE.md calls clean. They either re-review all eleven (paying the cost twice), or
conclude the checks are noisy. That is the erosion T-2693 documented for P-011 and T-2696 for
the canaries: a check that reports known-acknowledged findings teaches people to skim it, and
a check people skim is not a check. The allowlist mechanism exists precisely to prevent that,
and the ignore rule disables the mechanism everywhere but one machine.

It also affects T-2697 immediately: the `.cron-drift-allowlist` shipped there would be born
untrackable.

## Approach

Same shape as T-2694, and deliberately narrower.

`.context/working/` excludes the **directory**, and git cannot re-include a path whose parent
directory is excluded — so `!` negations under the current rule are a silent no-op. The
correct form excludes the directory's *contents* (`/*`), which leaves the directory itself
un-excluded so negations bind, then re-includes exactly the load-bearing class:

```
.context/working/*
!.context/working/.*-allowlist
```

One negation, one pattern, matching every present and future static-check allowlist by
convention rather than by name — so the next check that adopts the pattern is trackable
without another gitignore edit.

**Everything else stays ignored.** The vector index, session state, counters and logs the rule
exists to keep out remain out. And the 115 already-tracked files are unaffected either way,
since ignore rules never applied to them.

## Scope boundary

This changes the **rule**. It does not add the missing allowlist files: they live in the main
checkout and T-559 correctly stops a worktree session from reaching them. Narrowing the rule
is the structural half — it makes them visible to `git status` and addable by a plain
`git add`. The one-time catch-up is a Human AC so the operator reads the contents before
committing; an allowlist is a record of governance decisions and deserves that look.

## Acceptance Criteria

### Agent
- [x] The blanket `.context/working/` rule is replaced with a contents-plus-re-include form
- [x] `.context/working/.*-allowlist` is re-included by pattern, not by enumerating names
- [x] Session-local state the rule exists to exclude (vector index, session.yaml, counters,
      logs) remains ignored
- [x] No currently-tracked path under `.context/working/` becomes ignored — `git status`
      stays clean in this checkout
- [x] Fixture proves an allowlist path is addable by a plain `git add` after the change and
      was NOT before it — the load-bearing property
- [x] Fixture proves a `!` negation under the OLD blanket rule is a silent no-op, so nobody
      "fixes" this that way later
- [x] Fixture proves the ignored-by-design paths are still ignored
- [x] Fixture is host-independent — scratch repo, no real `.context/working`

### Human
- [ ] Commit the four static-check allowlists so the checks are clean off this machine.
      **Steps:**
      1. `cd /opt/termlink && git status --short .context/working/ | head -30`
      2. Read the list — confirm nothing machine-local or secret-bearing appears. Expect the
         `.*-allowlist` files and nothing else newly visible.
      3. `cd /opt/termlink && cat .context/working/.alloc-sink-allowlist .context/working/.drain-sink-allowlist`
         — these are governance records; check the cited reasons still read as correct.
      4. `cd /opt/termlink && git add .context/working/.alloc-sink-allowlist .context/working/.drain-sink-allowlist .context/working/.silent-exit-allowlist .context/working/.busy-spin-allowlist`
         (skip any that do not exist — silent-exit and busy-spin are documented as having
         zero entries, so they may legitimately be absent)
      5. `cd /opt/termlink && git commit -m "T-2698: track the static-check allowlists the blanket ignore rule hid"`
      6. `cd /opt/termlink && bash scripts/check-alloc-sink-clamps.sh && bash scripts/check-drain-sink-caps.sh`
      **Expected:** step 6 exits 0 for both — and, more to the point, they will now exit 0 in
      a fresh clone too.
      **If not:** the check names the site still unacknowledged; either it is a genuine new
      finding worth clamping, or it needs adding to the allowlist with a reason.

## Verification

bash tests/gitignore-working-scope-fixtures.sh
test -z "$(git status --porcelain .context/working/ | grep -v '^ M')"

## RCA

**Symptom:** `check-alloc-sink-clamps.sh` and `check-drain-sink-caps.sh` both exit 1 in a
fresh worktree, reporting 11 findings on a tree CLAUDE.md documents as clean.

**Root cause:** `.gitignore:113` ignores the whole `.context/working/` directory. The four
static-check allowlists live there, were never force-added, and are therefore invisible to
git — so the acknowledgements they carry exist only in the checkout that made them.

**Why structurally allowed:** the rule was correct when written (session-local scratch, keep
the vector index out) and nothing re-examined it when `.context/working/` later became the
home for load-bearing governance records. A blanket directory ignore is specifically
self-concealing: the files it hides never appear in `git status`, so the divergence between
what runs and what is recoverable produces no signal at all. The same mechanism produced
T-2689 (`lib/bvp.sh`), T-2692 (dangling refs) and T-2696 (the uncommitted canary fix) — three
prior instances, one mechanism.

**Prevention:** the re-include is by pattern (`.*-allowlist`), so future checks adopting the
convention are trackable without another edit. The fixture pins that a plain `git add` picks
up an allowlist path, and separately pins that the naive `!`-negation fix is a no-op — the
trap most likely to be walked into by whoever touches this next.

**Not prevented:** other load-bearing file classes that may later land in `.context/working/`
and are not allowlists remain untrackable by default. That is deliberate — the rule's purpose
is to keep scratch state out, and widening it further would trade this defect for the
opposite one.

## Decisions

### 2026-08-20 — This is the mechanism half; another branch already did the files

- **Context:** `worktree-governance-canary-signal` found the same defect independently and
  closed it as their T-2692 on 2026-08-18, measuring it more carefully than I did — 15 false
  positives across four checks, where I counted 11 across two. They fixed it by getting the
  four allowlist files tracked. Their `.gitignore` still carries the blanket
  `.context/working/` rule, so the files were force-added.
- **Chose:** Keep this task, narrowed to the mechanism only.
- **Why:** Force-adding the four files fixes today and leaves tomorrow broken — the fifth
  allowlist (T-2697's `.cron-drift-allowlist`, written this session) would be untrackable
  again, and nobody would notice, because that is precisely what a blanket ignore does. The
  two fixes compose: theirs recovers the existing acknowledgements, this one stops the next
  one going missing.
- **Superseded from this branch:** my duplicate implementations of their T-2690 and T-2692
  (canary-status TOOLING state, `.context/cron/ondemand-checks.conf` registry) were removed
  rather than merged. Theirs is better on both — they fixed the heartbeat-touched-
  unconditionally problem across ~22 canary scripts, which I had noticed and not fixed, and
  they derive "is this check cron-scheduled?" from the crontabs themselves rather than from
  my hand-maintained conf, which was itself a drift source.

### 2026-08-20 — Re-include by pattern, not by filename

- **Chose:** `!.context/working/.*-allowlist`.
- **Why:** The casualty is a *class* — every check following the four-sibling allowlist
  convention, including T-2697's new one and any future check. Enumerating today's four
  filenames would make the fifth silently untrackable again, reproducing the exact defect one
  file later.
- **Rejected:** Listing the four names explicitly. More precise about today, wrong about
  tomorrow.
- **Rejected:** Un-ignoring `.context/working/` entirely. The rule has a real job; the vector
  index and session state genuinely should not be committed.
