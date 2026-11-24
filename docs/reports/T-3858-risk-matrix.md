# Risk matrix — TermLink jobs on the ring20 build pool (T-3858)

## Header

- **Change:** TermLink sends its test, clippy and release-build jobs to the ring20 build pool. A submit script
  written by ring20 (`010-pool-submit.py`, reviewed v4 sha256 69f17302…f012) packs a `git archive` of a named commit
  (without `.claude/settings.local.json`, `.context`, `.tasks`, `.fabric`, `.agentic-framework`) plus the vendored
  crates, posts both with ring20's pool client (`/usr/local/lib/ring20-pool-client/pool-client.py`) to the hub on
  .122 under our TermLink hub identity, and reads the results back. Later called from cron or runme.
- **Baseline:** today. Every build and test runs locally on .107; nothing of ours runs on pool nodes; ring20's pool
  client is installed on .107 but we never run it.
- **Task:** T-3858

## Scales

| Value | Likelihood | Impact |
|------:|------------|--------|
| 1 | rare | negligible |
| 2 | unlikely | minor |
| 3 | possible | moderate (one job or result wrong) |
| 4 | likely | major (token stolen or data leaked, contained) |
| 5 | almost certain | severe (node or network compromised, accounts lost) |

**Score** = likelihood × impact, 1-25. **Bands:** low 1-5, medium 6-12, high 15-25.

## Summary

| # | Risk | Domain | How to detect the door is open | How to detect it happening | Impact | Likelihood | Exists anyway (baseline)? | Introduced by the change? | Key mitigation |
|---|------|--------|--------------------------------|----------------------------|-------:|-----------:|---------------------------|---------------------------|----------------|
| 1 | Third-party code runs unattended as root on .107 with our hub identity | supply chain / host | the submit script or pool client changes sha256 | a run prints a sha256 other than the pinned one | 5 | 2 | no | NEW (we never ran the pool client) | pin the submit script, pool-client.py and poold.py by sha256, refuse on change (non-root dropped: it needs a copy of a hub credential, T-3859 Decisions) |
| 2 | Repository content copied to more hosts | disclosure | the code-files list grows to include an excluded path | `code-files.txt` of a run lists a path outside the allowed set | 2 | 2 | yes (OneDev and the GitHub mirror hold it) | PARTLY NEW (three estate nodes and the .122 hub store a copy) | excludes; file list kept per run; secret scan at commit |
| 3 | A false pass from the pool is trusted | integrity | a release or close relies on a pool result alone | a pool pass where a local run at the same commit fails | 3 | 2 | no | NEW | pool results advisory; release and task gates keep the local run |
| 4 | Unattended submissions misfire (stuck, flooding, or silently not running) | availability / observability | a cron line exists without a heartbeat or log reader | no new result log for a scheduled day, or many jobs queued | 2 | 2 | no | NEW | exit 2 on could-not-run; log + heartbeat; one job set per schedule tick |

## Actor split (one table per risk)

Actors (fixed list): `own-agent-unintentional`, `human-unintentional`, `outsider-intentional`, `provider-host`, `intruder`.

### Risk 1 — Third-party code runs unattended as root on .107 with our hub identity

| # | Actor | How | L | I | Score | Mitigated L × I | Mitigated score |
|---|-------|-----|--:|--:|------:|-----------------|----------------:|
| 1.1 | own-agent-unintentional | we update to a new script version without reading it | 2 | 4 | 8 | 1 × 4 | 4 |
| 1.2 | human-unintentional | a hand edit of the handoff copy is picked up | 1 | 4 | 4 | 1 × 4 | 4 |
| 1.3 | outsider-intentional | no outside path to the handoff dir or the pool-client install on .107 | 1 | 5 | 5 | 1 × 5 | 5 |
| 1.4 | provider-host | ring20 ships a defective or over-reaching pool client update | 2 | 5 | 10 | 1 × 5 | 5 |
| 1.5 | intruder | someone with write access on .107 replaces the client to act as us | 1 | 5 | 5 | 1 × 5 | 5 |

### Risk 2 — Repository content copied to more hosts

| # | Actor | How | L | I | Score | Mitigated L × I | Mitigated score |
|---|-------|-----|--:|--:|------:|-----------------|----------------:|
| 2.1 | own-agent-unintentional | an exclude is dropped from the command line | 2 | 2 | 4 | 1 × 2 | 2 |
| 2.2 | human-unintentional | a secret is committed and then shipped | 1 | 4 | 4 | 1 × 4 | 4 |
| 2.3 | outsider-intentional | reads a pool node's job store | 1 | 2 | 2 | 1 × 2 | 2 |
| 2.4 | provider-host | pool nodes keep job archives longer than needed | 2 | 2 | 4 | 2 × 2 | 4 |
| 2.5 | intruder | compromises a pool node and reads archives | 1 | 2 | 2 | 1 × 2 | 2 |

### Risk 3 — A false pass from the pool is trusted

| # | Actor | How | L | I | Score | Mitigated L × I | Mitigated score |
|---|-------|-----|--:|--:|------:|-----------------|----------------:|
| 3.1 | own-agent-unintentional | a task is closed on a pool pass alone | 2 | 3 | 6 | 1 × 3 | 3 |
| 3.2 | human-unintentional | a release is cut on a pool pass | 1 | 3 | 3 | 1 × 3 | 3 |
| 3.3 | outsider-intentional | forges a result on the relay | 1 | 3 | 3 | 1 × 3 | 3 |
| 3.4 | provider-host | a node with a different toolchain or a stale vendor set reports pass | 2 | 3 | 6 | 1 × 3 | 3 |
| 3.5 | intruder | alters a node to always report pass | 1 | 3 | 3 | 1 × 3 | 3 |

### Risk 4 — Unattended submissions misfire

| # | Actor | How | L | I | Score | Mitigated L × I | Mitigated score |
|---|-------|-----|--:|--:|------:|-----------------|----------------:|
| 4.1 | own-agent-unintentional | a cron line submits repeatedly or never | 2 | 2 | 4 | 1 × 2 | 2 |
| 4.2 | human-unintentional | the cron is left on after the pool is retired | 2 | 1 | 2 | 1 × 1 | 1 |
| 4.3 | outsider-intentional | none: submission needs our hub identity | 1 | 1 | 1 | 1 × 1 | 1 |
| 4.4 | provider-host | pool outage makes every run end "could not run" | 2 | 2 | 4 | 2 × 1 | 2 |
| 4.5 | intruder | none beyond risk 1 | 1 | 1 | 1 | 1 × 1 | 1 |

## Net

The change adds one real risk: ring20's code (the submit script and the pool client it calls) would run unattended
as root on .107 under our hub identity (risk 1, provider-host 10). Mitigation, before any schedule: pin the submit
script, pool-client.py and poold.py by sha256 and refuse to run when any differs (a new version is read in full and
re-pinned in a commit). Pinning sets the likelihood to 1, so every row is at 5 or below. A non-root user was
considered and dropped (T-3859 Decisions): it would need a copy of a hub credential, a new risk for no score change.

Risks 3.1 and 3.4 (6) fall to 3 by keeping pool results advisory: local test runs stay the gate for task closure and
releases. Risks 2 and 4 stay low.

**Needs operator acceptance:** none after mitigation; risk 1 needs its mitigation task done before the first scheduled run.
