# Codex review — OD-18 (2026-10-06)

1. **[CHANGE] Define the lease as exclusive authority, not proof of agent health.** Sidecar renewal proves only that the sidecar can reach the authority; readiness needs separate evidence of successful delivery, acceptance, and progress on owed answers. The 32,205 refusals should have triggered bounded retries, an unhealthy state, and escalation.

2. **[CHANGE] Separate availability from execution progress.** A long task can legitimately retain “main” if the agent acknowledges coordination requests and reports progress; a fresh heartbeat must not hide an inability to receive them. Define acceptance deadlines and explicit busy/draining states rather than treating every slow answer as death.

3. **[CHANGE] Make health-triggered replacement an explicit authority transition.** Use sustained failure thresholds and a recovery cooldown to limit flapping; if renewal continues despite failed readiness, the home hub must revoke or atomically transfer authority and advance the generation. Otherwise the unhealthy sidecar can retain the role indefinitely.

4. **[CHANGE] Fencing prevents accepted stale actions, not competing beliefs.** Validate `(project, role, generation, holder-instance)` against current authority at every role-protected effect boundary—including obligation updates, delegated commands, and external writes—and reject operations when authority cannot be established; checking only introductions leaves already introduced stale holders dangerous.

5. **[MISSING] Some effects cannot be fenced.** An email, shell command, or third-party operation may already be underway or lack generation validation; route such effects through an enforcing gateway where possible, otherwise require reconciliation before replay and explicitly acknowledge that failover cannot guarantee exclusive effects.

6. **[CHANGE] Specify durable generation and transfer semantics before adding failover.** Acquisition, revocation, expiry, transfer, and generation advancement must be serialized at the home hub; restarts or backup restoration must never reuse an authority version that downstream systems could still accept.

7. **[CHANGE] Reconcile fencing with OD-10 conversation binding.** Losing “main” must remove role authority without automatically invalidating ordinary replies in an existing instance-bound conversation; conversely, an old conversation binding must not preserve permission to perform current-main actions. Classify operations by authority required.

8. **[CHANGE] “Introduce, then step aside” needs a narrower promise.** The resolver can leave the conversation transport path, but authority checks and durable obligation tracking remain necessary; handle expiry between resolution and acceptance by re-resolving an unaccepted new request, never by redirecting an established conversation.

9. **[CHANGE] Transfer responsibility for recovery, not blindly executable work.** Role obligations need durable request IDs, acceptance state, checkpoints, effect receipts, and deduplication; after failover, classify each as unstarted, safely resumable, completed, or uncertain. Half-done or uncertain work requires reconciliation before another agent repeats it.

10. **[MISSING] Define exactly when a role request becomes conversation-bound.** Record acceptance and binding atomically with obligation ownership so failover cannot both transfer the same owed answer and leave it assigned to the original instance; dead-lettering should preserve a visible unresolved outcome and notify the appropriate party.

11. **[CHANGE] Select through one authoritative allocation rule.** Keep a healthy incumbent; otherwise let the home hub atomically choose the highest operator-configured priority among authorized, ready candidates, breaking ties by stable agent ID and then instance ID. Explicit capability authorization is stronger than merely excluding seats labelled “worker” or “reviewer.”

12. **[MISSING] Support operator pinning with explicit fallback semantics.** Offer a strict pin, which leaves the role unavailable if its candidate fails, and a preferred candidate with ordered fallbacks; avoid automatic preemption when a preferred agent recovers, and show the operator the active policy.

13. **[AGREE] Home-hub unavailability must not authorize another hub to take over.** Show authority as unknown, bound retries with backoff, and fail closed for role-protected actions; separately decide whether already bound conversations may continue operations that require no current role authority.

14. **[MISSING] Cover unattended projects, host boundaries, and the human party.** Zero eligible candidates should produce a visible pending/unavailable state and operator escalation; start or resume only under OD-15 grants. Remote lookup must use verified project identity and canonical home-hub identity, not local discovery guesses.

15. **[MISSING] Make coordination inspectable and escalation independent of “main.”** Show holder, exact instance, host, generation, readiness, pending answers, and failover reason; OD-14 needs a durable project-level escalation record and human fallback when no main exists, plus authenticated, audited operator override.

16. **[CHANGE] Remove premature generality.** Implement and validate “main” before arbitrary singleton roles or worker pools; globally exclude broadcast races, but do not reject first-to-claim itself when one authority performs an atomic claim.

**Overall verdict:** Keep the home-hub resolver and exclusive role authority, but require readiness checks, enforceable fencing, and durable recovery semantics before calling this safe failover.
