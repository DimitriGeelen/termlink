## 1. Choice and Argument

**Choice:** Option C.

**Strongest Argument Against Choice:**
The primary weakness of Option C is the "Asynchronous Mirror" requirement f
for direct-call paths. It fundamentally violates the simplicity and reliabi
reliability of the direct-call protocol (AEF's current primary path). Requi
Requiring a high-throughput, low-latency direct HTTP call path to asynchron
asynchronously report its internal state to the hub introduces mandatory co
complexity, increases failure surface (mirror-failure, hub-down handling), 
and introduces potential race conditions. This creates a "non-local" depend
dependency that the original design attempted to avoid.

## 2. Telemetry Home

The hub is not the right home for cross-agent telemetry, given the Charter'
Charter's explicit non-goals.

1. The hub is designed for transport and transient messaging. A comprehensi
comprehensive telemetry record (full lifecycle history, every stage, every 
timestamp) constitutes a "system of record" regarding agent interactions. P
Placing this on the hub risks conflating operational logs with data transpo
transport, potentially violating retention policy boundaries or performance
performance guarantees for message queuing.
2. The hub does not require "read" access by agent decision-making processe
processes; it requires "operational visibility."

**Proposed Home: A dedicated "Operational Observability Layer" (OOL).**

1. The OOL should be a specialized, append-only data store optimized for ti
time-series data and writes (e.g., ClickHouse, specialized Kafka stream).
2. Telemetry events should be published to the OOL *independent* of the mes
message bus.
3. TermLink (or a dedicated sidecar) would provide a PULL interface to the 
OOL, fulfilling R2 and R3, but this is separate from message transit.

## 3. Failure Modes

Failure modes must be evaluated against the current lack of visibility (Inc
(Incident: "a peer agent waited ~1 day for answers; the TermLink sidecar ha
had receipted its mail as delivered, but nothing put it in front of the age
agent").

**Option A (AEF Receiver Own State):**
1. **Mode:** Local State Loss. If the agent/project's receiver sidecar or l
ledger fails catastrophically, the entire history of that message's lifecyc
lifecycle is lost.
2. **Detection:** Monitoring the local disk health and the "Agent Response 
Rate" externally. No cross-project correlation is possible.

**Option B (TermLink Own State):**
1. **Mode:** Hub Bottleneck / Single Point of Failure (SPOF). If TermLink b
becomes overloaded by the sheer volume of state updates (RECEIVED, INJECTED
INJECTED, etc.), its primary messaging function degrades or fails.
2. **Detection:** Increased end-to-end latency or persistent queue backlogs
backlogs reported by TermLink itself. The system becomes "slow" rather than
than "silent."

**Option C (Split/Hybrid State):**
1. **Mode:** Event Desynchronization. Since direct paths must asynchronousl
asynchronously mirror to the hub, if the direct path is fast but the mirror
mirroring path is slow, the telemetry record in the hub will be incomplete 
or lagged behind the actual agent state.
2. **Detection:** The agent must continuously check the hub record against 
its local authoritative state. Any delta flagged (e.g., Hub says `RECEIVED`
`RECEIVED` but local state says `PROCESSING`) triggers a telemetry integrit
integrity alarm.

## 4. Actionable Telemetry Content (R3, R4)

To allow agents to "reflect on what it means," the data must move beyond si
simple event logging and provide calculated intervals and environmental con
context.

1. **Minimal Core Record:**
    *   `event_id`: Unique ID for the step (e.g., `received_123`).
    *   `timestamp_utc`: Absolute time of the event.
    *   `event_type`: Standardized stage (e.g., `RECEIVED`).
    *   `source_project_fp` / `target_project_fp`.
    *   `previous_event_id`: Link for chain construction.
2. **Calculated Observability Fields (Critical for Agent Action):**
    *   `delay_since_previous_stage`: Duration (in seconds/milliseconds) be
between this event and the immediately preceding recorded event.
    *   `latency_to_hub_observer`: The total duration of the message from o
originating sidecar event to central telemetry recording (useful for detect
detecting bottlenecks in the OOL/Hub).
3. **Daily Digest Contents:**
    *   A time-series summary view showing `Mean Delay` per recipient and `
`P95 Delay` per event type.
    *   A breakdown of delay causes (e.g., "Average delay between `RECEIVED
`RECEIVED` and `INJECTED` was 54 seconds," implying system scheduling overh
overhead).

## 5. Missing Framing Element

The most critically missing element is a defined **State Machine Contract**
Contract** for the agents themselves.

The framing focuses heavily on how messages *move* and how *delivery* is tr
tracked, but it does not define the behavioral contract of the recipients.

1. **Agent Obligation:** Does an agent have a requirement to *acknowledge* 
when it has been injected? If an agent fails to process a message after `IN
`INJECTED`, is that a failure?
2. **Actionability Link:** If the goal is to let "agents work with that tel
telemetry," there is no stated requirement for agents to generate their own
own telemetry—e.g., "I am spending 30 seconds executing this logic." The cu
current requirement only measures the *transport* stages (sidecar to sessio
session), not the *processing* stage (within the agent logic). Without this
this link, the telemetry is merely an audit log, not a reflection tool.
