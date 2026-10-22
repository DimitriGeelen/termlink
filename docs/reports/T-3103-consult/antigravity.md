## Verdict

**Option A is directionally right in splitting the two cases, but its diagram proposal is a disguised rule violation.** Skipping `workflow-step` cards in the coverage check merely shifts a hand-shaped exemption from `watch-patterns.yaml` into check logic, leaving 8 diagram files watched by nothing when a 9th workflow or a `v(N+1).bpmn` arrives on re-vendor. For `.claude/commands/*.md`, widening to all 34 files is the right call.

---

## Answers (1–4)

### 1. Principled exemption vs. hand-shaped silencing
Skipping `workflow-step` in the coverage check is **hand-shaped silencing under another name**. While a `workflow-step` card maps a sub-file entity rather than a whole file, its `location` still asserts that the underlying `.bpmn` file is load-bearing.

**The separating test:** *If a new sibling artifact of the same kind is added tomorrow (e.g., a 9th workflow diagram or a new version `vN+1.bpmn` on re-vendor), will ANY automated check complain that it lacks cards?*
- Under Option A's skip rule, the existing step↔diagram guard only checks links for cards that *already exist* or diagrams already referenced. A brand-new 9th workflow or un-adopted `vN+1.bpmn` in `<vendored>/designer/projects/` will be silently ignored by both drift and coverage. A principled exemption delegates drift detection to another complete boundary check; a hand-shaped exemption leaves the category unmonitored for new additions.

### 2. Better options for the diagrams
Yes—two cleaner mechanisms avoid both carding dead `v1..v(N-1)` files and blinding drift detection:
1. **Project-directory boundary or stable alias:** Watch `<vendored>/designer/projects/*/` at the *workflow directory* level (or maintain a non-vendored manifest/symlink directory of active workflows in your own tree) so every workflow project folder must be accounted for, and the step↔diagram guard enforces that the carded file inside each project folder is the highest `vN.bpmn`.
2. **First-class version-aware selector in the step guard:** Exempt `.bpmn` from file-level `watch-patterns.yaml` *only if* the step↔diagram guard itself scans `<vendored>/designer/projects/*/` and fails when (a) a project directory has no cards, or (b) cards point to a non-latest `vN.bpmn`.

### 3. Carding all 34 command files vs. singleton card
**Card all 34 command files.** Operator slash commands are executable entrypoints that invoke scripts, CLI tools, and repo paths. If `capture.md` depends on a transcript-reader script, other commands almost certainly carry implicit dependencies (or will acquire them). Auto-generating 33 lightweight cards from their markdown headers costs minutes once, honours the natural-boundary rule, and ensures future commands cannot silently depend on scripts without being registered. A singleton card directly violates the anti-gaming rule and leaves coverage permanently noisy or hacked.

### 4. What breaks at the next re-vendor
- **Option A:** If a workflow bumps from `v2.bpmn` to `v3.bpmn` and keeps `v2.bpmn` alongside it, existing cards still point to `v2.bpmn` (stale) unless the step guard checks for newer versions; if `v2.bpmn` is deleted, the step guard fails; if a 9th workflow is added, **nothing fires** (silent drift).
- **Option B:** Every re-vendor that adds `v(N+1).bpmn` forces dummy cards for superseded versions, polluting impact analysis with dead files.
- **Option C / D:** Coverage check stays permanently broken (cry-wolf fatigue), hiding real coverage regressions elsewhere.

---

## Risks

- **Silent version staleness:** Re-vendoring replaces the folder wholesale every two months; if old `vN.bpmn` files remain in the tarball alongside `v(N+1).bpmn`, path-existence checks won't catch that cards point to superseded diagrams.
- **Card bloat vs. signal:** Mechanical header-scraped cards for 33 commands that have `depends_on: []` can become rubber-stamp noise if authors don't record script/path dependencies when editing commands.

---

## Recommendation

**Adopt Modified Option A:** Widen `.claude/commands/*.md` and card all 34 commands, and exempt `workflow-step` from file-level coverage **only after** upgrading the dedicated step↔diagram guard to scan `<vendored>/designer/projects/*/` and assert that *every* workflow folder has cards pointing to its highest `vN.bpmn`.

**Single most important reason:** A coverage exemption is only honest if another automated check enforces completeness across the natural boundary (`projects/*/latest`); otherwise, skipping `workflow-step` hides missing workflows and stale versions on every two-month re-vendor.
