# Codex review — GP-12 (2026-10-06)

1. [AGREE] These proxies improve deployment and inspection, but mostly measure packaging; the recorded failures concern identity binding, exclusive inbox ownership, routing consistency, and actual delivery progress.

2. [CHANGE] Failures **1, 3, and 5** could pass every structural threshold and a superficial status check: a compact, installed receiver can use the wrong key, exclude an agent, or contact the wrong hub; a truthful, behaviorally verified proxy 3 would expose them.

3. [CHANGE] Failure **2** violates a strictly enforced one-process limit, but can recur through two consumers inside one process or receivers on different hosts; require exclusive inbox ownership with fencing that prevents an obsolete owner from consuming or acknowledging messages.

4. [CHANGE] Failure **4** is excluded only if the process count covers the entire required receive path; moving injectors, mirrors, or wakers into shared services, plugins, or subprocesses must not remove them from the assessed dependency graph.

5. [CHANGE] “One status call, one screen” measures presentation, not truth: report separately verified receiver liveness, canonical hub identity, inbox ownership, signing identity, adapter availability, and last successful hand-over, each with freshness and explicit unknown states.

6. [MISSING] Reachability needs a bounded end-to-end probe through the production delivery path to the agent adapter, independent of an LLM turn; a heartbeat or successful enqueue cannot establish that hand-over works.

7. [CHANGE] Make the five historical failures standing fault-injection controls: wrong key binding, competing ownership, omitted wake registration, failed receive dependencies, and wrong runtime directory must produce the expected delivery outcome and a specific diagnosis within a declared time bound.

8. [CHANGE] A single process hiding five threads is not inherently gaming—threads may simplify coordination; assess independently failing components, queues, ownership boundaries, and recovery protocols instead of treating operating-system process count as a reliability measure.

9. [CHANGE] Under R-6’s explicit wording, a per-host receiver cannot replace per-agent sidecars without an operator amendment; technically, it may simplify installation and supervision while increasing shared failure impact, contention, and cross-agent identity risk.

10. [MISSING] Compare per-agent and per-host designs under equivalent load and injected failures, including one stalled adapter, one invalid key, and a host receiver restart; judge isolation, recovery time, resource cost, and operator effort.

11. [CHANGE] “One install command” is easily satisfied by a wrapper around fragile setup; require installation into a clean supported environment outside the checkout, deterministic identity provisioning, hub validation, and tested upgrade, rollback, and restart behavior.

12. [CHANGE] Ten API calls is language-neutral but semantically weak: one generic dispatch call can hide unlimited operations; inventory distinct operations, modes, privileges, and required call sequences, then remove those unsupported by contract needs.

13. [CHANGE] A 1,000-line cap is not comparable across shell and Python and invites code relocation or compression; count the complete maintained receive implementation consistently and use growth only to trigger review, never to establish reliability.

14. [MISSING] Require an explicit state and persistence model: delivery states and transitions, authoritative stores, derived files, identity/key bindings, retry ownership, and acknowledgement boundaries; test crash recovery at transitions where loss or duplication could occur.

15. [MISSING] Measure operational simplicity directly through automatic recovery time, manual recovery actions, and timed diagnosis exercises using unfamiliar operators; “why didn’t my message arrive?” should resolve to evidence-backed causes without reconstructing several logs.

16. [CHANGE] Neither 10 calls nor 1,000 lines is presently defensible as acceptance: derive provisional review thresholds from minimal conforming implementations and operation inventories, while deriving delivery, detection, and recovery limits from operator needs and validating them against standing controls.

Overall verdict: retain the proxies as review aids, but make verified delivery invariants, fault isolation, and bounded diagnosis and recovery the acceptance criteria for “very simple.”
