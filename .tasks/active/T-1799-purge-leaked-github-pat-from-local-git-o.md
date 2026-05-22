---
id: T-1799
name: "Purge leaked GitHub PAT from local git object store (replace-refs + dangling blobs) + gitignore approvals"
description: >
  T-1695 filter-repo cleaned reachable history but left 2745 refs/replace/* anchoring 2 dangling blobs that still contain the live github_pat_...7ehL token. Also .context/approvals/ is untracked-but-not-gitignored and one resolved-*.yaml carries the raw PAT. Purge object store, gitignore approvals, redact working file. PAT itself must be rotated by operator.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-22T07:20:56Z
last_update: 2026-05-22T07:20:56Z
date_finished: null
---

# T-1799: Purge leaked GitHub PAT from local git object store (replace-refs + dangling blobs) + gitignore approvals

## Context

The T-1695 `git filter-repo --replace-text` rewrite cleaned the token out of
reachable history (no branch/tag/`rev-list --all` contains it) but left 2745
`refs/replace/*` refs that anchor pre-rewrite commits whose trees still hold 2
blobs containing the live `github_pat_…7ehL`. Plus the untracked working file
`.context/approvals/resolved-772d160ab769.yaml` carries the raw PAT, and
`.context/approvals/` is not gitignored. The token is compromised (on disk +
session logs) and must be rotated by the operator regardless of git cleanup.

## Acceptance Criteria

### Agent
- [x] `.context/approvals/` added to `.gitignore` (root) so resolved-*.yaml can never be `git add`-ed again — also `git rm --cached`-ed the 2 previously-tracked (clean) approval files
- [x] PAT redacted from every working-tree file (token replaced with `[REDACTED-PAT-T-1799]`); `grep -rF <fragment> . --exclude-dir=.git` returns nothing
- [x] All 2745 `refs/replace/*` refs deleted (filter-repo cruft anchoring old objects) — `for-each-ref refs/replace` now 0
- [x] Reflogs expired (`--expire=now --expire-unreachable=now --all`) and `git gc --prune=now` run — `.git` 322M → 35M
- [x] Object-store scan proves token gone: `cat-file --batch-all-objects` grep returns 0 hits; both known blobs (1167a726, 71ab1eed) `cat-file -e` → absent; `fsck --full` dangling = 0
- [x] Reachable-history confirmed clean: `git log --all -S<token>` returns nothing (T-1695 rewrite already true; re-asserted)

### Human
- [ ] [REVIEW] Rotate/revoke the compromised PAT on GitHub
  **Steps:**
  1. Go to https://github.com/settings/tokens — find the fine-grained token ending `…7ehL`
  2. Revoke it. If the OneDev→GitHub mirror used it, mint a replacement and update OneDev's `github-push-token` secret
  **Expected:** Old token shows revoked; mirror push still works with the replacement (or is intentionally left for T-1695)
  **If not:** The token remains valid and is a live credential leak — treat as a security incident

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# NOTE: checks reference the two token-bearing blob OIDs (object hashes, not the
# secret) so this file never re-introduces the PAT nor trips the secret-scan hook.
# approvals dir is gitignored
git check-ignore .context/approvals/resolved-772d160ab769.yaml
# the two known token-bearing blobs no longer exist in the object store
! git cat-file -e 1167a72611f950d11743aeae9b5d5426539d3182 2>/dev/null
! git cat-file -e 71ab1eed5e934746b3173b4deb289e520d52ade2 2>/dev/null
# replace refs cleared (filter-repo cruft that anchored the old objects)
test "$(git for-each-ref refs/replace 2>/dev/null | wc -l)" = "0"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-22T07:20:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1799-purge-leaked-github-pat-from-local-git-o.md
- **Context:** Initial task creation
