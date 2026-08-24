# CTL-020 fires unconditionally in a linked worktree, and its mitigation makes things worse

**Filed by:** 010-termlink · **Task:** T-2715 · **Date:** 2026-08-24
**Component:** `agents/audit/audit.sh` (vendored) · **Class:** unreachable-by-design check with a harmful remediation

## Summary

CTL-020 checks that cron audit snapshots were produced in the last hour by testing
`[ -d "$CRON_DIR" ]`, where `CRON_DIR="$AUDITS_DIR/cron"`. In a **linked worktree**
that directory does not exist and cannot: `.context/audits/cron/` is **gitignored by
design**, so a worktree — which materialises tracked files only — never has it. The
check therefore fires on every run, and the branch it takes is the worse of the two.

This is the sibling of T-2714 (hook checks concatenating `$PROJECT_ROOT/.git/hooks/`).
Same shape: *a check whose verdict is decided by an assumption about layout that no
longer holds*. The fix here is smaller, because the file already contains the pattern.

## The site, and the rule that makes it unreachable

Line numbers at `4f9068496~1`:

```sh
# audit.sh:3213
# CTL-020 OE: Continuous Audit — cron audit files produced recently
CRON_DIR="$AUDITS_DIR/cron"
if [ -d "$CRON_DIR" ]; then
    ...
else
    grace_warn "CTL-020: Cron audit directory missing ($CRON_DIR)" \
         "Directory not created" \
         "Run: fw audit schedule install"
fi
```

and the rule that guarantees the `else` branch in any worktree:

```
# .gitignore:84-87  (T-1624)
# T-1624: cron audit snapshots are 15-min telemetry. On-disk rotation
# (audit.sh:119: find -mtime +7 -delete) doesn't sync to git → ~600 dirty
# files per session. Same class as L-028 (ephemeral session state).
.context/audits/cron/
```

Note the two decisions are individually correct and only interact badly. T-1624
gitignored the directory for a good reason — it is high-churn telemetry that would
otherwise dirty the tree ~600 files per session. CTL-020 checks it for a good reason
— continuous audit is worth verifying. Neither author was wrong; nothing connected
them.

*(A note on line references: an earlier draft of this task cited `.gitignore:54`.
That has drifted — line 54 is now a `__pycache__` rule. The cron rule is at :87.
Cited here as measured rather than as previously recorded.)*

## Why the mitigation is harmful, not merely useless

The remediation reads **"Run: `fw audit schedule install`"**. Running it in a
worktree installs a **host-level cron entry aimed at the worktree path**. Worktrees
are deleted — routinely, and often automatically at session end. So the operator is
instructed to point a persistent host cron at a path that will disappear, leaving a
cron job that fails on every fire, on a host whose `/etc/cron.d` the worktree does
not own. It also collides with the entry the main checkout already installed.

This is worse than a warning that says nothing: it converts a false finding into
durable host-level damage, and the damage outlives the worktree that caused it.
(Compare T-2815: `fw cron install` derives its `/etc/cron.d` filename from the
checkout basename, so a worktree run writes `agentic-audit-<worktree-name>` rather
than updating `agentic-audit-termlink` — the same "worktree writes host state under
the wrong identity" family.)

## Why `mkdir -p` is not the fix

The obvious shortcut — create the directory so the `-d` test passes — is worse
again. It moves the run into the `if` branch, where `find -mmin -60` returns zero,
and the check then reports **"CTL-020: No cron audit files in last hour"**. That is
a *stronger* claim than the one it replaced: it asserts the host's cron is not
firing, on the evidence of a directory this session just created and no cron has
ever written to. It fabricates host state and then reports on it.

The check should not be made to pass. It should decline to answer, because in a
worktree it has no evidence either way.

## Remedy — the file already has the pattern

Two checks in the same script already handle exactly this, using a shared helper
(`lib/paths.sh:212`):

```sh
# audit.sh:1638
if [ -f "$_cron_registry" ] && fw_is_linked_worktree "$PROJECT_ROOT"; then
# audit.sh:1708
if [ -d "$_cron_lint_dir" ] && fw_is_linked_worktree "$PROJECT_ROOT"; then
```

Both emit `info` — not `warn` — because "this host's cron is managed from the main
checkout" is a *fact about scope*, not a finding. CTL-020 should join them:

```sh
if fw_is_linked_worktree "$PROJECT_ROOT"; then
    info "CTL-020 skipped — linked worktree (cron audits are host-level, produced from the main checkout)"
elif [ -d "$CRON_DIR" ]; then
    ...unchanged...
fi
```

`info` matters and is not cosmetic. `warn` in a worktree is unactionable-by-
construction noise, and a check that cannot be cleared trains the operator to stop
reading the section it lives in — which is how a genuine CTL-020 finding in the main
checkout gets skipped.

## Status in 010-termlink

Applied locally as T-2721 (`4f9068496`) and registered in `.vendor-divergence.yaml`
as `filed-upstream`. Per G-062 the local edit is erased by the next `fw upgrade`,
which is why this is filed rather than left as a patch. Verified live: the pre-push
audit on this branch now prints `Cron drift checks skipped — linked worktree`, and
the skip is at `audit.sh:3250` post-fix.

## Related

- **T-2714** — the three hook checks concatenating `$PROJECT_ROOT/.git/hooks/`,
  filed at `framework:pickup` offset 35. Same pass, same shape, same remedy family
  (ask git / ask the helper, do not assume the layout).
- **T-2815** — `fw cron install` deriving its `/etc/cron.d` filename from the
  checkout basename. The write-side counterpart of this read-side blindness.
