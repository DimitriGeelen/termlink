# T-233 Q1b: Component Fabric Evidence Assessment

**Question:** Is the Component Fabric (.fabric/) actually working in practice?

## Summary Verdict

**The fabric is actively maintained and growing, but blast-radius/drift tools show no evidence of routine use.** Cards are created diligently at task boundaries but dependency analysis appears unused in practice.

## Evidence

### 1. Card Coverage: STRONG

- **65 component cards** in `.fabric/components/`
- **4 subsystems** defined in `subsystems.yaml` (protocol, session, hub/tooling, cli)
- **Zero missing cards** for CLI command modules or session modules — full coverage verified
- Cards registered incrementally across 15+ tasks (T-043, T-051, T-054, T-055, T-077, T-083, T-092, T-105, T-109, T-112, T-113, T-115, T-116, T-129, T-182, T-206, T-223)

### 2. Dependency Quality: GOOD

- **All 65 cards** have `depends_on` fields populated
- **21 cards** have `depended_by` fields populated (32% — reverse links less maintained)
- Sample card (pty.rs) shows typed dependencies with target+type format — structurally correct
- Dependencies reference actual crate boundaries (cli→session→protocol) matching real architecture

### 3. Freshness: MODERATE CONCERN

Last-verified date distribution:
| Date | Cards | Age (days) |
|------|-------|------------|
| 2026-03-21 | 15 | 2 |
| 2026-03-14–19 | 11 | 4–9 |
| 2026-03-08–12 | 39 | 11–15 |

**39 of 65 cards** (60%) last verified 11–15 days ago. Source files have been modified since (14 files newer than the remote commands card from 2026-03-21). Cards aren't being re-verified when source changes — only when new cards are registered.

### 4. Blast-Radius / Drift Usage: WEAK

- **1 commit** mentions blast-radius (T-043 initial setup only)
- **0 commits** mention "drift"
- **0 task files** reference `fw fabric blast-radius` as a verification step
- CLAUDE.md references blast-radius 4 times (in workflow rules), but git history shows it's not being followed in practice
- No evidence of drift detection catching real issues

### 5. Registration Discipline: STRONG

- Cards are consistently created when new source files are added (T-206 registered 14 cards at once for CLI refactor)
- TLS module (T-182), test-utils (T-092), mesh scripts (T-115, T-116) all got cards
- The "register after creation" rule from CLAUDE.md is being followed

## Assessment

| Dimension | Rating | Evidence |
|-----------|--------|----------|
| Card coverage | ★★★★★ | 65 cards, zero gaps in source modules |
| Dependency modeling | ★★★★☆ | Forward deps complete, reverse deps 32% |
| Active maintenance | ★★★☆☆ | 60% of cards stale by 11+ days |
| Analysis tool usage | ★☆☆☆☆ | blast-radius never used post-setup, drift never run |
| Registration discipline | ★★★★★ | Consistent across 15+ tasks |

**Bottom line:** The fabric is a well-populated structural map (registration works), but its analytical capabilities (blast-radius, drift, impact analysis) are dormant. It functions as documentation, not as an active governance tool. For specialist agent orchestration, this means the topology data EXISTS and could inform delegation decisions, but the query/analysis layer would need activation.
