## The System Review

The current framing is dangerously close to a **data capture fetish**. The 
operator is treating the message bus (TermLink) not as a coordination substrate, but as an unexploited data lake. While leveraging existing data 
for introspection is valuable, this endeavor must not be allowed to become 
an architectural burden that violates the core "Non-goal: Not a durable database or system of record." To enable "learning" by examining "raw material" might require moving the system away from being a simple log bus 
toward becoming a complex, managed data platform—a form of creeping persistence—which fundamentally changes the operational profile and increases cost and complexity unnecessarily.

***

## 1. Is this worth doing at all?

**Recommendation: Proceed with extreme caution, focused initially on high-signal signal-to-noise ratio (SNR) measurements.**

The primary argument *for* doing this is that AI agents are "dumb" executors, and their fleet performance is intrinsically linked to observable patterns in the coordination layer. Metrics like latency (reply 
time) or churn (re-engagement rates) can inform the agent’s internal scheduling logic, moving it from reactive to proactive.

The strongest argument *against* it is the **Observer-Effect/Metric Overhead.** The data itself is currently noisy (fingerprint mapping, inconsistent metadata) and the required analytical work (processing 4,367 records, filtering heartbeat pollution) is disproportionately high relative to the actionable insight. Forcing the creation of derived views, 
metrics, and monitoring layers imposes runtime overhead and cognitive load 
without necessarily delivering *new* intelligence that AEF (which already learns from its own artefact) cannot provide through better self-reflection.

***

## 2. Options

### Approach A: AEF-Side Batch Miner (Export/Analyze)
*   **What it produces:** High-level, retrospective statistics (e.g., "Top 
5 repeating bug categories in the last 30 days," "Average time between problem discovery and proposal.")
*   **Cost:** Medium (Requires dedicated computation for export and analysis).
*   **Failure modes:** Lacks real-time utility. Dependent on the log being 
accessible before trim, risking data loss during the initial analysis window.

### Approach B: Hub-Side Derived Metrics (Pre-Aggregation)
*   **What it produces:** Simple, low-overhead dashboards and counters (e.g., "Unanswered/Unread count per key topic," "Heartbeat rate variance per agent group.")
*   **Cost:** Low to Medium (Requires new read/write logic on the hub).
*   **Failure modes:** Risks compromising the system's core "append log" performance if the aggregation logic is poorly implemented. May still struggle with semantic understanding (i.e., "is this a bug or a heart beat?").

### Approach C: Capture-then-Learn Pipeline (Snapshot before trim)
*   **What it produces:** A curated historical dataset used for training small predictive models (e.g., "This failure class often leads to this outcome").
*   **Cost:** High (Requires robust snapshot/export logic and dedicated data engineering).
*   **Failure modes:** Requires building a durable database, directly challenging the charter non-goal. Risk of becoming a secondary, brittle system of record.

### Approach D: Producer-Centric Improvement
*   **What it produces:** Better, more stable data quality (consistent fingerprints, mandated metadata schemas).
*   **Cost:** Low (Requires policy enforcement).
*   **Failure modes:** Does not solve the underlying problem of lack of insight; it only optimizes data quality for *any* future analysis.

***

## 3. Where should derived knowledge live and who owns it?

Derived knowledge should live **in AEF Project Memory**, but only in the form of **actionable, formalized findings** (i.e., refined patterns or warnings).

*   **Why not the Hub?** Hub is for transit/coordination, not storage/analysis.
*   **Why not a Separate Store?** It creates undue complexity, storage costs, and maintenance overhead (violating "Not a durable database").

**Ownership:** AEF (the governance framework) should own the derived knowledge, but only if the insights are codified, deduplicated, and incorporated into the `learnings.yaml`. A human operator should only own the *process* of defining what constitutes a valid insight, not the data itself.

***

## 4. Retention Trimming

To decide what must be captured, we must categorize the raw material:

1.  **Signal (Priority 1):** Messages that trigger significant changes in state or require human input (e.g., `bug-report`, `feature-proposal`, high-value `learning` types).
2.  **State Change (Priority 2):** The *last known state* of a topic/thread before trim (e.g., the last message and activity table entry).
3.  **Noise (Priority 3):** Heartbeats, social stats, repeated placeholders.

**How to capture:** Do not build an archive. Instead, use **on-the-fly, event-driven metadata captures**. When a retention policy event is triggered, the system should not copy the whole log, but should capture a small "summary record" containing: the total count, the timestamp of the trim, and the metadata of the *last N* messages.

***

## 5. What to measure first

To decide if the overhead is worth the insight, measure the **Signal-to-Noise Ratio (SNR) of Unresolved Topics.**

**Measurement:** For all topics in the `inbox:*` and `dm:*` folders, calculate (Writes + Replies) / (Fetch Counts).
*   A high ratio suggests a highly active thread/project with meaningful activity.
*   A low ratio, coupled with high "unread since" values, suggests potential project failure, dead threads, or stagnation—a direct, actionable signal that is easily computable from existing activity logs.

***

## 6. One Mistake

The most critical mistake is relying on the **sender fingerprint (per host)** as a basis for attribution and system diagnosis. Because the fingerprint is per host, not per agent, when "Agents are collaborating," the log merely shows "Host X said thing Y," obscuring which specific autonomous agent within that host initiated the logic. This makes attributing failures or successful coordination difficult and limits the fidelity of the "meta-learning" of the AEF.
