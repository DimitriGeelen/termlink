---
id: T-1664
name: "episodic generator emits invalid YAML on backslash in AC text"
description: >
  episodic generator emits invalid YAML on backslash in AC text

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T18:40:58Z
last_update: 2026-05-17T18:40:58Z
date_finished: null
---

# T-1664: episodic generator emits invalid YAML on backslash in AC text

## Context

Discovered while closing T-1663 (MCP parity). `fw task update T-1663 --status work-completed` failed at episodic generation with `yaml.scanner.ScannerError: unknown escape character '|'` because an AC's verification evidence contained `grep "name1\|name2\|name3"` and the episodic writer quote-wrapped it without escaping the backslash. Per YAML 1.2, `\|` is not a valid escape inside a double-quoted scalar.

Bug in `.agentic-framework/agents/context/lib/episodic.sh:288`:
```bash
local text=$(echo "$line" | sed 's/^- //' | sed 's/"/\\"/g')
```
Escapes `"` but not `\`. Same pattern at line 304 (challenges section) and line 361 (updates section action field).

Fix: escape backslashes BEFORE quotes (order matters — escaping quotes first would double-escape the new `\"`):
```bash
local text=$(echo "$line" | sed 's/^- //' | sed 's/\\/\\\\/g' | sed 's/"/\\"/g')
```

The bug has likely lurked for any AC text containing literal backslashes (regex pipes, escape sequences in example commands). Fix lands in vendored copy + upstream framework via dispatch (Channel 1 mirror).

## Acceptance Criteria

### Agent
- [x] All three quote-escape sites in `.agentic-framework/agents/context/lib/episodic.sh` (lines 288, 304, 361) prepend a backslash-escape pass before the quote-escape pass. **Verified 2026-05-17:** sites 288 (outcomes), 304 (challenges), 361 (git_timeline) all now do `sed 's/\\/\\\\/g' | sed 's/"/\\"/g'`. Comment at site 288 documents the escape-order invariant for future editors.
- [ ] **DEFERRED to T-1665:** Same fix landed in upstream `/opt/999-AEF/agents/context/lib/episodic.sh` via `termlink dispatch`. Dispatch attempted (worker `t1664-upstream`, task ref T-1664) but exited with `FATAL: cd /opt/999-AEF failed` — that path is not accessible from this host's container. Upstream sync requires a different vector (manual SSH, or run from a host that mounts /opt/999-AEF). Filing T-1665 as the operator-actionable follow-up so consumer `fw upgrade` doesn't silently re-introduce the regression.
- [x] Regression smoke: regenerate T-1663's episodic via `.agentic-framework/agents/context/context.sh generate-episodic T-1663` and confirm `python3 -c "import yaml; yaml.safe_load(open('.context/episodic/T-1663.yaml'))"` exits 0 — proves the fix handles the original failure case. **Verified 2026-05-17:** regen succeeded, 5 outcomes loaded; line 32 now contains `\\|` (valid YAML) instead of the original `\|` that crashed the parser.
- [x] Synthetic regression: write a one-shot test where an AC text contains `\|`, `\\`, `\n` literals; generate episodic; YAML parses clean. **Verified 2026-05-17:** synthetic task fixture at `/tmp/synth-test/.tasks/completed/T-9999-synthetic.md` with three ACs (regex pipe `grep "a\|b\|c"`, double backslash `"C:\\foo\\bar"`, mixed `awk '/\|/'`); episodic generated; `yaml.safe_load` returned dict with 3 outcomes intact.

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
grep -q "sed 's/\\\\/" .agentic-framework/agents/context/lib/episodic.sh
python3 -c "import yaml; yaml.safe_load(open('.context/episodic/T-1663.yaml'))"

## RCA

**Symptom:** `fw task update T-1663 --status work-completed` exited 2 with `yaml.scanner.ScannerError: while scanning a double-quoted scalar … found unknown escape character '|'`. The task moved to `completed/` and the episodic file was generated but is unparseable, breaking downstream consumers (handover enricher, audit, search).

**Root cause:** `episodic.sh` quotes per-line content with `sed 's/"/\\"/g'` only — it escapes literal `"` to `\"` so it survives inside the surrounding double-quoted YAML scalar, but does NOT pre-escape literal `\`. When the input contains a backslash followed by anything other than a YAML-defined escape (here `\|` from a shell regex pipe), the emitted YAML is double-quoted with an invalid escape sequence.

**Why structurally allowed:** (1) The framework's own AC verification commands rarely contain backslashes, so this code path is mostly exercised on safe input. (2) The episodic generator has no per-task validation step — it appends lines to a file and exits 0 regardless of whether the resulting YAML parses. The yaml.scanner error surfaces only on the NEXT consumer load, which is `fw task update --status work-completed`'s post-write validate step. (3) There is no `python3 -c "yaml.safe_load(...)"` self-check inside episodic.sh.

**Prevention:** (a) Escape backslashes before quotes (this task's fix). (b) Add a self-validation step at the end of `generate_episodic` — pipe the file through `python3 -c "yaml.safe_load(sys.stdin.read())"` and bail loud if it fails, so the generator never silently writes broken YAML even if a future field-emit site misses escaping. (c) Capture as PL learning so the next person editing episodic.sh knows the escape-order invariant.

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

### 2026-05-17T18:40:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1664-episodic-generator-emits-invalid-yaml-on.md
- **Context:** Initial task creation
