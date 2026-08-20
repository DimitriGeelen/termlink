---
id: T-2809
name: "Chase the unfiled BVP-estimator corruption note — it no longer reproduces, and the fallback path strips comments instead"
description: >
  Commit 444a7e9b3 (2026-06-13) ends "Framework bug to file", describing the bvp estimator corrupting old-format task frontmatter across 106 files. It was never filed. Attempted to file it: across 26 real old-format tasks the corruption does NOT reproduce and appears fixed upstream. A different defect does exist — the no-ruamel fallback silently strips every frontmatter comment. Report both, and do not document a warning for a bug that is gone.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, bvp, g-062, directive-2]
components: []
related_tasks: [T-2203, T-2808, T-2801, T-1166]
created: 2026-08-20
last_update: 2026-08-20
date_finished: null
---

# T-2809: Chase the unfiled estimator-corruption note

## Context

Found while closing T-2203. Its sweep commit `444a7e9b3` (2026-06-13) ends:

> NOTE: bvp recalc (estimate all + estimate-cost all) was reverted this session — the bvp
> estimator corrupts OLD-format frontmatter (no template anchor): it wrote proposed-score
> list items WITHOUT the `bvp_scores_proposed:`/`cost_estimate_proposed:` key, producing
> orphaned-list malformed YAML in 106 files. **Framework bug to file.**

**It was never filed.** For ten weeks the only record that the estimator could destroy task
files was a paragraph in one commit message — a place nothing reads. Same shape as T-1166's
71-day-late checkpoint and T-2801's stranded envelope: an obligation written where it has no
reader.

### Why this looked urgent

**T-2808 ran `estimate all` and `cost-all` earlier in this same session**, before the note was
found. Re-checked immediately on finding it: **0 malformed frontmatter across all 2459 task
files.** So no damage — but at that point I could not tell whether that was luck (the run was
scoped `--statuses started-work`, all modern-template files) or because the bug was gone.

### What the evidence actually says

It is gone, or at least does not reproduce here.

| test | result |
|---|---|
| `estimate all` on 1 anchor-less old-format task (scratch copy) | key written correctly, parses |
| `estimate all` + `cost-all` on **25** anchor-less old-format tasks | 25 wrote, 0 errored, **0 malformed** |
| same, with `_HAS_RUAMEL` forced **False** (the fallback branch) | valid YAML, key present |

The reported symptom is list items emitted *without* their owning key — the signature of a
comment-preserving round-tripper losing the key. Neither the ruamel path nor the fallback
produces it on the current vendored estimator. The June observation was almost certainly real
at the time; the code has moved since, and the vendored copy carries no history here to diff
against (it was untracked until T-2807 committed it), so "fixed upstream" is the inference,
not a proven bisect.

### The defect that IS there

The no-ruamel fallback rewrites frontmatter through `yaml.safe_dump`, which **silently strips
every comment**. The task template documents its own fields in comments — `# bvp_scores:`,
`# revisit_at:`, the whole BVP block — so on a host without ruamel installed, running the
estimator quietly deletes the documentation from every task it touches. Valid YAML, no error,
no warning. That is a smaller blast than the reported one and a real Directive #2 violation.

**Exposure if it fires:** 1792 of 2460 task files (73%) lack the anchor; an unscoped
`estimate all` writes to all of them.

## Approach

Report the outcome upstream rather than the original claim: that a bug was recorded and never
filed, that it no longer reproduces, exactly what was tested so nobody re-derives it, and the
comment-stripping finding that surfaced instead.

**Do not** add a CLAUDE.md warning about the corruption. Documenting a hazard that no longer
exists is cargo-culting — it costs every future reader attention and slowly becomes folklore
nobody can check. Record the outcome, not the fear.

## Scope boundary

Tests, measures, files, records. Does **not** patch the estimator (vendored, G-062). Does
**not** install ruamel anywhere or change any host's environment. Does **not** migrate the 1792
old-format task files to the modern template — a separate deliverable and a very large diff.

## Acceptance Criteria

### Agent
- [x] Reproduction attempted on **copies**, never live task files — scratch `PROJECT_ROOT`
      trees, 26 real old-format task files total
- [x] **The reported corruption does NOT reproduce.** `estimate all` + `cost-all` over 25
      anchor-less old-format tasks: 25 wrote, 0 errored, **0 malformed frontmatter**
- [x] The no-`ruamel` fallback tested separately (forced `_HAS_RUAMEL=False`): also produces
      valid YAML with the key present — the orphaned-list symptom is not that branch either
- [x] **A different, real defect found in that fallback:** `yaml.safe_dump` silently strips
      every frontmatter comment, and the task template documents its fields in comments
- [x] Blast radius measured: **1792 of 2460** task files (73%) lack the anchor
- [x] No live task file corrupted — all 2459 re-scanned afterwards, 0 malformed
- [x] Filed upstream on `framework:pickup` **offset 23** with what was tested, the negative
      result, the comment-stripping finding, and the ten-week unfiled gap
- [x] CLAUDE.md records the outcome — explicitly NOT a warning about the corruption; it also
      documents the ruamel caveat and the `cost-*` gap that makes quadrants silently empty

## Verification

# ONE LINE per command — the P-011 gate executes each non-comment line separately,
# so a multi-line `python3 -c` heredoc is torn into fragments that each fail.
# No live task file has malformed frontmatter. This is the property the reproduction
# work had to preserve, and the same check that proved T-2808's run was safe.
python3 -c "import glob,sys,yaml; bad=[]; [bad.append((f,'bad')) for f in glob.glob('.tasks/active/*.md')+glob.glob('.tasks/completed/*.md') for s in [open(f,errors='replace').read()] for e in [s.find(chr(10)+'---'+chr(10),4)] if e<0 or not isinstance((yaml.safe_load(s[4:e]) if True else None),dict)]; sys.exit('MALFORMED: %d %s' % (len(bad),bad[:3]) if bad else 0)"
# The estimator and its policy inputs are tracked, so this survives a clean clone.
test -n "$(git ls-files .agentic-framework/agents/termlink/bvp-estimator/estimator.py)"
test -n "$(git ls-files policy/bvp-scoring-rubric.md)"
# CLAUDE.md records the outcome and names the ruamel caveat.
f=$(mktemp); grep -n 'does not reproduce' CLAUDE.md > "$f" 2>/dev/null; n=$(wc -l < "$f"); rm -f "$f"; test "$n" -ge 1

## Decisions

### 2026-08-20 — Test the claim before filing it

- **Chose:** Reproduce on scratch copies across 26 old-format tasks before writing any report.
- **Why:** The note was ten weeks old and described a specific, checkable symptom. Filing it
  unverified would have sent upstream a bug that does not exist, which costs their time and
  costs this project credibility on the next report — and the framework-pickup topic only
  works if what arrives on it is trustworthy.

### 2026-08-20 — Reproduce on a copy, never in place

- **Chose:** Scratch `PROJECT_ROOT` trees.
- **Why:** The reported defect destroys the file it writes to. In a repo where 2459 task files
  are the project's memory, confirming that in place would manufacture the damage.

### 2026-08-20 — Report the negative result rather than quietly dropping it

- **Chose:** File upstream even though the answer is "does not reproduce".
- **Why:** A negative is information: it tells upstream the fix held, and it tells the next
  reader of that commit message they need not re-investigate. Dropping it because the news is
  undramatic recreates the original failure — a finding known to one session and no one else.

### 2026-08-20 — No CLAUDE.md warning about the corruption

- **Chose:** Record the outcome; do not add a hazard note.
- **Why:** Documenting a defect that no longer reproduces is cargo-culting. It taxes every
  future reader, cannot be falsified by anyone who trusts it, and this repo's CLAUDE.md is
  already long enough that adding unfalsifiable folklore has a real cost.
