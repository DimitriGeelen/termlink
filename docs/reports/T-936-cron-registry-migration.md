# T-936: Cron Registry Migration Research

## Problem

Termlink had three inconsistent cron layers:
1. Live `/etc/cron.d/agentic-audit-termlink` — 11 jobs using global binary path
2. Git-tracked crontab stub — empty (migration started but not completed)
3. `cron-registry.yaml` — was near-empty, now populated with all 11 jobs

## Findings

### Current State (2026-04-12)
- `cron-registry.yaml`: **11 jobs populated** (all live jobs catalogued)
- `fw cron install --dry-run`: Clean diff showing two intentional changes:
  1. Binary path: `/root/.agentic-framework/bin/fw` -> `/opt/termlink/.agentic-framework/bin/fw` (vendored)
  2. Pickup processor: every-30s hack -> every-15m registry entry

### Options Evaluated
- **Option A (Full registry):** GO — registry already populated, dry-run clean
- **Option B (Rollback to hand-written):** Rejected — fights framework direction
- **Option C (Hybrid):** Unnecessary — registry covers everything

### Binary Path Decision
The vendored path (`/opt/termlink/.agentic-framework/bin/fw`) is correct. The global path (`/root/.agentic-framework/bin/fw`) points to a different framework lineage (v1.4.553 vs vendored v0.9.700). After T-909 vendoring, the project should be self-contained.

## Recommendation

**GO** — Run `fw cron install` to apply the migration. All jobs preserved, binary path corrected.
