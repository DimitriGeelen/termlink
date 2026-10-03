This looks like a critical review and architectural synthesis phase for the
the project. Based on the log, here is a summary of the current state of th
the "long-term road" and the implications of the recent decisions (up to 20
2026-10-04).

### 🛣️ Current State of the Long-Term Road (T-3396 Focus)

The current roadmap is moving away from quick, tactical fixes toward a soli
solid, highly durable, and standardized architecture. The core decisions re
revolve around robust agent behavior, standardized telemetry, and predictab
predictable timing mechanisms.

#### 1. Core Infrastructure & Reliability (The Foundation)
*   **Addressing & Routing (D-599, D-645, D-660):** The addressing scheme i
is solidified around durable roles and specific circuit IDs (`inbox:<circui
(`inbox:<circuit-id>` with five levels). There is a commitment to move towa
toward the **T-3397 design**, utilizing the sidecar API for blob movement, 
moving past initial `sidecar:` addressing toward more defined inputs (`dm:`
(`dm:`/`inbox:`).
*   **Failure & Retry Strategy (D-600):** A universal, tiered retry ladder 
is established (2x1 minute $\rightarrow$ 2x1 month), providing predictable 
recovery mechanisms.
*   **Agent Persistence (SQ-5):** The installation of the `wake-supervisor`
`wake-supervisor` cron is slated to handle agent lifecycle management.

#### 2. Agent Behavior & Timing (The Engine)
*   **Injection/Wake-up (T-3396, T-3330, Watcher):**
    *   The long-term goal involves **PTY injection on idle**, indicating a
a move toward event-driven rather than constant polling states.
    *   Timing is governed by a **30-second tick/watch** (`watcher.py`, T-3
T-3330), where urgent messages bypass this tick for immediate injection.
    *   **Polling Standards (ladder):** The standard fallback is a polling 
ladder (15s $\rightarrow$ 1 year, each rung twice), providing a "standard f
fallback mechanism" for any activities where immediate injection isn't feas
feasible.
*   **Data Integrity & Consistency (SQ-4):** The rule was set that **urgent
**urgent messages never inject into a busy prompt**, ensuring state consist
consistency, although this specific rule required later confirmation.

#### 3. System Standardization (The Framework)
*   **Telemetry (T-3330):** Telemetry is mandatory and will be the standard
standard for *all* vendor agents, ensuring observability across the entire 
ecosystem.
*   **Deployment Model (T-3330):** Every sidecar component is now mandated 
to ship alongside the primary deployment, ensuring components are never dep
deployed out of sync.
*   **Context/Capacity (CONTEXT_WINDOW):** The working session limit is cap
capped at 900k tokens (950k is the design upper bound).

---

### 🧭 Synthesized "Long-Term Road" Plan

Based on this synthesis, the remaining components needed to "pack it full o
out" and fully realize the long-term vision are:

1.  **Finalize T-3397 Implementation:** Fully integrate the cross-host/urge
cross-host/urgent bypass architecture into the live deployment environment.
environment.
2.  **Validate Idle PTY Injection:** Successfully test and oper
operationalize the PTY injection logic to replace/supplement aggressive pol
polling (moving from "tick" to "event-driven idle").
3.  **Finalize Naming Conventions:** While two identities per level is in p
place, the full consensus needs to be formalized to manage the `pid` and pr
project slot mapping (T-3751 IW-1) across all vendors.
4.  **Scale and Test Durability:** Validate the 5-level addressing and dura
durable role mechanisms under maximum load to confirm the resiliency and pr
prevent race conditions during state transitions.

**Status Summary:** We have the blueprint for the system behavior (how it t
thinks, how it waits, and how it recovers). The focus is now on the deploym
deployment and validation of the finalized routing and event-trigger mechan
mechanisms.
