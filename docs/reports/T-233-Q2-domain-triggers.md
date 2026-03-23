# T-233 Q2: Domain-Specific Trigger Design

## Mechanism

Domain-specific triggers automatically route work to specialists based on signals detected in the user's request or the agent's working context. The orchestrator doesn't need to be told "use the infra specialist" — it recognizes infra-relevant signals and routes accordingly.

## Trigger Taxonomy

Four signal types, ordered by reliability:

### 1. File-Pattern Triggers (highest confidence)
File paths being read/written carry strong domain signal:
- `Dockerfile`, `docker-compose.yaml`, `*.tf`, `k8s/` → **infra-specialist**
- `*.css`, `*.tsx`, `*.vue`, `src/components/` → **frontend-specialist**
- `*.test.*`, `*.spec.*`, `tests/`, `cypress/` → **test-specialist**
- `.github/workflows/`, `Jenkinsfile`, `.gitlab-ci.yml` → **ci-specialist**

**Why highest confidence:** File edits are concrete actions, not ambiguous language. A Write to `Dockerfile` is unambiguously infrastructure work.

### 2. Tool/Command Triggers (high confidence)
CLI tools and commands invoked via Bash:
- `docker`, `terraform`, `kubectl`, `helm` → **infra-specialist**
- `npm`, `webpack`, `vite`, `eslint` → **frontend-specialist**
- `cargo test`, `pytest`, `jest` → **test-specialist**
- `psql`, `redis-cli`, `mongosh` → **data-specialist**

### 3. Keyword Triggers (medium confidence)
Natural language in the user's message or task description:
- "deploy", "scale", "container", "cluster" → **infra-specialist**
- "layout", "responsive", "animation", "component" → **frontend-specialist**
- "coverage", "regression", "assert", "mock" → **test-specialist**

**Caution:** Keywords alone are ambiguous. "Test the deployment" touches both testing and infra domains.

### 4. Context Triggers (supplementary)
Derived from framework state, not user input:
- Task tags (`tags: [infra]`) → route to matching specialist
- Task workflow_type (`test`) → **test-specialist**
- Active file in editor (LSP integration) → infer domain from file type
- Recent git diff (changed files imply domain)

## Multi-Domain Conflict Resolution

When multiple domains match (common case), use a **scoring model**:

1. **Score each domain** by summing trigger weights: file-pattern=3, tool=2, keyword=1, context=1
2. **Primary domain** gets the delegation. Secondary domains are noted in the request context.
3. **Tie-breaking rule:** Prefer the domain matching the _action_ (what's being changed) over the domain matching the _subject_ (what's being discussed). E.g., "write a Dockerfile for the React app" → infra (action=Dockerfile), not frontend (subject=React).
4. **Explicit override:** User can always force routing via `@infra` mention syntax (see Q2-interactive report).

## False Positive Handling

Three defenses:

1. **Confidence threshold** — Single keyword match alone never triggers delegation. Require score >= 2 (e.g., keyword + file pattern, or two keywords from same domain).
2. **Negative signals** — Some patterns suppress delegation: "explain Docker" (read-only, no specialist needed), "delete the old test" (cleanup, not testing work).
3. **Soft routing** — Below threshold, the orchestrator _suggests_ a specialist rather than auto-delegating: "This looks like infra work. Route to infra-specialist? [Y/n]"

## Configuration Format

Trigger rules should be user-configurable via a YAML manifest:

```yaml
# .claude/specialists/triggers.yaml
domains:
  infra:
    file_patterns: ["Dockerfile", "*.tf", "k8s/**"]
    commands: ["docker", "terraform", "kubectl"]
    keywords: ["deploy", "container", "cluster"]
    threshold: 2
  frontend:
    file_patterns: ["*.tsx", "*.css", "src/components/**"]
    commands: ["npm", "vite", "eslint"]
    keywords: ["layout", "responsive", "component"]
    threshold: 2
```

Projects add/remove domains and tune thresholds to match their stack. A Rust-only project would have no frontend domain. A full-stack project might lower the threshold for common domains.

## Implementation Sketch

The trigger engine runs as a **pre-processing step** in the orchestrator, before the agent begins work:

1. Parse user message → extract keywords
2. Check PreToolUse context → extract file paths and commands (if available)
3. Score each configured domain
4. If max score >= threshold → auto-delegate (or suggest, based on config)
5. Log the trigger match for auditability

This is stateless per-request — no learning or adaptation needed in v1.

## Key Design Decision

**Triggers are a _routing hint_, not a commitment.** The specialist can decline ("this isn't actually infra work") and bounce back to the orchestrator. This keeps false positives recoverable without complex pre-validation.
