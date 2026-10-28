<!-- T-3615 external review, seat codex, source /tmp/tl-dispatch/t3615-rev-codex-279f651a7210/result.md, exit 1 -->
1. **Objectives: coverage is not achieved. Confidence: high.**

   References use repository-relative paths: **D** = `docs/reports/T-3615-fleet-telemetry-design.md`; **A** = `crates/termlink-session/src/invocation_audit.rs`; **R** = `scripts/telemetry-rollup.sh`; **V** = `scripts/telemetry-fleet-view.sh`; **T** = `tests/telemetry-rollup-fixtures.sh`.

   The proposed census uses attributed rows, registered sessions and presence. Extending dispatch-folder scans covers dispatched non-users, but provides no discovery mechanism for an independently launched, unregistered agent that never calls TermLink. Installing collectors only where TermLink runs also falls short of “every agent on any fleet host” (D:46–58,94–103). Call this an **observed-agent census** until independent lifecycle collection exists.

   Current usage aggregates discard harness, model, worker, task and project, retaining class/surface/name counts; those dimensions cannot support later comparisons once raw records rotate (R:329–341). Remote installation relocates the script under `/var/lib/termlink-telemetry`, while dispatch/review defaults derive from its parent directory: persistent project records need explicit discovery/configuration (runme.sh:2006–2008; R:78,141–144).

   Fleet completeness is misleading: without `--expect`, hosts come only from reports inside the window. A host silent throughout the window disappears; an empty fleet can return success (V:86–87,134–142,164). Require an authoritative host inventory.

2. **Attribution confuses execution context with identity. Confidence: high.**

   A markerless non-Anthropic agent invoking commands with piped stdin becomes `background`; a human using redirected stdin does too. An agent-marked one-shot service can become `standing` before its SDK marker is considered. Inherited explicit class overrides even dispatch markers (A:275–299). The heuristic therefore hides precisely the unfamiliar agents this project seeks.

   Classes mix lifecycle (`standing`), launch mechanism (`dispatched`) and role (`reviewer`); a standing reviewer cannot express both. Split those dimensions and retain explicit unknown values (A:248–260,284–299).

   Attribution is cached per process, and model falls back to `ANTHROPIC_MODEL`; inherited or changing runtime configuration can produce stale identity (A:314–320,346–353). Provider inference from worker kind is a hypothesis, not evidence of the endpoint used (D:98–100).

   Record attribution source/rule and confidence. Validate against independently labeled launches across every harness, service, interactive session and nested worker. Reconcile counts against launcher records and report confusion rates. Reduced `unknown` counts do not demonstrate increased accuracy.

3. **Outcomes measure execution hygiene more than fitness. Confidence: high.**

   Dispatch aggregation groups class/kind/model, defaults missing kind to `claude`, skips runs without finish timestamps, and tests emptiness by file size. This can credit unidentified workers to Claude, exclude crashed/hung launches, and count boilerplate as useful output (R:353–385). Task type appears separately in close-ledger aggregates, without a common run-level join in the exported result (R:397–403,460–464).

   Verdict status joins that day’s verdicts against all currently available application records. Later applications do not automatically revise previously posted days because catch-up selects only missing days. Comparisons depend on evaluation delay and rerun timing (R:108–112,409–443). Applying a verdict measures acceptance, not independent correctness.

   Add stable task/run/attempt IDs; task category and difficulty; actual model/configuration; launch failures and cancellation; queue, execution and end-to-end latency; tokens, money and local compute; accepted completion, regression rate, rework and human review time. Evaluate after a defined maturation period. Compare equivalent jobs with sample sizes and uncertainty. Otherwise stronger workers assigned harder jobs can look worse.

   Expanding roots also requires deduplication: directories are counted independently and verdict metadata is keyed only by directory name, permitting duplicate counts or cross-project collisions (R:354–377,429–430).

4. **Privacy and report integrity need changes before expansion. Confidence: high for local behavior; medium for exploitability.**

   There is a concrete privacy-contract violation: aggregates export each sink’s full `path`, despite prohibiting private paths beyond project name (R:337–341; D:62–63). Standing unit names and reconnect peer identifiers also leave the host (R:241,453–456). The privacy fixture excludes two example raw values rather than validating an export allowlist (T:156). Export opaque source labels and constrained categorical values.

   Host identity and precedence are self-asserted: the producer accepts `--host`; the viewer trusts payload host/day and lexicographic `generated_at` without checking envelope identity. A publisher able to inject such a payload could mask another host’s report using a future timestamp (R:65,467–470; V:69–84). Bind host identity to authenticated credentials at ingestion; validate timestamps/schema and retain revisions.

   “No new secret” does not establish least privilege. The design suspects remote hosts possess the full hub secret (D:39–42,74–75). Verify and replace that exposure before expanding unattended use; use host-scoped telemetry publishing rights. Actual credentials and hub authorization were not verified, so cross-host publication/deletion permissions remain unknown.

5. **Data quality can turn missing evidence into apparent zero. Confidence: high; two-week sufficiency: medium.**

   Existing files are labeled present even when no records survive for the requested day; only current and `.1` files are read. Given roughly one-day retention, daily collection and catch-up cannot establish complete windows (R:191–210,308–341; D:25,76–77). Add coverage intervals, loss/error counters and partial status. Audit append errors are swallowed, so successful commands do not prove successful observation (A:395–402).

   Rerunning an old day after rotation can replace a fuller report with a later partial report because latest generation time wins (V:81–84). Preserve immutable snapshots with explicit completeness/revision rules. The viewer fetches only one 5,000-message page, risking missing newer history as volume grows (V:44).

   Standing restarts and dead letters are current snapshots, including during historical catch-up, not daily deltas or uptime (R:145–156,289–295,447–458). Polling is classified solely by verb: intentional `list`/`pty output` calls become polling, while background labels do not remove collector-generated traffic (R:299–305,329–336; runme.sh:2018).

   Missing reports cannot distinguish collection, host, transport or collector failures. Add independently checked delivery receipts and an out-of-band health check; the design acknowledges the latter’s absence (D:64–65).

   Two weeks is an instrumentation pilot, not a defensible universal cut threshold. Require complete observation and sufficient comparable tasks, including rare but essential workflows.

6. **Top five changes, ranked by impact. Confidence: high.**

   1. Add independent lifecycle collection and an authoritative host/project inventory; quantify unobserved coverage before claiming a census (§1).
   2. Persist incremental local summaries with coverage/loss metadata, immutable delivery IDs, receipts and independent health checks; prevent partial reruns replacing complete reports (§5).
   3. Bind reports to host-scoped credentials, resolve suspected full-secret exposure, and enforce an export allowlist (§4).
   4. Establish a shared run/task/outcome schema covering cost, latency, rework and delayed quality assessment before cross-project aggregation (§3).
   5. Separate role/lifecycle/identity dimensions and record attribution provenance; drop automatic provider certainty and calendar-only keep/cut decisions (§§2,5).

7. **Verification limits. Confidence: high in static findings; low in deployment claims.**

   I read the supplied design, attribution implementation, rollup, fleet viewer, fixtures and relevant installer sections. The checkout lacks usable Git metadata, so I could not verify commit `8e52cb0cf`. No builds, tests, project scripts, network requests or host contacts were performed.

   Fixtures use a fake TermLink binary and cannot establish delivery or authorization guarantees (T:2–4,40–45). Live inventory, credentials, ACLs, rotation concurrency, dispatcher schemas, model identity and report completeness remain unverified. Slice 4 is a proposal, not an implementation reviewed here (D:93–103).

   Saved to `.context/working/t3615/review-openai.md`.