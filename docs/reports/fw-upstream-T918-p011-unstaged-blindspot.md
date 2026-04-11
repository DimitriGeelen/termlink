# Framework upstream report: P-011 verification gate is blind to un-staged files

**Discovered:** 2026-04-11 in /opt/termlink session S-2026-0411-1559
**Severity:** low (governance blindspot, no data loss)
**Source policy:** P-011 (Verification Gate)

## Observation

Closing T-906 and T-907 in the termlink project passed the P-011 verification
gate even though their deliverable files were untracked — they existed on disk
(written 2026-04-08 by a prior session) but were never `git add`ed. The gate's
`test -f docs/reports/T-XXX.md` command checks the working tree, which always
sees working-directory files regardless of index state.

## Root cause

`agents/task-create/update-task.sh` runs the `## Verification` block as raw
shell. The idiomatic pattern taught in the framework README is `test -f <path>`.
That command has no git-awareness. Any file in the working directory passes,
including:

- Files written but never staged (the case we hit)
- Files ignored by `.gitignore`
- Files in a worktree but not in the current HEAD's index

The gate was designed to prove that a deliverable *exists* (anti-fraud against
tasks that close without producing anything). It does not prove the deliverable
*was committed*. In practice those two checks are almost always the same — but
the failure mode is silent when they diverge.

## Reproduction (from any framework consumer)

```bash
mkdir -p docs/reports
echo "placeholder" > docs/reports/test-blindspot.md   # untracked, on disk
.agentic-framework/bin/fw task create --name "blindspot probe" --type build
# Edit .tasks/active/T-XXX-*.md to add:
#   ## Verification
#   test -f docs/reports/test-blindspot.md
.agentic-framework/bin/fw task update T-XXX --status work-completed
# → gate PASSES, task closes, git status still shows test-blindspot.md as untracked
```

## Real-world incident

In /opt/termlink:

- T-906 deliverable: `docs/reports/T-906-model-param-dispatch.md` (written
  2026-04-08, committed 2026-04-11 at c242a52 after being orphaned for 3 days)
- T-907 deliverable: `docs/reports/verification-T1061-phases.md` (same story)

Both tasks had already been moved to `.tasks/completed/` weeks ago. The
deliverables were only caught during a broader tree cleanup; without that, the
files could have sat untracked indefinitely.

## Remediation options

### Option A — Track-aware helper (minimal, additive)

Add `fw verify exists <path>`:

```bash
fw verify exists() {
    local path="$1"
    test -f "$path" || { echo "missing on disk: $path"; return 1; }
    git ls-files --error-unmatch "$path" >/dev/null 2>&1 || {
        echo "untracked: $path"
        return 1
    }
}
```

Update templates and task-creation docs to prefer `fw verify exists` over raw
`test -f`. Existing tasks using raw `test -f` keep working — no retroactive
break. Tasks that want strict tracking opt in explicitly.

**Pros:** explicit, back-compat, documentable, teachable.
**Cons:** relies on humans/agents remembering to use the helper.

### Option B — Auto-upgrade `test -f` in the gate (stronger, implicit)

In `update-task.sh`, detect `test -f <path>` lines in `## Verification` and
augment them to also run `git ls-files --error-unmatch`.

**Pros:** catches every existing task without any edit.
**Cons:** silently changes the semantics of verification commands — tasks that
*intentionally* check a file that shouldn't be tracked (e.g. build artifact,
runtime log) would start failing with no obvious cause.

## Recommendation

**Option A.** Gates should announce their semantics, not upgrade them silently.
Adding a new helper is honest; monkey-patching `test -f` creates a debugging
trap the next time someone writes `test -f /tmp/something.log`.

## Delivery

This report is authored in the termlink project. Deliver upstream via:

```bash
cd /opt/termlink && .agentic-framework/bin/fw task work-on T-918
# then termlink push --to fw-agent docs/reports/fw-upstream-T918-p011-unstaged-blindspot.md
# or copy manually to the framework repo's docs/upstream-reports/
```

The framework session picks this up as a structural governance concern, not as
a blocking incident — no consumer is broken, but the gate's claim ("closed
tasks have their deliverables") is slightly less true than it appears.
