# T-233 Q2b: Template Caching & Learning Mechanism

## Question
How do agents cache learned templates from specialist interactions? Where stored, how versioned, how propagated?

## Design: Three-Layer Template Cache

### Layer 1: Agent-Local Cache (Per-Agent Directory)
Each agent gets a `.context/specialists/<agent-id>/templates/` directory containing YAML template files learned from specialist round-trips. Structure:

```
.context/specialists/
  coder-001/
    templates/
      fabric-card.yaml      # learned from fabric specialist
      task-create.yaml       # learned from task specialist
    manifest.yaml            # index: template → specialist, version, hit count
  auditor-002/
    templates/
      audit-report.yaml
    manifest.yaml
```

**Why per-agent, not shared-first:** Agents develop *usage-specific* template variants. A coder's fabric card template emphasizes `depends_on`; an auditor's emphasizes `interfaces`. Premature sharing flattens these into lowest-common-denominator templates.

### Layer 2: Shared Registry (Propagation)
When an agent's local template hits **5+ uses with 0 specialist corrections**, it's promoted to `.context/specialists/shared/templates/`. Promotion criteria:

- **Hit count ≥ 5** — enough usage to trust the pattern
- **Correction count = 0** since last specialist validation — no stale corrections pending
- **Specialist version match** — template was learned from the current specialist version

Shared templates are available to all agents as **read-only defaults**. An agent can override with a local variant (Layer 1 takes precedence).

### Layer 3: Specialist Canonical Templates
Specialists own the *canonical* template definitions in their context manifests (connects to Q4's "living brain"). A specialist's manifest includes:

```yaml
templates:
  fabric-card:
    version: 3
    schema_hash: abc123    # hash of required fields
    exemplar: |
      id: ...
```

This is the source of truth. Cached versions are derivatives.

## Versioning: Schema Hash + Monotonic Counter

Each specialist template has:
- **version** (integer, incremented on breaking changes)
- **schema_hash** (hash of required field names + types)

Cache invalidation rule: **on specialist interaction, compare cached `schema_hash` against specialist's current hash.** Mismatch → discard cached template, do full round-trip, cache new version. This is lazy invalidation — no polling, no push notifications. Staleness is detected at use time.

**Why not timestamps:** Clocks drift across sessions. Schema hashes are content-addressed and deterministic.

## Propagation: Pull-on-Miss, Not Push

Agents don't broadcast template updates. Instead:

1. Agent needs template → checks Layer 1 (local) → Layer 2 (shared) → Layer 3 (specialist round-trip)
2. On specialist round-trip, agent caches result at Layer 1
3. Layer 1 → Layer 2 promotion is automatic on threshold (5 uses, 0 corrections)
4. Layer 2 entries are invalidated when ANY agent discovers a version mismatch during a specialist interaction

**Why pull, not push:** Push requires a coordination mechanism (pub/sub, event bus). TermLink has events, but template updates are infrequent — the complexity isn't justified. Pull-on-miss is simple, correct, and self-healing.

## Connection to Q4 Context Manifest (Living Brain)

The specialist context manifest from Q4 IS the Layer 3 source of truth. The template library doesn't *become* the specialist context — it's a **materialized cache of one facet** of it. The manifest contains richer information (decision history, constraint rationale, common mistakes) that templates alone don't capture.

However, the template cache *informs* manifest evolution: if 80% of agents override a shared template the same way, that signal should flow back to the specialist as a manifest update proposal. This is the learning loop:

```
Specialist manifest → Agent caches template → Agent adapts locally →
Adaptation converges across agents → Signal to update manifest
```

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Storage location | Per-agent local + shared registry | Usage-specific variants matter |
| Invalidation | Lazy (schema hash check on use) | Simple, no coordination overhead |
| Propagation | Pull-on-miss | TermLink events overkill for infrequent updates |
| Promotion threshold | 5 uses, 0 corrections | Balance trust vs. speed |
| Manifest relationship | Cache is subset of manifest | Templates ≠ full specialist knowledge |

## Word Count: ~480
