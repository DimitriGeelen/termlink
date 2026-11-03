I have written the consultation review to [antigravity.md](file:///tmp/tl-dispatch/o3-antigravity-316b334c8c5c/tree/docs/reports/T-3353-O3-consultation/antigravity.md).

---

# Independent Consultation: Interim "Main" Session Eligibility

## (1) Choice and Why

**Choice: A (Any interactive session may claim; first to claim wins; operator pin overrides).**

Option A is the best interim rule because the lease mechanism already addresses the core hazard (uncoordinated multi-session mail consumption), provided lease liveness tracks operator activity. In the common single-session case, A delivers mail immediately without manual setup. In multi-session cases, exclusive leasing plus prompt-driven claim attempts and operator pinning gives a clean override path.

Option B is ineffective: because all sessions in a project share one agent ID in practice, filtering by address-book agent ID provides zero discrimination between an active terminal and a 17-day-old forgotten session, while introducing a silent failure mode whenever the address-book entry is unset. Option C prevents stale-session capture only by breaking default liveness on every project start or session restart.

## (2) Strongest Argument Against This Choice

Option A is **fail-open**, whereas the planned target architecture (eligibility profiles defaulting to `"no"`) is **fail-closed**. Under A, if an operator opens a secondary interactive session for a quick side task—or if an older session retains the lease—time-sensitive peer mail (like a password request) can surface in a terminal window the operator is no longer watching. Mail waiting in an unpinned queue (Option C) is a recoverable backlog; mail surfaced in the wrong open terminal appears "delivered" to the hub while remaining invisible to the human.

## (3) Failure Modes in Each Option

- **Option A (Any interactive session claims):**
  - *Heartbeat hoarding:* If "the holder renews it" is a background timer rather than tied to human prompt activity, a forgotten 17-day-old session that started first will renew its lease indefinitely, blocking newer sessions unless pinned.
  - *Scratch-session capture:* Opening a temporary interactive session when no lease is held immediately claims main, routing mail to a throwaway window.
  - *Slow failover:* After an active session goes idle, another session waits up to 30 minutes (15 min lapse + 5 min quiet + 10 min cooldown) before automatic takeover unless pinned.

- **Option B (Address-book agent ID only):**
  - *False discrimination:* Suffers all of Option A's multi-session failure modes because sibling sessions share the same agent ID.
  - *Silent black hole:* Missing or mismatched address-book entries silently prevent any session from becoming main.

- **Option C (Explicit operator pin only):**
  - *Routine delivery stalls:* Unpinned projects, or projects whose pinned session exits and restarts, silently halt mail delivery until manually re-pinned.
  - *Pin fatigue:* Operators will automate pinning in launch wrappers, collapsing C back into A.

## (4) What the Options Miss

1. **Human-activity renewal & preemption:** Lease renewal must require recent human input, never an idle background heartbeat. A human prompt in session $S_2$ when $S_1$ has been idle for $N$ minutes should preempt $S_1$ faster than the 30-minute lapse/quiet/cooldown window.
2. **Re-queue on lease transfer:** Surfacing mail should not mark it handled; if a lease lapses before acknowledgement, the message should re-surface in the new main session.
3. **Cross-session visibility:** Non-main interactive sessions should show a status banner when peer mail is queued or held by another TTY.
