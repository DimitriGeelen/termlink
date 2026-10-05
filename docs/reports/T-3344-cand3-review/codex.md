# Codex review — CAND-3 verification tiers (2026-10-05)

1. [AGREE] **Keep the one-agent tier.** A real receiver with a scripted sender can verify busy/idle delivery, transcript evidence, downgrade behaviour and readiness precisely; two LLMs add little to those isolated assertions.

2. [CHANGE] **“Lowest tier” should apply per failure mode, not per requirement.** A requirement can fail in logic, integration or deployment independently; record multiple necessary tests, their fault models and evidence, rather than assign one tier and exclude higher coverage.

3. [CHANGE] **Tier 1 overclaims pure logic.** State classification and retention policy fit fixtures; truthful liveness inputs, durable stage memory, actual sweeping, resend after loss and authenticated consent enforcement need real components. Otherwise all three production failures can coexist with green fixtures.

4. [CHANGE] **Authenticated hub identity is insufficient.** The stray second hub could truthfully report its own identity; verify that launcher, session registration, sidecar, inbox and sender resolve to the intended canonical hub, and deliberately introduce a second runtime directory to expose split bindings.

5. [CHANGE] **Telemetry ingestion is not journey verification.** Tier 2 can prove that a hub accepts events; proving that HANDED_OVER events correspond to actual session content requires tier 3, and reply delivery requires tier 4. Independently correlate fresh message IDs, content and session identity with transcripts; the October 3 failure passed storage checks.

6. [CHANGE] **Fresh heartbeat must not imply receive health.** R-29.e explicitly accepts “fresh heartbeat and no flag” as CLEAR, which the 32,205 refused confirmations contradict; exercise installed key lookup, authenticated confirmation and observable progress with a live sidecar, then verify the real agent’s deafness response.

7. [MISSING] **Topology is an additional test dimension.** OD-1 requires two hubs across hosts, circuit establishment, credentials, direct transport, hub fallback and one conversation log; test partition, reconnect and path switching without duplicate hand-over or silent instance substitution. Two agents on one hub cannot establish these claims.

8. [MISSING] **Harness coverage is explicit, not optional.** OD-7 requires Claude Code and opencode adapter parity per release; run tier-3 contract tests against each real harness and a representative mixed-harness round trip, including already-running, idle, busy and naturally restarted sessions under OD-6.

9. [MISSING] **Recovery needs real persistence and interruption tests.** Kill sidecar/hub around STORED and between transcript hand-over and settlement recording; restart and reboot representative hosts, then verify recovery without loss or duplicate delivery. The hand-over/recording crash window particularly challenges CAND-2’s promise; a fixture cannot establish atomicity across those systems.

10. [MISSING] **The deployed installation must be the subject of acceptance.** Exercise clean install, upgrade and boot through installed supervisors, schedules, hooks, keys and runtime paths, including systemd and non-systemd environments (R-8/R-39); OD-9 also requires exactly one receiver per inbox and two-agent acceptance with a negative control before each host switches.

11. [CHANGE] **Tier-2 negative control:** deliberately bypass the open-message retention guard, then run the real sweep with an old open journey and an expired final journey; the unchanged retention assertion must fail because the open journey disappears. Merely showing that a scripted client can write/read records proves nothing about retention.

12. [CHANGE] **Tier-3 negative control:** disable the installed receiver mail-delivery hook while keeping storage and heartbeat healthy, then send urgent mail during a controlled busy turn; the normal transcript/deadline assertion must fail, no HANDED_OVER may appear, and the appropriate stalled state must become visible.

13. [CHANGE] **Tier-4 negative control:** leave the request leg working but block the reply’s delivery into the original sender’s session across all eligible paths; the unchanged round-trip check must fail despite a receiver-generated reply. Also retain OD-14’s specified injector-stop control proving escalation lands by the next main-agent session.

14. [CHANGE] **Closure is a release gate, not continuing proof.** Require the daily canary’s actual installation and execution, detection of missing/stale runs, and verified escalation landing; rotate coverage across fleet configurations. OD-14 requires escalation, not an immediate canary alarm. A once-green run cannot prevent later darkness.

15. [MISSING] **Two agents omit existing acceptance scope.** R-1.e explicitly requires three-agent many-to-many delivery; add that scenario and resolve the operator-leg acceptance gap. Include explicit ACKNOWLEDGED_NO_ACTION and OD-15’s denied-resume cases, not only successful resumption.

16. [CHANGE] **Remove blanket fixture/mutant obligations for every requirement on every push.** Deployment and agent behaviour cannot all be meaningfully reduced to fixtures; share representative live scenarios across requirements and use simulated time for long retention/polling schedules, while retaining targeted negative controls.

Overall verdict: Keep the four tiers, but replace exclusive tier assignment with failure-mode coverage across topology, harness, recovery and deployed operation.
