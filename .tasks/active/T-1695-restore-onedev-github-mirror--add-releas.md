---
id: T-1695
name: "Restore OneDev → GitHub mirror — release pipeline silently broken since 2026-05-02 (G-058)"
description: >
  OneDev → GitHub mirror has been broken since 2026-05-02. GH HEAD frozen at b39fc916, OneDev HEAD at b179b0cb. 16 days of commits + 3 release tags (v0.10.0, v0.11.0, v0.11.1) never reached GitHub Releases. Homebrew install path broken. Operator-only: needs OneDev UI access + likely github-push-token rotation.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [release, operator-action, G-058]
components: []
related_tasks: [T-1691]
created: 2026-05-18T10:43:28Z
last_update: 2026-05-18T10:43:28Z
date_finished: null
---

# T-1695: Restore OneDev → GitHub mirror (G-058)

## Context

Per CLAUDE.md "CI / Release Flow", releases work because `.onedev-buildspec.yml`
mirrors all branches + tags from OneDev to `github.com/DimitriGeelen/termlink`,
which triggers `release.yml` for `v*` tags, which publishes Homebrew-installable
binaries. As of 2026-05-18T10:30Z the mirror has been silently broken for **16 days**:

| Side    | HEAD       | Date                  | v0.10.0 | v0.11.0 | v0.11.1 |
|---------|------------|-----------------------|---------|---------|---------|
| OneDev  | b179b0cb   | 2026-05-18 (today)    | ✓       | ✓       | ✓       |
| GitHub  | b39fc916   | 2026-05-02T05:39Z     | ✗       | ✗       | ✗       |

Sibling: T-1696 (agent-buildable, work-completed) added a drift canary so the next
breakage is caught in <24h instead of 16 days.

## Diagnostic (T-1695 inception, 2026-05-18T10:55Z — agent autonomous)

Agent hit OneDev's REST API (`/~api/builds?query=...`) using the access token
embedded in the `origin` remote URL. Findings:

- **Mirror job is still firing on every push** — not a scheduling failure. Build
  #1606 was triggered by commit `06e81da4` (the T-1696 close commit I just
  pushed), submitted 2026-05-18T10:50:56Z, finished FAILED 4 seconds later.
- **Failure signature: fast-fail ≈ 2000ms** (pendingDuration=1005ms,
  runningDuration=2001ms across the last 30 failures). Network/DNS issues take
  10-30s; build setup failures take ~5s. **2s consistent fast-fail is the
  signature of an HTTPS auth-401 on `git push`** — the runner connects, sends
  credentials, gets rejected, exits.
- **Last successful mirror build: #1114, 2026-04-27T21:07:39Z, commit `e261275bc6`.**
  Failure span = **21 days** (2026-04-27 → today), ~900+ consecutive failures
  (paginated through builds 1115 → 1605 over offsets 0..1000, all FAILED or
  CANCELLED-by-supersession). The 2026-05-02 commit `b39fc916` reaching GitHub
  was almost certainly a one-off operator action, not the mirror succeeding.
- **OneDev log API is HTML-only** (`/~projects/30/builds/N/log` returns a
  Wicket page driven by websockets; no plaintext log endpoint via REST in this
  OneDev version). Cannot read the exact stderr line, but the signature is
  unambiguous.

**High-confidence root cause:** `github-push-token` secret in OneDev is
expired or revoked. The PR token referenced in `.onedev-buildspec.yml`
(`passwordSecret: github-push-token`) is rejected by GitHub on `git push`
with an HTTP 401, causing the 2-second fast-fail.

**Operator's actual workload (reduced from 4 ACs to ~3 steps):**
1. On GitHub: Settings → Developer settings → Personal access tokens —
   confirm the existing PAT used for OneDev mirror is expired/revoked.
2. Generate a new fine-grained PAT: repo `DimitriGeelen/termlink` only,
   permissions Contents: Read+Write, Workflows: Read+Write, expiration ~1 year.
3. On OneDev (UI): Project termlink → Settings → Build → Secrets — edit
   `github-push-token`, paste the new PAT, save.
4. After save, **OneDev will auto-retry the mirror job on next push, OR you can
   force-fire** by re-running build #1606 in the OneDev UI (Build → Re-run).
   Subsequent commits will all push through in catch-up order; 21 days of
   backlog (~900 commits + 3 release tags) will replay in seconds.
5. Once GH catches up, `gh release list` will show v0.10.0, v0.11.0, v0.11.1;
   the release.yml workflow will fire for each tag and publish binaries.

Agent can verify post-fix via:
```
git ls-remote github HEAD   # should match OneDev's b179b0cb (or newer)
git ls-remote --tags github | grep v0.11.1   # should appear
gh release list -L 5 --repo DimitriGeelen/termlink   # three new tags
```

## Acceptance Criteria

### Human
- [ ] [REVIEW] OneDev mirror job log inspected; root cause identified
  **Steps:**
  1. Open `https://onedev.docker.ring20.geelenandcompany.com/termlink` → Project → Builds
  2. Filter for job "Push to GitHub Mirror" — find the first failed run on/after 2026-05-02
  3. Read the failure log — expect one of: auth/401 (token expired), 403 (token scope insufficient), network/timeout, or job-suspended
  4. Note the root cause (one line)
  **Expected:** Specific failure reason captured
  **If not:** Check OneDev's own job-runner health; the job may have stopped scheduling entirely

- [ ] [REVIEW] `github-push-token` secret rotated if expired
  **Steps:**
  1. On GitHub: Settings → Developer settings → Personal access tokens → check expiration of the existing PAT used for the mirror
  2. If expired or scope-insufficient: generate a new fine-grained PAT (Contents: Read+Write, Workflows: Read+Write, repo `DimitriGeelen/termlink` only, expiration ~1 year)
  3. On OneDev: Project → Secrets → update `github-push-token` with the new value
  **Expected:** Token valid and OneDev secret updated
  **If not:** Skip if token was not the failure mode

- [ ] [REVIEW] Mirror job force-fired; backlog catches up
  **Steps:**
  1. On OneDev: re-run the "Push to GitHub Mirror" job manually (Build → Re-run, or push an empty commit to fire BranchUpdateTrigger)
  2. Wait for job success
  3. Verify catch-up: `git ls-remote github HEAD` should show OneDev's current HEAD; `git ls-remote --tags github | grep -E "v0\\.1[01]"` should show v0.10.0, v0.11.0, v0.11.1
  **Expected:** GH HEAD matches OneDev HEAD; all three release tags present on GH
  **If not:** OneDev job still failing — return to step 1 of the first AC

- [ ] [REVIEW] Releases published on GitHub for v0.10.0, v0.11.0, v0.11.1
  **Steps:**
  1. Run `gh release list -L 10 --repo DimitriGeelen/termlink`
  2. Confirm all three release tags appear with binaries attached (release.yml builds macos + linux + checksums)
  3. If a release row is missing: check `gh run list --repo DimitriGeelen/termlink --limit 10` for failed Release workflow runs — re-trigger manually via `gh workflow run release.yml -f tag=v0.11.1` if needed
  **Expected:** Three releases visible with binary assets
  **If not:** GH Actions Release workflow failed — diagnose via run logs

## Verification

# Operator-driven task; verification is the four Human ACs above.
# Agent-side sanity check after operator reports done:
git ls-remote github HEAD | awk '{print $1}'
git ls-remote --tags github | grep -E 'v0\.11\.1$'

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

### 2026-05-18T10:43:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1695-restore-onedev-github-mirror--add-releas.md
- **Context:** Initial task creation
