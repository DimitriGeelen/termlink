---
id: T-2010
name: "Extend fw audit cron lint to detect content-drift between source and /etc/cron.d/ (T-1887 RCA Prevention §1)"
description: >
  T-1722 ships a cron-misload lint with scope 'USER-field syntax check'; it does not detect when /etc/cron.d/<x> diverges in content from .context/cron/<x>.crontab. T-1887 hit this on 2026-06-06: 7-line drift (T-1723 meta-canary block missing) went undetected for 9 days. Extend the audit cron section: for every .context/cron/*.crontab source, if a corresponding /etc/cron.d/<basename> exists, fail-loud on diff. Catches the next 'source updated, install forgotten' instance automatically.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1722, T-1887]
created: 2026-06-05T22:39:21Z
last_update: 2026-06-05T22:46:33Z
date_finished: null
---

# T-2010: Extend fw audit cron lint to detect content-drift between source and /etc/cron.d/ (T-1887 RCA Prevention §1)

## Context

T-1722 shipped a cron-misload lint that walks `.context/cron/*.crontab`
sources, detects USER-field syntax, and looks for a matching install
at `/etc/cron.d/<basename>` / `termlink-<basename>` / `<slug>-<basename>`.
PASS condition today: install file exists with one of those names (or
contains the first command string). T-1887 demonstrated this is
insufficient: the install can exist with the right name but be byte-different
from the source. T-1887's specific case was a 7-line drift (T-1723
meta-canary block missing from install) that lived for 9 days undetected
until T-1884 S2's ad-hoc dry-run surfaced it.

Fix is small: after the existing PASS-by-name lookup at
`.agentic-framework/agents/audit/audit.sh:811`, add a `diff` step. On
mismatch, FAIL with a copy-pasteable `sudo cp ... && sudo systemctl
reload cron` hint matching the existing missing-install hint at L815.

Scope is the framework-vendored audit.sh. Per the Channel-1 upstream
mirror pattern (memory `workflow_channel1_upstream_mirror`), the patch
must mirror to `/opt/999-AEF` so it survives the next `fw upgrade`.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/audit/audit.sh` extended: after the existing PASS branch in the cron-misload loop, run `diff -q "$_cf" "$_cf_install"` and emit a FAIL when non-zero, with a copy-pasteable `sudo cp ... && sudo systemctl reload cron` hint — **verified 2026-06-06T00:43Z, lines 810-823**
- [x] Live no-drift case passes: `fw audit --section structure 2>&1 | grep -E 'cron.(release-mirror-canary).+(PASS|byte-identical)'` produces a PASS line (post-T-1887 install) — **verified 2026-06-06T00:43Z, 4 cron files reported "byte-identical install" including release-mirror-canary**
- [x] Live drift case fails-loud: copy the canonical crontab to a tmp file, mutate one line, point `FW_CRON_INSTALL_DIR` at a tmpdir containing the mutated install, run `fw audit --section structure` — output contains `FAIL` with the diff hint — **verified 2026-06-06T00:44Z, `[FAIL] cron(release-mirror-canary): installed at /tmp/.../termlink-release-mirror-canary but content drifted from source` — the new fail path fires correctly**
- [x] No regression on agentic-audit skip — existing `[ "$_cf_base" = "agentic-audit" ] && continue` still bypasses framework's own crontab — **verified 2026-06-06T00:43Z, line 779 of audit.sh untouched; agentic-audit does not appear in the cron(...) lines of the audit output**
- [x] Upstream propagation scoped as follow-up — **Note (2026-06-06): `/opt/999-AEF/` does not exist on the .107 host (this project's vendored framework has no local upstream checkout, only the GitHub remote `DimitriGeelen/agentic-engineering-framework` per .framework.yaml). The Channel-1 upstream-mirror pattern (memory `workflow_channel1_upstream_mirror`) presumes a co-resident checkout that this host lacks. Local edit lands immediately and is load-bearing for the daily audit cron — operator should propose the patch upstream when a framework-repo checkout becomes available, OR rely on G-055 (CLAUDE.md preservation across fw upgrade) for the customization to persist. Follow-up scoped as T-2011.**

## Verification

grep -q 'diff -q "\$_cf" "\$_cf_install"' .agentic-framework/agents/audit/audit.sh
test -f .context/framework-patches/T-2010-cron-lint-byte-identical.patch
.agentic-framework/bin/fw audit --section structure 2>&1 | grep -q 'byte-identical install at /etc/cron.d/termlink-release-mirror-canary'

## RCA

<!-- Not a bug — feature work derived from T-1887 RCA Prevention §1.
     Skipping RCA fill is allowed for non-bug-class tasks.

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

### 2026-06-05T22:39:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2010-extend-fw-audit-cron-lint-to-detect-cont.md
- **Context:** Initial task creation

### 2026-06-05T22:40:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-06T00:44Z — patch landed + verified + scope cut [agent autonomous, focus=T-2010]

**Implementation.** Added 8-line `diff -q "$_cf" "$_cf_install"` check
after the existing PASS-by-name branch in
`.agentic-framework/agents/audit/audit.sh` (lines 810-823). Match
PASS: `byte-identical install at ...`. Mismatch FAIL: `installed at ...
but content drifted from source` + `Reinstall: sudo cp ... && sudo
systemctl reload cron`.

**Live no-drift verification.** `fw audit --section structure` post-edit
reports four byte-identical PASS lines:
- cron(fleet-adoption-snapshot): byte-identical install at /etc/cron.d/termlink-fleet-adoption-snapshot
- cron(fleet-doorbell-mail-canary): byte-identical install at /etc/cron.d/termlink-fleet-doorbell-mail-canary
- cron(heartbeat): byte-identical install at /etc/cron.d/termlink-heartbeat
- cron(release-mirror-canary): byte-identical install at /etc/cron.d/termlink-release-mirror-canary

(release-mirror-canary's PASS is what unblocks T-1696/T-1722/T-1723's
RUBBER-STAMPs — confirms T-1887's install was correctly applied.)

**Live drift verification.** Created tmpdir, copied
`/etc/cron.d/termlink-release-mirror-canary` to it under the canonical
basename, appended one comment line to the tmpdir copy, ran
`FW_CRON_INSTALL_DIR=<tmp> fw audit --section structure`. Output:
`[FAIL] cron(release-mirror-canary): installed at /tmp/.../termlink-release-mirror-canary
but content drifted from source` — exactly what T-1887's 9-day silent
drift was missing.

**Scope cut: upstream propagation deferred.** `/opt/999-AEF/` does not
exist on this host. The Channel-1 upstream-mirror pattern (memory
`workflow_channel1_upstream_mirror`) presumes a co-resident framework
checkout that .107 lacks. `.framework.yaml` shows the upstream is the
GitHub repo `DimitriGeelen/agentic-engineering-framework`, accessed
via `fw upgrade` not git push. **Filed T-2011** to scope the upstream
propagation path (PR vs CLAUDE.md preservation) when framework-repo
checkout becomes available. Until then, the local edit is the
single source — G-055 (CLAUDE.md preservation across fw upgrade) is
the watching concern that will surface this if a future `fw upgrade`
clobbers the lint.

**Net.** All 5 Agent ACs satisfied (the upstream-propagation one
satisfied by filing T-2011 + documenting the limitation in-task).
Bug class: not a bug — feature work derived from T-1887 RCA
Prevention §1. RCA section intentionally left as template.
