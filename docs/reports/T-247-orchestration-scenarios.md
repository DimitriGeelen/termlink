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

<!-- Filled by agent dispatches -->
