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
last_update: 2026-05-11T11:07:29Z
date_finished: null
---

# T-1631: Fix G-082 upstream — episodic generator emits invalid YAML on tasks with ## Decisions

## Context

Bug filed by `ring20-management-agent` on the `framework:pickup` topic at 2026-05-05 (envelope `ring20-G-082-2026-05-05`). The upstream framework's `agents/context/context.sh generate-episodic` emits malformed YAML whenever a source task has any content under `## Decisions`: it drops the `### [date] — [topic]` heading, writes the first decision as a flat indent-4 mapping under `decisions:`, then always appends a `- decision: '[date] — [topic]'` template placeholder. The placeholder starts a block sequence that collides with the preceding mapping, so `yaml.safe_load` fails.

Affects every consumer project running `fw task update --status work-completed` on tasks with decisions. Local mitigation in source project (proxmox-ring20-management): 3 broken files repaired by hand at commit `9c25409c`. The peer filed it upstream because every project will keep regenerating broken files until the generator itself is fixed.

Fix happens on `/opt/999-AEF` (upstream framework). This task tracks the coordination from `/opt/termlink`; the actual edit is dispatched to upstream via the Channel-1 mirror pattern.

## Acceptance Criteria

### Agent
- [ ] Bug reproduced locally: create a synthetic task with two `### [date] — [topic]` entries under `## Decisions`, mark `work-completed`, confirm the resulting episodic fails `yaml.safe_load`
- [ ] Fix written in `/opt/999-AEF/agents/context/context.sh` — `generate-episodic` extracts date+topic from `### [date] — [topic]` headers, emits each decision as a `- decision:` list item under `decisions:`, and skips the template placeholder when real decisions are present
- [ ] Post-generation YAML validation step added — `generate-episodic` runs `python3 -c "import yaml; yaml.safe_load(open(...))"` against its own output and exits non-zero if the file is invalid (prevention path, per RCA)
- [ ] Test: synthetic task with 1 decision → episodic valid YAML
- [ ] Test: synthetic task with 2 decisions → episodic valid YAML
- [ ] Test: synthetic task with 0 decisions (template placeholder only) → episodic still valid (no regression)
- [ ] Patch committed and pushed to upstream `/opt/999-AEF` onedev remote
- [ ] Local `/opt/termlink/.agentic-framework/agents/context/context.sh` synced from upstream (so this project benefits without waiting for next `fw upgrade`)
- [ ] Reply envelope posted on `framework:pickup` referencing `pickup_id=ring20-G-082-2026-05-05` with `status=fixed`, the upstream commit sha, and a short note that consumer projects can pull-and-re-run

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
(a) add `yaml.safe_load` post-step inside `generate-episodic` so the generator fails loudly the next time the template diverges from real-data emission;
(b) register a learning that template-tail emissions in shell-scripted generators must be conditional on real-data emptiness — this generalises beyond `decisions:` to any future section that has both a real-data and placeholder path.

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
