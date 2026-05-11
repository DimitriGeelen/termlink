---
id: T-1631
name: "Fix G-082 upstream — episodic generator emits invalid YAML on tasks with ## Decisions"
description: >
  Fix G-082 upstream — episodic generator emits invalid YAML on tasks with ## Decisions

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-11T11:07:29Z
last_update: 2026-05-11T11:10:01Z
date_finished: null
---

# T-1631: Fix G-082 upstream — episodic generator emits invalid YAML on tasks with ## Decisions

## Context

Bug filed by `ring20-management-agent` on the `framework:pickup` topic at 2026-05-05 (envelope `ring20-G-082-2026-05-05`). The upstream framework's `agents/context/context.sh generate-episodic` emits malformed YAML whenever a source task has any content under `## Decisions`: it drops the `### [date] — [topic]` heading, writes the first decision as a flat indent-4 mapping under `decisions:`, then always appends a `- decision: '[date] — [topic]'` template placeholder. The placeholder starts a block sequence that collides with the preceding mapping, so `yaml.safe_load` fails.

Affects every consumer project running `fw task update --status work-completed` on tasks with decisions. Local mitigation in source project (proxmox-ring20-management): 3 broken files repaired by hand at commit `9c25409c`. The peer filed it upstream because every project will keep regenerating broken files until the generator itself is fixed.

Fix happens on `/opt/999-AEF` (upstream framework). This task tracks the coordination from `/opt/termlink`; the actual edit is dispatched to upstream via the Channel-1 mirror pattern.

## Acceptance Criteria

### Agent
- [x] Bug reproduced locally — symptom mutated from peer's report (current vendored copy no longer emits the trailing placeholder; instead silently merges duplicate keys). See `Updates 2026-05-11T11:14Z`. Real root cause: `lib/episodic.sh:125` `grep -v '^##'` greedily consumed `### ` H3 headings.
- [x] Fix written in `/opt/999-Agentic-Engineering-Framework/agents/context/lib/episodic.sh` — `grep -v '^##'` → `grep -v '^## '` (with trailing space) preserves H3 headings; downstream `^### ` handler now fires correctly. Upstream commit `7dedefca7` on `master` (note: actual path is `/opt/999-Agentic-Engineering-Framework`, my prior memory said `/opt/999-AEF` which was a stale alias).
- [x] Post-generation YAML validation step added — `do_generate_episodic` now runs `python3 -c "import yaml,sys; yaml.safe_load(open(sys.argv[1]))" "$episodic_file"` after writing; exits 2 with operator-facing guidance if invalid. Upstream commit `7dedefca7`.
- [x] Test: synthetic task with 1 decision → episodic valid YAML, `decisions` is list of 1 mapping with proper chose/rationale/alternatives_rejected. Sandbox `/tmp/g082-sandbox/.context/episodic/T-G082-2.yaml`.
- [x] Test: synthetic task with 2 decisions → episodic valid YAML, `decisions` is list of 2 mappings each preserving its own fields (pre-fix: silently merged into single mapping). Sandbox `T-G082-1.yaml`.
- [x] Test: synthetic task with 0 decisions → episodic valid YAML, `decisions: null` (no spurious data, no regression). Sandbox `T-G082-0.yaml`.
- [x] Patch committed and pushed to upstream onedev remote — `7dedefca7` on `agentic-engineering-framework.git` (push range `3b5c566a8..7dedefca7  master -> master`).
- [x] Local `/opt/termlink/.agentic-framework/agents/context/lib/episodic.sh` synced — vendored copy was edited first to verify the fix works, then mirrored upstream. Vendored sha and upstream sha now contain identical T-1631/G-082 markers.
- [x] Reply envelope posted on `framework:pickup` referencing `pickup_id=ring20-G-082-2026-05-05` — local hub offset 8, .122 hub offset 2. `msg_type=pickup-bug-fixed`, includes upstream sha `7dedefca7`, root-cause clarification (symptom mutation), and consumer-action steps (`git pull origin master` + regenerate corrupted episodics).

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

## RCA

**Symptom:** `yaml.safe_load` fails on `.context/episodic/T-XXX.yaml` for every completed task that had any `## Decisions` content. Framework can't index those tasks' decisions; downstream tooling (audit, healing, learnings auto-promotion) skips them silently. Reported on 3 files in proxmox-ring20-management (T-597, T-635, T-653) before the peer filed upstream.

**Root cause:** `generate-episodic` in `agents/context/context.sh` has a hardcoded template tail that emits a `- decision: '[date] — [topic]'` list-item placeholder regardless of whether real decisions were extracted from the source task. When real decisions exist, the function writes them as a flat indent-4 mapping under `decisions:` (no leading `-`), then appends the placeholder list-item. The resulting YAML mixes a block mapping and a block sequence under the same key → parse error at the collision line.

**Why structurally allowed:** Three independent gaps stacked:
1. `generate-episodic` has no output-validation step. It writes the file and exits 0 even if the file is unparseable.
2. The episodic parser elsewhere in the framework (`PL-037`, T-1150) was already tolerant of legacy formats. That tolerance was a *consumer-side* mitigation; it never alerted that the *generator* was producing invalid output.
3. No upstream consumer-project test exercised `generate-episodic` against tasks with realistic `## Decisions` shape — the framework's own test corpus uses simpler tasks.

**Prevention:** Two-layer defence —
(a) add `yaml.safe_load` post-step inside `generate-episodic` so the generator fails loudly the next time the template diverges from real-data emission — **applied in upstream commit `7dedefca7`**;
(b) register a learning that regex-based content filters in shell generators must specify trailing context (use `^## ` not `^##`) when adjacent header levels share a prefix, AND that yaml.safe_load tolerates duplicate keys via silent merge — meaning "the file parsed" is NOT sufficient validation for structured emission, the emitted SHAPE must also match expectation.

**Post-fix RCA correction (2026-05-11T11:23Z):** My initial RCA in this task body (commit `fc55b2eb`) attributed the bug to a "trailing template placeholder list-item collision" based on the peer's report. Reproduction in the current vendored code revealed the placeholder was already removed in an earlier framework commit — the actual remaining bug was the greedy `^##` regex stripping `### ` H3 lines, producing silently-merged decisions. Both my initial RCA and the peer's original report described an OLDER compound-bug shape that's no longer the failure mode. The fix-reply on `framework:pickup` includes this correction so any downstream agent reading the chain has the accurate post-fix RCA.

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

## Updates

### 2026-05-11T11:07:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1631-fix-g-082-upstream--episodic-generator-e.md
- **Context:** Initial task creation

### 2026-05-11T11:14Z — bug reproduced + real root cause identified [agent]
- **Action:** Built sandbox at `/tmp/g082-sandbox/` with 3 synthetic tasks (0/1/2 decisions). Ran `bash agents/context/context.sh generate-episodic` against each.
- **Finding:** Current vendored copy does NOT match peer's reported failure mode. yaml.safe_load passes — but `decisions` is a flat mapping where the SECOND decision's fields silently clobbered the FIRST. The `### date — topic` headings were completely absent from the output, even though the `^### ` handler at line 315 of episodic.sh should have fired.
- **Root cause:** line 125 of `lib/episodic.sh` filters `decisions_section` with `grep -v '^##' | grep -v '^<!--' | grep -v '^-->' | grep -v '^\s*$'`. The first pattern `^##` greedily matches `###` lines too, stripping every H3 heading before the parser sees it. Peer's older failure mode (yaml.safe_load fail at line 9/44) was a compound bug — the trailing template placeholder is already removed in current code, so the symptom mutated from parse-fail to silent-merge.

### 2026-05-11T11:21Z — fix applied + verified locally + mirrored upstream [agent]
- **Vendored copy:** changed `grep -v '^##'` to `grep -v '^## '` (with trailing space) at line 125; added post-generation `yaml.safe_load` validation block before the final success echo (with operator-facing guidance on failure).
- **Verification:** Re-ran all 3 synthetic tasks. T-G082-0 (0 decisions) → `decisions: null`. T-G082-1 (2 decisions) → list of 2 mappings, each preserving chose/rationale/alternatives_rejected. T-G082-2 (1 decision) → list of 1. All yaml.safe_load valid with correct shape.
- **Upstream mirror:** `termlink_run` (project-boundary-aware execution path) — `cd /opt/999-Agentic-Engineering-Framework && python3 /tmp/g082-upstream-patch.py && git add agents/context/lib/episodic.sh && git commit && git push origin master`. Push range `3b5c566a8..7dedefca7  master -> master`. Note: actual upstream path is `/opt/999-Agentic-Engineering-Framework`, NOT `/opt/999-AEF` as a stale memory entry suggested — corrected the patch script to probe both.

### 2026-05-11T11:23Z — fix-reply posted on framework:pickup at both hubs [agent]
- **Local hub (workstation-107):** offset 8, `msg_type=pickup-bug-fixed`, refs `pickup_id=ring20-G-082-2026-05-05`, includes upstream sha `7dedefca7`, root-cause clarification (symptom mutation), and consumer-action steps.
- **.122 hub (ring20-management):** offset 2.
- **Consumer action:** projects can `git pull origin master` in their `.agentic-framework/` (or wait for next `fw upgrade`) and then regenerate any episodics that had silent merge (e.g. proxmox-ring20-management's T-597, T-635, T-653).

