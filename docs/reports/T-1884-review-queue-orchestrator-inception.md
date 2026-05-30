# T-1884 — Consumer-side Review-Agent Orchestrator (Inception)

**Status:** started-work (filing-time draft, awaiting operator review of plan)
**Recommendation:** GO (pending S1/S2/S3 spike evidence)
**Created:** 2026-05-30
**Predecessors:** T-1443 (`fw reviewer` v1.5), T-1950 (auto-tick policy), T-2002 (`ux-review` agent)

## Problem

47 tasks in `partial-complete` (Agent ACs done, Human ACs unchecked) for 2+
weeks. Queue growth ~3-5/week from ongoing agent-skill / soak work. Without a
drain mechanism, 100+ in 10 weeks. The Watchtower review page is the canonical
operator surface, but 47 manual click-throughs is friction the operator never
overcomes.

## What's already shipped upstream (do NOT re-build)

| Surface | Scope | Action |
|---|---|---|
| `fw reviewer` (T-1443 v1.5) | `[REVIEWER]`-prefixed **Agent** ACs | Static-scan + AUTO-TICK on PASS |
| `agents/ux-review/` (T-2002) | Interactive render surfaces | Playwright drive + screenshots + console scan; **informs only**, never ticks |
| T-1950 auto-tick policy | Constitutional rail | NEVER tick `### Human` ACs (Decisions 36/113/213) |

The gap: a **consumer-side router** that picks the right surface per Human-AC
class and surfaces evidence — without violating the constitutional rail.

## AC class taxonomy (from manual scan of the 47)

Rough partition (to be quantified in S1):

- **REVIEW / render-surface** — "watch view is steady (no flicker)", "table is
  scannable", "live overview is steady" → ~15 of 47 (T-1486, T-1494, T-1496,
  T-1498, T-1557, T-1558, T-1559, …). **ux-review territory.**
- **REVIEW / CLI-output** — "error messages name the failing input clearly",
  "output is operator-scannable", "empty-with-filter reads naturally" → ~22 of
  47 (T-1482, T-1483, T-1484, T-1485, T-1487, T-1488, T-1489, T-1490, T-1491,
  …). **Capture + grep + flicker-check territory.**
- **RUBBER-STAMP / mechanical** — "cron entry installed in /etc/cron.d", "MCP
  listing shows the three new tools", "Skill discoverable and invokable from
  Claude Code" → ~7 of 47 (T-1691, T-1696, T-1722, T-1723, T-1836, T-1841).
  **Shell-validatable.**
- **RUBBER-STAMP / release** — "GitHub Release published with macOS+Linux
  binaries" → ~3 of 47 (T-1673, T-1691). **HTTP-validatable via gh CLI.**

## Routing policy (proposal — to be ratified after spikes)

```
                   ┌─ [REVIEWER]-Agent     → fw reviewer (existing)
                   │
   Per AC class ───┼─ [REVIEW] render      → fw ux-review → evidence into Updates
                   │                          (NEVER ticks; operator clicks)
                   │
                   ├─ [REVIEW] CLI         → script -c capture + grep + flicker
                   │                          → evidence into Updates
                   │                          (NEVER ticks; operator clicks)
                   │
                   ├─ [RUBBER-STAMP]       → shell-validate Steps → PASS/FAIL
                   │   mechanical            → evidence into Updates
                   │                          → POLICY: tick on PASS (memory
                   │                            [Validate-don't-punt]) IF
                   │                            operator opted in this session;
                   │                            else surface evidence and
                   │                            preserve constitutional rail
                   │
                   └─ FAIL on any           → file T-XXXX investigate-and-fix
                                             with G-019 RCA stub + link
```

**The load-bearing decision:** how `[RUBBER-STAMP]` validation outcome routes.

- **Path A (constitutional):** never tick, always surface. Operator clicks 47
  times but each click costs <5s because evidence is in the Updates block.
  Downside: still 47 clicks.
- **Path B (memory-blessed):** tick on PASS-ROBUST. Operator clicks only the
  FAIL/INCONCLUSIVE residue (~5-10 of 47). Upside: actual drain. Downside:
  overrides T-1950 — needs explicit Tier-2 operator authorization per session.

The orchestrator can support BOTH via a CLI flag (`--tick-mechanical-pass`
defaults OFF). Operator selects per invocation.

## Spike plan

### S1: AC classifier (≤45 min)

Read all 47 active `partial-complete` task files, extract `### Human` ACs +
Steps blocks. Classify per the taxonomy above. Emit a markdown table:

```
| Task    | Prefix          | Class                 | Confident? |
|---------|-----------------|-----------------------|------------|
| T-1486  | [REVIEW]        | REVIEW / render       | Y          |
| T-1487  | [REVIEW]        | REVIEW / CLI          | Y          |
| T-1696  | [RUBBER-STAMP]  | RUBBER-STAMP / mech   | Y          |
| ...     |                 |                       |            |
```

Confidence = both prefix and Steps content unambiguously map to one class.

**Pass:** ≥80% Confident=Y.
**Fail:** <50% Confident=Y → operator wins by sorting manually.

### S2: Mechanical-Step dry-run validator (≤45 min)

For each `RUBBER-STAMP / mechanical` AC from S1, parse the Steps block, synth
shell commands, dry-run against current state. Compare exit code + stdout vs
**Expected:** block. Output:

```
T-1696  RUBBER-STAMP / mech   PASS-ROBUST  cron /etc/cron.d/termlink-release-mirror-canary exists
T-1722  RUBBER-STAMP / mech   FAIL         upstream not landed on /opt/999-AEF
T-1841  RUBBER-STAMP / mech   PASS-LOOSE   skill discoverable but smoke evidence stale 1d
T-1836  RUBBER-STAMP / mech   PASS-ROBUST  MCP lists termlink_listener_heartbeat
```

**Pass:** ≥15 of 47 ACs land in PASS-ROBUST or PASS-LOOSE.
**Fail:** <5 — wrapper saves less than it costs.

### S3: ux-review wireup smoke (≤45 min)

Pick T-1486 (`agent presence --watch view is steady`). Drive ux-review (or its
underlying Playwright path) to capture:

1. Screenshot at t=0
2. Screenshot at t=5s (verify no flicker by pixel-diff)
3. Console errors during 5s observation
4. Append evidence into T-1486's `## Updates` block

**Pass:** evidence-block-only is enough for operator rubber-stamp.
**Fail:** per-task config (URL/route/page-id) explosion — nullifies one-verb UX.

## Decided (2026-05-30 operator dialogue)

**D1 — Tick-on-mechanical-PASS default: ON, gated on independent reviewer.**
Auto-tick is allowed BUT only when a separate reviewer agent (distinct context
from the producer) emits a PASS verdict. The producer agent NEVER ticks own
work. This is the same anti-bias rail T-1443/T-1950 already applies to
`[REVIEWER]`-Agent ACs, extended to `[RUBBER-STAMP]`-mechanical Human ACs.

**D2 — Verb name: `fw independent-review`.** Names the load-bearing rail
(independence) rather than the action (drain). Operator-typed.

**D3 — UX: hybrid Option C with isolation rail.**
- `fw independent-review` (no args) → batch over all `partial-complete`
- `fw independent-review T-XXX` → one task
- `fw independent-review --since 7d`, `--class rubber-stamp`, `--resume`
  → filtered batch
- **Isolation rail (operator-specified):** each reviewer instance works on
  EXACTLY ONE task at a time. The orchestrator may run reviewers sequentially
  OR dispatch multiple in parallel (each handling its own one task) — both are
  legal, but no reviewer ever crosses task boundaries. State journaled to
  `.context/working/.independent-review-state.json` for crash-safe resume.
- Dispatch mechanism for parallel mode is TBD in build phase (likely
  termlink-dispatched reviewer agents on separate hosts, or Claude Code
  sub-agents on this host — explicitly NOT `claude -p` per memory
  `feedback_avoid_claude_p_expensive.md`).

**D4 — Auto-followup filing: always, even on INCONCLUSIVE.** Anti-pile-up
default. Every non-PASS result fires `T-XXXX investigate-T-<src>` with G-019
RCA stub. Operator gets explicit next-actions, not silent surface-only that
gets ignored. The 47-task pile-up that motivated this inception is precisely
the failure mode being designed against.

## Spike results

### S1 — AC classifier (COMPLETE, PASS at 87.5%)

**Round 1** (initial taxonomy): 65.3% confident (47/72). MARGINAL — failure
mode interpretable: needed 3 more classes (OBSERVE-INFRA, OPERATOR-ACTION,
TIME-GATED) + RUBBER-STAMP prefix-trust.

**Round 2** (refined): **87.5% confident (63/72). PASS.** Final distribution:

| Class | Count | Conf% |
|---|---:|---:|
| REVIEW-CLI | 32 | 94% |
| RUBBER-STAMP-MECHANICAL | 9 | 100% |
| REVIEW-RENDER | 8 | 88% |
| OBSERVE-INFRA | 7 | 100% |
| OPERATOR-ACTION | 6 | 100% |
| TIME-GATED | 3 | 100% |
| RUBBER-STAMP-RELEASE | 1 | 100% |
| OTHER | 6 | 0% |

**Drain split:**
- Auto-validatable (REVIEW-CLI + REVIEW-RENDER + RUBBER-STAMP-* + OBSERVE-INFRA):
  **57 of 72 = 79%** — orchestrator can dry-run + surface evidence
- Surface-only (OPERATOR-ACTION + TIME-GATED + OTHER):
  **15 of 72 = 21%** — orchestrator surfaces "needs human" with no verdict

**A1 status:** VALIDATED. Prefix + content-keyword classification achieves
≥80% confident routing on the current 72-AC corpus.

**Evidence:** `docs/reports/T-1884-S1-results.md` (108-line full output);
`scripts/T-1884-S1-classify.py` (165-line classifier).

**Taxonomy updates beyond filing-time spec:**
- Added OBSERVE-INFRA (state observation against remote host — `termlink
  remote exec`, hub probe). Origin: T-1137's "/var/log below 50%" type ACs.
- Added OPERATOR-ACTION (human must do first — rotate PAT, decide). Origin:
  T-1695 / T-1799 PAT rotation flow.
- Added TIME-GATED (deferred event — "on next deploy"). Origin: T-1633's
  post-bake observation pattern.
- RUBBER-STAMP prefix-trust: absent stronger signal, default to MECHANICAL
  on the prefix alone. The orchestrator dry-runs Steps as the confirmation
  mechanism.

### S2 — Mechanical-Step dry-run (PENDING)

### S3 — ux-review wireup smoke (PENDING)

## Dialogue Log

### 2026-05-30 — Inception filing dialogue (post-compaction continuation)

**Operator** picked option 3 ("file a helper task that combines fw reviewer +
ux-review + shell-validate + auto-followup into one verb") from the post-resume
options. Pre-compaction investigation found upstream T-1950 design that
constitutionally forbids auto-ticking `### Human` ACs, in tension with project
memory `[Validate-don't-punt]` which says agent should tick mechanical Steps.

**Agent** staged the inception with three spikes (S1/S2/S3), reframed the
load-bearing decision as a policy lever (tick-on-mechanical-PASS flag) so the
operator selects per session — preserving both the constitutional default
AND the memory-blessed power-user path.

**Outstanding before spike execution:** operator review of this filed
template (template-review-first per inception discipline #2) — agent will NOT
execute S1/S2/S3 until operator gives the GO.
