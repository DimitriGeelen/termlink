# T-247: Orchestration Real-World Scenarios

## Problem Statement

We've built `orchestrator.route` (T-237) and the bypass registry (T-238). These have unit tests but no end-to-end validation against real-world usage patterns. We need concrete scenarios from multiple perspectives to stress-test the system, discover gaps, and calibrate parameters (promotion threshold, failover behavior, registry performance).

## Approach

5 agents explore scenarios from different lenses:
1. **Framework maintenance** — fw doctor, audit, metrics, context operations
2. **Code review / quality** — lint, test, blast-radius, diff analysis
3. **Infrastructure / deploy** — server ops, homebrew, remote sessions
4. **Research / explore** — codebase search, documentation, learning capture
5. **Adversarial / failure** — malicious bypass, cascading failures, race conditions

## Agent Reports

| # | Lens | File | Scenarios |
|---|------|------|-----------|
| 1 | Framework maintenance | [T-247-scenarios-framework-maintenance.md](T-247-scenarios-framework-maintenance.md) | Health check + bypass lifecycle, cross-specialist audit, stale session cleanup |
| 2 | Code review / quality | [T-247-scenarios-code-review.md](T-247-scenarios-code-review.md) | Blast-radius check, lint de-promotion, diff fan-out |
| 3 | Infrastructure / deploy | [T-247-scenarios-infrastructure.md](T-247-scenarios-infrastructure.md) | Rolling brew audit, pre-deploy health gate, remote session lifecycle |
| 4 | Research / explore | [T-247-scenarios-research.md](T-247-scenarios-research.md) | Cross-codebase grep, episodic memory query, concurrent learning capture |
| 5 | Adversarial / failure | [T-247-scenarios-adversarial.md](T-247-scenarios-adversarial.md) | Registry write race, promotion gaming, stale route cascade |

## Gaps Discovered

| # | Gap | Severity | Found by | Task |
|---|-----|----------|----------|------|
| 1 | Registry write race — load/modify/save not atomic | High | Adversarial, Code Review, Research | T-248 |
| 2 | No command validation — arbitrary strings promotable | Medium-High | Adversarial | T-249 |
| 3 | Transport failures not tracked in bypass stats | Medium | Adversarial | T-250 |
| 4 | No mutation-vs-read-only distinction | Medium | Framework Maintenance | T-251 |
| 5 | No infra-vs-command failure distinction | Medium | Code Review, Infrastructure | T-252 |
| 6 | No bypass invalidation signal | Medium | Code Review | T-253 |
| 7 | Serial failover with fixed timeout | Medium | Adversarial | T-254 |
| 8 | Semantic failures invisible to registry | Low-Medium | Research | (documented, no fix needed — caller responsibility) |

## Task Decomposition

| Task | Name | Horizon | Dependencies |
|------|------|---------|-------------|
| T-248 | Fix bypass registry write race — atomic file operations | now | — |
| T-249 | Bypass command validation — denylist and caller identity | now | — |
| T-250 | Track transport failures in bypass registry | now | — |
| T-251 | Bypass eligibility — mutation flag | now | — |
| T-252 | Distinguish infra vs command failure | now | T-250 |
| T-253 | Bypass invalidation signals | next | T-248 |
| T-254 | Failover optimization — circuit breaker | next | T-250 |
| T-255 | Live orchestration test harness | now | T-248..T-252 |

## Suggested Build Order

1. **T-248** (write race) — highest severity, foundational fix
2. **T-250** (transport failures) + **T-251** (mutation flag) — independent, can parallel
3. **T-249** (command validation) — security hardening
4. **T-252** (infra vs command failure) — depends on T-250's tracking
5. **T-255** (live harness) — exercises all fixes
6. **T-253** + **T-254** — next horizon, build after core stabilizes
