1. **Decision Recommendation**  
1a. **Preferred Option**: **C** (Split by layer).  
1b. **Strongest Argument Against**: The complexity of ensuring AEF’s direct
direct-call receivers reliably mirror events to the hub (especially when th
the hub is unreachable). This introduces a new failure mode (buffering, dat
data loss during outages) that must be explicitly managed.  

2. **Hub as Telemetry Home**  
2a. **Is the hub the right home?**: Yes, **conditionally**. The charter exp
explicitly avoids making TermLink a "system of record," but telemetry is no
not a system of record—it is a *log of events* for debugging and coordinati
coordination, not a primary data store. The hub can host telemetry as long 
as it is treated as a *complementary* log (not the sole source of truth). T
This aligns with the charter’s goal of being a "durable append-log message 
bus."  
2b. **Alternative**: If the hub were not suitable, telemetry could live in 
a shared, lightweight log (e.g., a Kafka topic or a separate append-only st
store). However, this would violate the "no second cross-host bus" non-goal
non-goal, as it would create an independent telemetry bus.  

3. **Failure Modes per Option**  
3a. **Option A (AEF owns all telemetry)**:  
- **Breaks first**: When AEF’s HTTP receiver fails or is unreachable (e.g.,
(e.g., due to a bug or misconfiguration).  
- **Detection**: Missing timestamps in `.context/sidecar/direct-ack.jsonl` 
for messages sent during the outage.  

3b. **Option B (TermLink owns all telemetry)**:  
- **Breaks first**: When AEF’s direct HTTP path (its primary communication 
method) cannot be integrated into TermLink’s binary.  
- **Detection**: AEF’s agents report "RECEIVED" but no "INJECTED" or "REPLI
"REPLIED" events, leading to silent stalls.  

3c. **Option C (Split by layer)**:  
- **Breaks first**: When AEF’s mirroring process fails (e.g., buffer overfl
overflow, hub downtime).  
- **Detection**: Hub telemetry logs show missing stages (e.g., "INJECTED" n
never appears) for messages that AEF’s direct calls claim to have processed
processed.  

4. **R3: Minimal Record and Digest Requirements**  
4a. **Record (per-message)**:  
- `client_msg_id`: Unique identifier for the message.  
- `stage`: Enum (RECEIVED, INJECTED, REPLIED, etc.).  
- `timestamp`: ISO 8601 timestamp.  
- `from_project`, `to_project`: Identifies the origin and destination of th
the message.  
- `error_details` (optional): If the stage is "UNDELIVERABLE" or "REJECTED,
"REJECTED," includes a brief reason.  

4b. **Daily Digest (per-agent)**:  
- Aggregated statistics: Number of messages per stage, average delay betwee
between stages, error rate.  
- Top 5 slowest messages (by delay) and their `client_msg_id`.  
- Summary of "REPLIED" vs. "UNDELIVERABLE" events.  

5. **Missing Framing Elements**  
5a. **Risk**: **Lack of fallback for hub-unreachable scenarios**. Under Opt
Option C, AEF’s mirroring process must buffer events locally and retry inde
indefinitely, but this is not addressed in the current framing.  
5b. **Unstated Requirement**: **Cross-agent telemetry interoperability**. T
The system assumes all agents use TermLink’s telemetry format, but if other
other projects use incompatible formats, the "pullable" requirement (R3) be
becomes unactionable.  
5c. **Unstated Option**: **Hybrid ownership** (e.g., TermLink handles "RECE
"RECEIVED" and "INJECTED" stages, AEF handles "REPLIED"). This could reduce
reduce coupling but requires a clear contract between TermLink and AEF.
