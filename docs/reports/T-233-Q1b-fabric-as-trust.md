# T-233 Q1b: Component Fabric as Trust Ledger

**Question:** Can fabric component cards be extended into trust cards for specialist script supervision?

## Current Fabric Capabilities

The Component Fabric (`.fabric/`) provides structural topology: 50+ component cards with `id`, `type`, `subsystem`, `depends_on`, `depended_by`, `purpose`, `tags`, and `last_verified`. The `fw fabric blast-radius` command already computes transitive downstream impact for commits by walking the dependency graph. Subsystem mapping (`subsystems.yaml`) groups components into protocol → session → hub → cli → agent-mesh layers.

Key observation: the fabric is **static topology** — it describes what depends on what, not how things behave at runtime. Blast-radius is a structural prediction ("if you change X, Y and Z might break"), not a runtime observation.

## Trust Card Design: What Would Be Added

A trust card for a specialist script (e.g., `dispatch.sh`, `agent-wrapper.sh`) would extend the existing card with:

| Field | Source | Nature |
|-------|--------|--------|
| `risk_class` | Blast-radius analysis (existing) | Static, derivable |
| `known_failure_modes` | Healing patterns (`patterns.yaml`) | Accumulated, cross-project |
| `context_history` | Per-project run/fail counts | Runtime, project-scoped |
| `maturity_score` | Computed from failure diversity handled | Derived from context_history |

## Feasibility Assessment

**risk_class — YES, natural fit.** The fabric already computes blast-radius per commit. A script's `depended_by` count and subsystem position directly yield a risk class. A `dispatch.sh` that touches hub+CLI+session is high-risk; a leaf reporting script is low-risk. This is a one-time derivation from existing topology, no new infrastructure needed.

**known_failure_modes — YES, with cross-referencing.** The healing agent already stores failure patterns in `patterns.yaml` with `origin_task` references. Cross-referencing a script's task history against failure patterns is mechanical. The gap: patterns today are not tagged to specific scripts, only tasks. Adding a `component:` field to failure patterns would close this.

**context_history — PARTIAL fit.** The fabric is file-based YAML, not a runtime database. Per-project run/fail counts require a new accumulation mechanism. Options: (a) a `trust/` directory alongside `components/` with per-script YAML tracking runs, or (b) a `context_history` section in the existing card. The portability requirement (maturity carries, context resets) maps naturally: on project promotion, copy the card but zero out `context_history` while preserving `maturity_score` and `known_failure_modes`.

**maturity_score — YES, computable.** Formula: weight failure diversity (distinct failure classes survived) more than raw run count. A script that has hit and recovered from 5 different failure modes is more mature than one with 100 clean runs. This mirrors the framework's antifragility directive — systems that have been stressed and healed are stronger.

## Supervision Level Computation

Just as `blast-radius` walks `depended_by` to compute impact, a `supervision-level` command could combine:

```
supervision = f(risk_class, inverse(maturity_score), context_familiarity)
```

- **High risk + low maturity + new context** → full supervision (human approval per action)
- **Low risk + high maturity + familiar context** → autonomous execution
- **Any risk + known failure mode active** → targeted supervision (watch for that specific class)

This fits the existing `fw fabric` CLI pattern: `fw fabric supervision <script>` would output a recommendation using the same traversal infrastructure.

## Architectural Fit

**Strengths:** Reuses existing infrastructure (cards, blast-radius traversal, healing patterns). The fabric is already the structural source of truth — adding trust metadata keeps it as the single registry. The static/runtime split (topology vs. context_history) mirrors the existing static/verified split (depends_on vs. last_verified).

**Concerns:** The fabric is currently pure topology (no runtime state). Adding `context_history` introduces a stateful dimension. This could be mitigated by keeping runtime counters in a separate `trust/` directory that references component IDs, preserving the fabric's structural purity.

## Recommendation

Extend fabric cards with `risk_class` and `known_failure_modes` (static/accumulated). Keep `context_history` in a separate `trust/` overlay keyed by component ID. Compute `maturity_score` and `supervision_level` as derived values, not stored — same pattern as blast-radius (computed on demand from graph + history).
