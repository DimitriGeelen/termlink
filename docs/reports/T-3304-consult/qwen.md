### 1. Queryable Source of Truth vs. Transport  
The hub should **remain a transport**, not a queryable source of truth. The strongest argument against making it a source of truth is **scalability and operational complexity**. Querying arbitrary data across topics would require maintaining rich indexes (e.g., full-text search, time-range filters, cross-topic joins), which would drastically increase storage, CPU, and latency overhead. For a system designed for **coordination**, not archival, this would contradict its core charter. Agents should manage their own state and history, while the hub focuses on **reliable message delivery** and **minimalistic durability**.  

### 2. Retention and Pruning Model  
If the hub **must** retain history (contrary to its charter), the model should be **automatically pruned with time-based retention** (e.g., N days) as the default, with exceptions for critical topics (e.g., health probes). Operators should set retention policies per topic, but **automatic sweeps** (not manual cron jobs) would reduce operational burden. This balances durability with resource constraints. However, the current design’s explicit pruning is a **non-goal** that should be preserved: **manual control** is better for systems where retention policies are unpredictable or require human oversight.  

### 3. Subscription and Query Model  
"Cursor + optional filter" is **inadequate** for modern use cases. **Time-range queries** are essential for agents needing to process historical data (e.g., diagnostics, audits). **Server-side indexes** on metadata (e.g., `conversation_id`, `metadata.cv_key`) would improve filter performance, but the current system’s 20s deadline for filtered reads is a **critical bottleneck**. **Pagination/chunking rules** (e.g., limit=1000, offset=0) should be mandatory to avoid overwhelming clients. **Cross-topic queries** should be **avoided** entirely, as they violate the hub’s transport-only charter. The first necessary addition is **time-range queries**; **cross-topic queries** should never be implemented.  

### 4. Topic Discovery and Selection  
Topics should be **discovered via a centralized registry** with **tags, descriptions, and metadata**, not just name prefixes or regex. A **registry service** would allow agents to query topics by relevance (e.g., "all topics related to health monitoring"). Naming conventions should follow **semantic categories** (e.g., `fleet/control`, `agent/health`) to avoid chaos. Agents should **subscribe via a registry API**, not manually list topics, to ensure discoverability and reduce errors. **Tags** and **descriptions** are mandatory for any topic with public relevance.  

### 5. Cost Implications  
- **Storage**: Retaining history increases storage costs exponentially. For tens of agents, **bounded retention** (e.g., N days) is safer than "forever."  
- **CPU**: Filters, time-range queries, and indexes consume CPU. **Avoiding cross-topic queries** and limiting filters to metadata fields (not full-text) keeps CPU usage low.  
- **Complexity**: Queryable features add complexity. **Stick to transport-only** to minimize it.  
- **Failure Modes**: Manual pruning and lack of federation increase risks. **Automate pruning** and **consider federation** (see below). For a small fleet, **bounded retention** and **cursor-based subscriptions** are sufficient.  

### 6. Hidden Mistake: No Federation  
The **absence of hub federation** is the most critical oversight. While the charter explicitly avoids federation, this design choice **limits scalability** and **prevents cross-machine coordination**. In a fleet spanning multiple machines, agents on different hubs cannot share topics or coordinate work. This violates the charter’s goal of enabling a **fleet of AI agents to discover each other**. Federation (e.g., via a **centralized topic registry** or **overlay network**) should be mandatory, even if it complicates the system. The current design’s refusal to federate is a **fundamental flaw** that undermines its own charter.

