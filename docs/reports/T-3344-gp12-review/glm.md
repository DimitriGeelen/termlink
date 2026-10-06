# GLM review — GP-12 (2026-10-06)

**External review — measurable meaning for "very simple" (R-6, GP-12)**

1. [MISSING] (Q1) Failure 1 (key stored under another agent's name; 32,205 refusals with a fresh heartbeat) passes all five proxies — none measures identity custody or deaf-while-alive, the arc's worst failure.
2. [MISSING] (Q1) Failure 3 (agent deliberately excluded from the waker) also passes — a deliberate workaround is invisible to counts unless the status call must name exclusions.
3. [CHANGE] (Q1) Failure 5 is only half-covered: proxy 2 constrains install-time settings, but the split was a runtime binding — CAND-8's `hub_id` refuse-on-mismatch must run continuously, not once at install.
4. [AGREE] (Q1/Q2) Proxy 3 is the strongest of the five — but only if "can receive" is defined per CAND-6 (last real HANDED_OVER, never a heartbeat), else it re-creates R-29.e's false CLEAR.
5. [CHANGE] (Q2) Proxy 1 is gamed by one process hiding threads, cron entries and hook scripts — count cooperating components that must be alive and mutually consistent (daemons, schedulers, hooks, PTYs), not processes.
6. [CHANGE] (Q2) Proxy 5 is gamed by moving lines into sourced files, vendored helpers or chain neighbours — today's 620-line sidecar inside a 2,472-line chain is exactly that pattern; the tripwire must count the transitive closure of sidecar-owned source.
7. [MISSING] (Q2) Every proxy needs a kill-checked negative control (CAND-3: the three real failures as standing controls) proving it goes red when the failure is replayed — otherwise green is unfalsifiable.
8. [CHANGE] (Q3) Keep per-agent: R-6.a is confirmed [C], and a per-host receiver concentrates all agents' keys in one process — the exact failure-1 blast radius — though it would shrink the failure-4 process count; measure "components per delivered message" instead of blessing either shape.
9. [CHANGE] (Q4) Line counts don't compare shell, Python, or OD-9's ruled in-binary `termlink sidecar` — the cap is ill-defined for the confirmed end state; demote it to a same-implementation growth tripwire.
10. [MISSING] (Q4) API-call counts are gamed by mode-flag mega-calls; derive the count from the ruled contract instead (OD-3/4/5/8 already fix send, status, stage events, answer pull, admin) — then "10" becomes evidence, not assertion.
11. [MISSING] (Q5) Persistent stores/files per agent (queue, flag, journal, mirror, keys) — failure 1 lived precisely in uncounted store and identity sprawl; count and list them.
12. [MISSING] (Q5) Identities/keys per agent and their allowed locations, as a checked invariant — A-4 records that all agents on a host still sign as one identity, which no proxy touches.
13. [MISSING] (Q5) Crash-recovery steps (QS-19/GP-5 still open): kill the sidecar mid-delivery and count operator actions to a consistent state; "very simple" should mean zero, automatic.
14. [MISSING] (Q5) Time-to-diagnose: a measured bound for an operator to answer "why didn't my message arrive" using proxy 3, drilled against the three standing failures.
15. [CHANGE] (Q6) 1,000 lines is an anchor, and it collides with proxy 1: consolidating injector+waker+mirror+API+supervisor into the one permitted process re-baselines ~1,979 lines — derive the cap by budgeting per function (store, queue, inject, report, supervise) against the tiered tests, then tripwire on per-release deltas.
16. [MISSING] (Q5) Message-state count as a simplicity measure: D-3 plus OD-3/OD-8 rulings already approach a dozen sender-visible states — every state added is complexity the operator pays for at diagnosis time; cap and justify additions.

**Verdict:** The five proxies measure adjacency and packaging (failures 2, 4, half of 5) while the arc's real killers — binding, key custody, deaf-while-alive (1, 3) — go unmeasured; keep them as tripwires, tie each to a standing negative control, and add the missing identity, store, recovery, state-count and diagnose-time measures.
