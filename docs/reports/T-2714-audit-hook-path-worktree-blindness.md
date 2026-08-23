# audit.sh hook checks are structurally blind in a linked worktree

**Filed by:** 010-termlink · **Task:** T-2714 · **Date:** 2026-08-23
**Component:** `agents/audit/audit.sh` (vendored) · **Class:** false negative about a live safety gate

## Summary

Three checks in `audit.sh` decide whether a git hook is installed by
string-concatenating `"$PROJECT_ROOT/.git/hooks/<name>"`. In a **linked worktree**
`.git` is a *file* containing `gitdir: …`, not a directory, so that path can never
exist. All three checks report the hook as missing — permanently, and regardless of
whether it is installed and firing. The same concatenation also ignores
`core.hooksPath`, which git honours.

The most serious of the three is **C-002**, which does not claim a *file* is absent
but that a *specific safety gate* is absent from a hook that is present and enforcing.

## The three sites (line numbers at `4f9068496~1`)

| Line | Check | Claim when it misfires |
|---|---|---|
| `audit.sh:2393` | `[ -f "$PROJECT_ROOT/.git/hooks/commit-msg" ]` | "No commit-msg hook" |
| `audit.sh:3022` | `grep -q "inception-research-warnings" "$PROJECT_ROOT/.git/hooks/commit-msg" 2>/dev/null` | C-002: commit-msg hook missing research-artifact check |
| `audit.sh:3375`, `:3379` | `[ -x "$PROJECT_ROOT/.git/hooks/pre-push" ]` | CTL-011: pre-push hook missing or not executable |

`:3022` additionally routes the error to `2>/dev/null`, so "Not a directory" is
indistinguishable from a genuinely missing gate.

## Evidence — the hooks are installed and do fire

Measured in the worktree that reports all three findings:

```
$ cat .git
gitdir: /opt/termlink/.git/worktrees/t2687-pickup-failopen     # a FILE, not a dir

$ git config core.hooksPath
/opt/termlink/.git/hooks                                        # honoured by git, ignored by audit.sh

$ git rev-parse --git-path hooks/commit-msg
/opt/termlink/.git/hooks/commit-msg                             # 10729 bytes, mode 775
```

The installed `commit-msg` **does** carry the marker C-002 greps for, and the
enforcement block behind it is live:

```
/opt/termlink/.git/hooks/commit-msg:165  # Block inception commits after the first if no docs/reports/T-XXX artifact exists.
/opt/termlink/.git/hooks/commit-msg:166  # inception-research-warnings: audit marker (C-002 OE check)
/opt/termlink/.git/hooks/commit-msg:174          HAS_STAGED_RESEARCH=$(git diff --cached --name-only | grep -c "^docs/reports/" || true)
```

Empirically, hooks fire on every commit in this worktree — the handover commit
made during this session printed `Task T-2723 updated (…)` from the post-commit
hook. So upstream must **not** "fix" this by reinstalling hooks; nothing is missing.

## Why this is worse than a cosmetic warning

The mitigation text reads *"Install hooks: ./agents/git/git.sh install-hooks"*.
Running it does the right thing — it writes to the common hooks dir, which is where
git actually looks — **and the warning survives anyway**, because the check never
looks there. An operator following the instruction concludes that either
`install-hooks` is broken or the finding is noise. During the originating audit the
reporting agent ran it and recorded CTL-011 as remediated before re-running the
check; it was not, and could not have been.

A mitigation that cannot fix its own finding costs more than no mitigation, because
it spends the operator's trust in the check. And a false negative about a *gate's
existence* (C-002) is strictly worse than one about a *file's existence*: it tells
the operator that inception research-artifact enforcement is off while it is running
on every commit.

## Remedy

Resolve through git rather than concatenating a layout assumption. One call handles
the normal checkout, the linked worktree, and `core.hooksPath`:

```sh
_resolve_hook_path() { # $1 = hook name → absolute path (may not exist)
    local _p
    _p=$(git -C "$PROJECT_ROOT" rev-parse --git-path "hooks/$1" 2>/dev/null) || _p=""
    if [ -z "$_p" ]; then
        printf '%s\n' "$PROJECT_ROOT/.git/hooks/$1"   # fallback: preserve old behaviour
        return
    fi
    case "$_p" in
        /*) printf '%s\n' "$_p" ;;
        *)  printf '%s\n' "$PROJECT_ROOT/$_p" ;;
    esac
}
```

**Define it at top level, not inside a `should_run_section` block.** The three hook
checks live in three different sections (`enforcement`, `oe-research`, `oe-daily`).
A section-local definition means `--section oe-daily` alone calls an undefined
function, which resolves to the empty string and produces a false WARN with a blank
Evidence line. We shipped that bug and fixed it in a follow-up commit; a full run
masks it because `enforcement` happens to define the function first.

## Status in 010-termlink

Applied locally as T-2721 (`4f9068496`, `86e9db17e`) against the vendored copy, and
registered in `.vendor-divergence.yaml` as `status: local-only`. Per G-062 a local
edit to vendored code is erased by the next `fw upgrade`, which is why this is filed
rather than left as a patch. There is no change to request beyond taking the fix
upstream.

## Related

Two sibling defects found in the same audit pass, same shape — *a check whose verdict
is decided by an assumption about layout that no longer holds*:

- **T-2711** — `PROJECT_ROOT` walk matches the vendored framework's `FRAMEWORK.md`
  before the consumer's `.framework.yaml`, so `revisit-due-scan.sh` resolves to
  `.agentic-framework/.tasks/active`, finds nothing, and `exit 0`s. "I could not
  look" and "I looked and found nothing" share an exit code.
- **T-2713** — hook telemetry counts exit-2 blocks as hook *failures*, so a gate
  doing its job reads as a broken gate.
