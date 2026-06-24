---
id: T-1824
name: "fleet secrets-audit --target-cache <PATH> — narrow drift check to one named cache file (T-1822 follow-up #1)"
description: >
  fleet secrets-audit --target-cache <PATH> — narrow drift check to one named cache file (T-1822 follow-up #1)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-28T08:13:50Z
last_update: 2026-05-28T08:25:23Z
date_finished: 2026-05-28T08:25:23Z
---

# T-1824: fleet secrets-audit --target-cache <PATH> — narrow drift check to one named cache file (T-1822 follow-up #1)

## Context

T-1822 shipped `fleet secrets-audit --check-drift <PATH>` which compares
every scanned cache file against the named authoritative. The Recommendation
flagged a semantic edge case (now confirmed live on .107): when the cache
dir contains caches for peer hubs and the authoritative is the LOCAL hub's
`<runtime_dir>/hub.secret`, peer caches correctly differ → flagged
`warn-drift` even though that's the operator-expected state.

This task adds `--target-cache <PATH>` to scope the drift check to one
named cache file. When set: only that one row gets a drift verdict;
every other cache row keeps its T-1820 verdict (perms/format/orphan) and
is NOT classified as drift. The canonical PL-041 use case (single-hub
host, the operator knows which cache should mirror this hub's secret)
becomes the precise primitive.

## Acceptance Criteria

### Agent
- [x] CLI flag `--target-cache <PATH>` added; valid only paired with `--check-drift`; standalone use prints a clear error and exits 2
- [x] When `--target-cache` is set, drift comparison applies only to the named cache row (other rows get no drift verdict; their classifier call receives `None` for authoritative_hex)
- [x] When `--target-cache` is set but the named cache doesn't exist in the scan dir, audit prints a warning and exits 1 (operator typo'd a path) — `target_cache_missing` populated; JSON gains `target_cache_error` field
- [x] Existing `--check-drift` behavior unchanged when `--target-cache` is omitted (backward-compat: broad-mode still works) — covered by `secrets_audit_target_cache_broad_mode_unchanged_when_target_none` test
- [x] JSON envelope gains `target_cache` field (null when omitted, path string when set)
- [x] 3 new scanner-level tests: narrowing-applies-only-to-target, drift-on-target-when-differs, broad-mode-unchanged-when-none
- [x] `cargo check -p termlink` passes
- [x] `cargo test -p termlink secrets_audit` passes (11 existing + 3 new = 14/14)
- [x] Live-run on .107: `--check-drift /var/lib/termlink/hub.secret --target-cache /root/.termlink/secrets/laptop-141.hex` produces drift verdict ONLY on laptop-141.hex; ring20-management.hex/ring20-dashboard.hex now show plain `ok` (false-positive elimination confirmed)

### Human
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
cd /opt/termlink && cargo check -p termlink 2>&1 | tail -5
cd /opt/termlink && cargo test -p termlink secrets_audit 2>&1 | tail -10

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

## Recommendation

**GO** — ship as-is.

Closes the broad-mode false-positive problem surfaced in T-1822
Recommendation. The narrowing is path-canonicalized so symlinks and
`~/`-expansion work; non-target rows are passed `None` for authoritative,
which makes the classifier fall back to its pre-T-1822 verdict (perms /
format / orphan / ok). Backward compatible: omitting `--target-cache`
preserves T-1822 broad-mode behavior (covered by dedicated test).

**Live result on .107** with target-narrowed:

```
# authoritative: ok-mirror 0o600 /var/lib/termlink/hub.secret
# target-cache: /root/.termlink/secrets/laptop-141.hex
warn-drift   0o600 /root/.termlink/secrets/laptop-141.hex   [content differs]
info-orphan  0o600 /root/.termlink/secrets/proxmox4.hex     [not referenced]
ok           0o600 /root/.termlink/secrets/ring20-dashboard.hex
ok           0o600 /root/.termlink/secrets/ring20-management.hex
```

Compare to T-1822 broad-mode where ring20-* both showed `warn-drift`.
The narrowing turns 3 false positives into 1 true positive on the
operator-selected target. This is the precise PL-041-style "did THIS
cache rot vs THIS authoritative?" primitive the operator actually
wants for cron alerting.

**Follow-ups:**
1. MCP parity (`target_cache: Option<String>` in T-1821 params) — file as T-1825
2. Long-term: G-011 IP-keyed cache deprecation (still pending; structural)

## Updates

### 2026-05-28T08:13:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1824-fleet-secrets-audit---target-cache-path-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d1382651
- **Timestamp:** 2026-05-28T08:25:47Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#9 (Agent)** — Live-run on .107: `--check-drift /var/lib/termlink/hub.secret --target-cache /root/.termlink/secrets/laptop-141.hex` produces drift verdict ONLY on laptop-141.hex; ring20-management.hex/ring20-dashboa
  - **AC-verify-mismatch** (narrow, heuristic) — `path=var/lib/termlink/hub.secret in: Live-run on .107: `--check-drift /var/lib/termlink/hub.secret --target-cache /root/.termlink/secrets/laptop-141.hex` produces drift verdict ONLY on la`

### 2026-05-28T08:25:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
