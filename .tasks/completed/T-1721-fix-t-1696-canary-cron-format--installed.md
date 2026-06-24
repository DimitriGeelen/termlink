---
id: T-1721
name: "Fix T-1696 canary cron format — installed in user-crontab with /etc/cron.d/ USER-field syntax (silent fail)"
description: >
  T-1696's release-mirror canary was installed in root's user crontab but uses /etc/cron.d/ system-cron syntax (USER field). cron parses 'root' as the first command token → 'root: command not found' → silent exit 127 → log file never written. Result: 67-commit OneDev→GitHub drift since 2026-05-18 went undetected for 2+ days, recreating the exact G-058 silent-failure pattern the canary was designed to prevent. Fix: move canary to /etc/cron.d/termlink-release-mirror-canary and strip the broken entry from root's user crontab.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [release, cron, G-058, bug, silent-fail]
components: []
related_tasks: [T-1695, T-1696]
created: 2026-05-20T06:51:27Z
last_update: 2026-05-20T06:55:56Z
date_finished: 2026-05-20T06:55:56Z
---

# T-1721: Fix T-1696 canary cron format — installed in user-crontab with /etc/cron.d/ USER-field syntax (silent fail)

## Context

T-1696 shipped a daily canary (`scripts/check-mirror-freshness.sh --quiet`) to detect OneDev→GitHub mirror drift in <24h instead of 16 days (G-058). The source-of-truth crontab file `.context/cron/release-mirror-canary.crontab` correctly uses `/etc/cron.d/` syntax (with explicit `USER` field on the schedule line). However, the entry was loaded into root's **user crontab** (`crontab -e`) instead of `/etc/cron.d/`. In user-crontab format, there is no USER field — the schedule line `13 7 * * * root cd /opt/termlink && ...` is parsed as `command = "root cd /opt/termlink && ..."`. `root` is not a binary in `$PATH`, so the command exits 127 daily with no log written. Confirmed via syslog (CRON daemon logs both 2026-05-19 07:13 and 2026-05-20 07:13 firings) + reproduction (`bash -c 'root bash scripts/check-mirror-freshness.sh --quiet'` → `bash: line 1: root: command not found`).

Live drift state at T-1721 filing (2026-05-20T06:49Z):
- OneDev HEAD: `a5a469e3` (today's handover commit)
- GitHub HEAD: `8e9f4e62` (2026-05-18, T-1695 manual catch-up)
- 67 commits behind, 2+ days stale — never flagged.

This is structurally the same failure mode the canary was designed to prevent: silent fail in the release pipeline. PL-168 ("Canary scripts without a trigger are not prevention — they are dormant tooling") describes exactly this regression. The fix relocates the cron entry to the canonical install location matching other termlink crons (`/etc/cron.d/termlink-heartbeat`, `/etc/cron.d/termlink-watchdog`).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Source-of-truth `.context/cron/release-mirror-canary.crontab` installed as `/etc/cron.d/termlink-release-mirror-canary` with mode 644 root:root, matching the convention used by `termlink-heartbeat` + `termlink-watchdog`. Verified: `stat -c '%a %U %G'` returned `644 root root`.
- [x] Broken `13 7 * * * root cd /opt/termlink && ...` line removed from root's user crontab. `sudo crontab -l | grep -c release-mirror` returns 0. Orphan T-1696 comment stanza (lines 12-24 of pre-strip crontab) also cleaned to prevent confusion.
- [x] `/etc/cron.d/termlink-release-mirror-canary` parsed cleanly by cron — no error messages in `journalctl -u cron --since "5 minutes ago"` after install. The file uses the same USER-field syntax that already-working `/etc/cron.d/termlink-heartbeat` and `/etc/cron.d/termlink-watchdog` use, so parser acceptance is structurally guaranteed.
- [x] One manual fire reproduced the cron-equivalent execution and wrote the expected drift line to `.context/working/.release-mirror-canary.log`. Log content matches live state (`GitHub mirror: drift` + `origin (OneDev): a5a469e3...` + `GitHub: 8e9f4e62...` + `GitHub is 67 commit(s) behind origin`). 8 log lines = 2 manual fires × 4-line drift output, confirming append semantics work.
- [x] `## Verification` section populated with 4 commands; all pass against current state (see Updates entry below for verbatim outputs).
- [x] Source-of-truth crontab file updated with a `# Installed to: /etc/cron.d/termlink-release-mirror-canary (USER field syntax — do NOT load into a user crontab)` comment line near the top — matches the convention used by `.context/cron/heartbeat.crontab` and prevents the misload that caused this bug. `/etc/cron.d/` resynced post-edit; parity confirmed by `diff -q`.

<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
test -f /etc/cron.d/termlink-release-mirror-canary
test "$(stat -c '%a %U %G' /etc/cron.d/termlink-release-mirror-canary)" = "644 root root"
test "$(crontab -l 2>/dev/null | grep -c release-mirror)" = "0"
diff -q .context/cron/release-mirror-canary.crontab /etc/cron.d/termlink-release-mirror-canary

## RCA

**Symptom:** OneDev→GitHub mirror silently drifted by 67 commits over 2+ days post-T-1695 manual catch-up. The T-1696 canary log (`.context/working/.release-mirror-canary.log`) showed zero entries despite cron firing twice in the drift window.

**Root cause:** The source-of-truth crontab file `.context/cron/release-mirror-canary.crontab` is correctly formatted for `/etc/cron.d/` (with USER field). It was installed into root's *user* crontab instead, where the USER field is not part of the format — the `root` token was parsed as the leading command word, causing `bash: line 1: root: command not found` and exit 127 on every fire. No output → empty log → no drift signal.

**Why structurally allowed:** No audit/doctor check verifies the canary's actual *trigger* — only the source-of-truth file's presence on disk. The framework's PL-168 ("Canary scripts without a trigger are not prevention") was registered at T-1696 close but no enforcement test was wired to it. There is no install script (`scripts/install-*-cron.sh`) for the canary that would have produced the right install location by construction (sibling `termlink-heartbeat` has one, this one was hand-rolled). Combined with the silent-fail nature of cron stdout/stderr redirection, the bug was undetectable from inside the project until I happened to investigate the empty log.

**Prevention (separate follow-ups, not in scope for T-1721):**
1. `scripts/install-release-mirror-canary-cron.sh` to make `/etc/cron.d/` install the only path — mirrors `scripts/install-heartbeat-cron.sh` pattern.
2. `fw doctor` (or audit) check that every `.context/cron/*.crontab` entry is reflected in `/etc/cron.d/<name>` AND is **NOT** present in root's user crontab. Cron-install-location consistency lint.
3. Smoke-test the canary itself periodically — if the log file is older than `2 × cron_interval`, the canary is silently broken (meta-canary). For this canary that means: if `.release-mirror-canary.log` mtime older than 48h AND HEAD-vs-GH drift is non-zero, fire a warning. (Add to canary script itself: a stale-self check.)
   *Filed as T-1722 (TODO this session).*

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

### 2026-05-20T06:51:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1721-fix-t-1696-canary-cron-format--installed.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-0927548e
- **Timestamp:** 2026-05-20T06:55:56Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-20T06:55:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
