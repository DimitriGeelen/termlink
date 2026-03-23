# T-233 Q2b: Pre-Execution Routing Decision

## The Decision Point

Before dispatching work, the agent faces a three-way branch: (1) execute locally, (2) use a cached specialist route, or (3) consult the orchestrator. This report designs that decision mechanism.

## Local Route Cache

**Format:** YAML files in `.cache/routes/` keyed by capability slug.

```yaml
# .cache/routes/commit-conventional.yaml
capability: commit-conventional
specialist: git-specialist
confidence: 0.95
request_schema:
  required: [files, message_type, scope]
  optional: [breaking, body]
learned_from: orchestrator    # or "builtin"
last_used: 2026-03-22T14:30:00Z
hit_count: 7
ttl_hours: 168               # re-validate after 7 days
```

**Why YAML over skill files:** Route cache entries are metadata about *where* to send work, not executable logic. Skill files carry prompt templates and execution semantics. Conflating routing with execution creates coupling — a route change shouldn't require editing a skill.

## Decision Algorithm

```
1. EXTRACT capability slug from intent
   - File patterns → "dockerfile-write", "test-rust"
   - Tool patterns → "cargo-test", "docker-build"
   - Task metadata → workflow_type + tags

2. LOOKUP in .cache/routes/{slug}.yaml
   ├─ HIT + not expired + confidence >= 0.8
   │    → Use cached route (skip orchestrator)
   ├─ HIT + expired OR confidence < 0.8
   │    → Background re-validate with orchestrator, use cache optimistically
   ├─ PARTIAL MATCH (prefix match, e.g., "commit" matches "commit-conventional")
   │    → Consult orchestrator with hint: "I know about {cached_slug}, is this the same?"
   └─ MISS
        → Consult orchestrator, cache the response

3. LOCAL EXECUTION gate (checked BEFORE cache lookup)
   - Agent has the skill locally (e.g., builtin git commit, file read)
   - Domain score < 2 (from Q2 trigger taxonomy: no strong specialist signal)
   - Task is trivially scoped (single file, < 50 lines changed)
   → Execute locally, no delegation
```

## Partial Match Handling

The hardest case: "I know about commits but not THIS kind of commit." Three strategies:

1. **Prefix matching** — `commit-*` cached routes form a family. If `commit-conventional` is cached but the intent is `commit-fixup`, the agent sends the orchestrator a *refinement query*: "I have a route for `commit-conventional`. Does `commit-fixup` use the same specialist?" This avoids a full discovery round-trip — the orchestrator returns either "yes, same route" or a new route entry.

2. **Confidence decay** — Each cache entry tracks `hit_count` and `last_used`. Entries degrade: confidence drops 0.05 per week of non-use. Below 0.5, the entry is treated as a miss. This prevents stale routes from persisting after specialists are reconfigured.

3. **Schema validation** — If the cached `request_schema` doesn't match the current request's fields, treat as partial miss. Example: cached route for `commit-conventional` requires `[files, message_type]`, but the current intent includes `[rebase_onto]` — schema mismatch triggers orchestrator consultation.

## Cache Lifecycle

| Event | Action |
|-------|--------|
| Orchestrator returns route | Write to `.cache/routes/`, set `learned_from: orchestrator` |
| Successful dispatch via cache | Increment `hit_count`, update `last_used` |
| Specialist rejects request | Invalidate cache entry, re-consult orchestrator |
| TTL expires | Background re-validate on next use |
| Agent restart | Cache persists on disk (survives sessions) |

## Integration with Existing Architecture

- **Domain triggers (Q2)** feed Step 1 — they produce the capability slug and confidence score
- **Specialist watcher** is the target of cached routes — the `specialist` field maps to a TermLink session tag
- **dispatch.sh** gains a `--cached` flag: skips orchestrator event round-trip, sends directly to the named specialist session
- **Orchestrator** gains a `/route` RPC: returns specialist + schema for a capability, without full task dispatch

## Key Design Decision

Cache is *routing metadata only*, never prompt templates or execution logic. The agent caches "commit-conventional goes to git-specialist with schema X" but the actual prompt construction still uses the specialist's own template. This means specialist upgrades don't require cache invalidation — only routing changes do.

**Word count:** ~490
