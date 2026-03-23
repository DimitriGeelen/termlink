# T-233 Q2: Evaluation-Based Specialist Discovery

## Research Question
How could an orchestrator discover WHAT to delegate by evaluating/parsing the current task context?

## Approach
The orchestrator reads the task file (frontmatter + body), acceptance criteria, and related code to automatically decompose work into specialist-routable units.

## Signals in Task Files That Indicate Domain

**Frontmatter signals:**
- `workflow_type` → direct mapping: `build`→coder, `test`→tester, `design`→designer, `inception`→researcher
- `tags` → domain hints: `["ui", "api"]` suggests frontend + backend specialists
- `components` → file-level routing: component paths map to subsystems via `.fabric/subsystems.yaml`

**Body signals:**
- `## Context` references to docs, APIs, or infrastructure → research or infra specialist
- Keywords in description: "refactor" → coder, "investigate" → researcher, "deploy" → infra
- Links to external systems (URLs, server references) → infra specialist

**AC signals (richest source):**
- Each `- [ ]` checkbox is a potential work unit
- AC text contains action verbs that signal domain: "implement" → coder, "verify" → tester, "design" → designer, "research" → researcher
- `### Agent` vs `### Human` split already partitions machine-verifiable from human-verifiable work
- `## Verification` commands reveal what toolchains are involved (cargo → Rust coder, curl → API/infra)

## Parsing ACs Into Specialist-Routable Units

**Algorithm:**
1. Parse frontmatter YAML for `workflow_type`, `tags`, `components`
2. Extract each AC checkbox as a candidate work unit
3. Classify each AC by keyword matching + component mapping:
   - File paths in AC text → look up subsystem in fabric → route to domain specialist
   - Action verbs → map to specialist type (implement/fix/refactor → coder, test/verify → tester)
   - Infrastructure nouns (server, deploy, hub, socket) → infra specialist
4. Group adjacent ACs that share the same domain into a single dispatch unit
5. Identify cross-domain ACs (e.g., "implement API endpoint and write tests") → split or assign to primary domain with secondary review

**Output:** A dispatch manifest — list of `{specialist, work_items[], context_refs[]}` tuples.

## Multi-Domain Tasks

Three strategies:
1. **Split dispatch:** Route each AC group to its specialist independently. Works when ACs are naturally independent.
2. **Sequential pipeline:** Primary specialist produces artifact, secondary specialist consumes it (e.g., coder writes code → tester writes tests). Requires ordering logic.
3. **Orchestrator synthesis:** For tightly coupled multi-domain ACs, the orchestrator keeps coordination responsibility and dispatches narrowly scoped sub-questions to specialists.

**Detection heuristic:** If >60% of ACs map to one domain, it's single-domain with auxiliary work. If no domain exceeds 40%, it's genuinely multi-domain and likely needs decomposition into separate tasks first.

## Practical Considerations

- **False positives:** Keyword matching is brittle. "Test the deployment" could route to tester OR infra. Mitigation: use component fabric as ground truth, keywords as fallback.
- **Task granularity matters:** Well-sized tasks (one deliverable per task sizing rules) are easier to route. Oversized tasks produce noisy multi-domain classifications — this is itself a signal to decompose.
- **Bootstrap cost:** Requires a classification vocabulary (verb→domain, component→specialist mappings). This could live in a YAML config checked into the project.
- **Complements interactive discovery (Q2-interactive):** Evaluation handles routine routing; interactive handles ambiguous cases where the human overrides classification.
