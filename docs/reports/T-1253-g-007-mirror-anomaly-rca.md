# T-1253: G-007 mirror anomaly — root cause analysis

**Task:** T-1253 (inception, GO 2026-04-25)
**Build follow-up:** T-1255 (commit `0ea839b7`, upstream `7f84a3ec`)
**Status:** RESOLVED — see G-007 in `.context/project/concerns.yaml`

## Problem statement

G-007 had been "watching" since 2026-04-12: the GitHub mirror would lag the
OneDev source-of-truth by 25+ minutes despite successful pushes to OneDev.
CLAUDE.md "CI / Release Flow" enshrines OneDev as source-of-truth + GitHub
as a read-only mirror via `.onedev-buildspec.yml`'s `PushRepository` job
(BranchUpdateTrigger).

The 2026-04-25T16:08Z observation **inverted the direction:** after a
normal commit + auto-handover sequence on /opt/termlink, GitHub was
**ahead** of OneDev:

| Probe                       | Result                              |
| --------------------------- | ----------------------------------- |
| local HEAD                  | `93e39ff1`                          |
| `git ls-remote github main` | `93e39ff1` (in sync with local)     |
| `git ls-remote origin main` | first attempt 502, retry `a586edd8` (14+ min stale) |
| after `git push origin main`| onedev advances to `93e39ff1`       |

GitHub was AHEAD of OneDev for window `[16:07:05, 16:08:23+]`. Per the
documented mirror flow, that should be impossible — github only receives
updates *via* onedev's PushRepository job.

## Assumptions tested

- **A-1 (DISPROVEN):** `fw git commit` auto-pushes to remotes. Disproved by
  reading `.agentic-framework/agents/git/git.sh` and
  `.agentic-framework/lib/version.sh`; no auto-push at commit time. Git
  hooks (post-commit, pre-push) do not push either.
- **A-2 (CONFIRMED):** The handover agent, when invoked with `--commit` (the
  routine path, including auto-handover from PreCompact + budget-checkpoint
  hooks), pushes to **all** configured remotes individually, not just
  `origin`. Confirmed by `.agentic-framework/agents/handover/handover.sh:771-790`:
  ```bash
  for remote_name in $(git -C "$PROJECT_ROOT" remote); do
      timeout 60 git push --follow-tags "$remote_name" HEAD
  ```
- **A-3 (PROBABLE):** When OneDev is briefly unreachable (502, network
  glitch) at the moment of `handover --commit`, the loop pushes to GitHub
  successfully but the OneDev push fails (or times out at 60s). Net effect:
  GitHub advances, OneDev does not. The OneDev BranchUpdateTrigger never
  fires because OneDev never received a new ref. The mirror diverges
  silently.
- **A-4 (UNTESTED):** The next successful push to OneDev does NOT trigger a
  corrective re-push to GitHub from OneDev's side, because PushRepository
  runs `force: false` and OneDev would attempt a non-fast-forward (GitHub
  already has commits OneDev doesn't), causing the push step to fail/skip.

## Decision: GO

Bounded fix with two-step path. Implemented in T-1255 (commit `0ea839b7`).

### Step 1: handover.sh push-target change (structural)

`agents/handover/handover.sh` push loop now skips non-origin remotes when
more than one remote is configured:

```bash
_remote_count=$(git -C "$PROJECT_ROOT" remote 2>/dev/null | wc -l)
while IFS= read -r remote_name; do
    if [ "$_remote_count" -gt 1 ] && [ "$remote_name" != "origin" ]; then
        echo -e "  ${CYAN}Skipping $remote_name (mirrored from origin via PushRepository)${NC}"
        continue
    fi
    ...
done
```

Single-remote behaviour preserved (degenerate case where the loop runs
once correctly).

### Step 2: audit drift check (observability)

`agents/audit/audit.sh` GIT TRACEABILITY section gained a github-vs-origin
divergence check that compares `git ls-remote origin main` and `git
ls-remote github main` when both remotes exist. Catches drift within one
audit cycle (15min) instead of via human spot-check at 25+ min stale.

### Test

`tests/handover-push-target.sh` is a hermetic test using two file:// remote
repos that proves only origin receives the push when github is also
configured.

## Scope fence

- **IN:** Identify structural cause + recommend bounded fix.
- **OUT:** Fixing OneDev's reliability (502 cause). Fixing the historical
  backlog of divergent commits (would require force-push or onedev manual
  ops).

## References

- T-1140 (PL-036, prior un-actioned warning that named this exact dual-push
  pattern) — now `status: closed` with cross-ref.
- G-007 (`.context/project/concerns.yaml`) — `status: resolved` 2026-04-25.
- Upstream commit `7f84a3ec` (mirror to consumer in `0ea839b7`).
