## 1. Overall  
**Verdict:** The design partially achieves the goal of interactive two-way 
conversation between agents, but **the biggest weakness is the inconsistent
inconsistent handling of urgency and readiness detection**. While SQ-4 expl
explicitly prohibits urgent injection into busy prompts (ARC11:305-316), th
this was later contradicted (2026-10-03, not recorded as superseded), creat
creating ambiguity. The 30-second watcher (watcher.py:3-6) and "urgent imme
immediately" rule (CONSULT:17) attempt to address this, but the unresolved 
contradiction risks system instability during critical operations.  

---

## 2. Gaps and Contradictions  
- **Contradiction in urgency logic (SQ-4):** The rule "urgent never injects
injects into a busy prompt" (ARC11:305-316) is explicitly contradicted on 2
2026-10-03, yet the document does not mark this as superseded. This creates
creates untestable ambiguity in urgency behavior.  
- **Addressing conflicts (D-599 vs D-660):** D-599 mandates `inbox:<circuit
`inbox:<circuit-id>` with durable roles, but D-660 (2026-09-27) adopts `dm:
`dm:`/`inbox:` over `sidecar:`, explicitly rejecting "Option 3 (support bot
both)" (decisions.yaml:4617-4622). This contradicts D-599’s requirements an
and may break compatibility with prior implementations.  
- **Naming model ambiguity (naming entry):** The 7-vendor review (IN-010@25
(IN-010@250; `T-3751-review-brief.md:105-108`) is described as a review, no
not a ruling, and no operator ruling on "2a" is recorded. This leaves the n
naming model incomplete and untestable.  
- **Missing fallback for CONTEXT_WINDOW:** While `TOKEN_CRITICAL` is define
defined (T-3332), the document does not clarify how to handle exceeding the
the 900k token limit, leaving a gap in session termination logic.  

---

## 3. The Hardest Parts  
### **Readiness Detection (Harness Stop/Prompt Hooks vs Screen Inspection)*
Inspection)**  
**Solution:** Implement the 30-second watcher (watcher.py:3-6) to monitor m
message flags and trigger readiness checks. Use the "urgent immediately" ru
rule (CONSULT:17) to bypass readiness checks for urgent messages, but docum
document this as a temporary exception.  

### **Urgent Bypass into a Busy Agent (SQ-4 Contradiction)**  
**Solution:** Reconcile the contradiction in SQ-4 by explicitly defining a 
fallback mechanism (e.g., "urgent bypass only if the agent is unresponsive 
for >15s"). Log all bypassed urgent messages to track conflicts.  

### **Making Already-Running Sessions Reachable**  
**Solution:** Use the sidecar API (D-645: blobs through the sidecar API) to
to maintain session state and relay messages. Implement a heartbeat mechani
mechanism (e.g., 15s ping) to confirm agent availability and queue messages
messages if the agent is busy.  

### **Cross-Host Send Path vs "No Second Bus" Charter Rule**  
**Solution:** Route all cross-host communication through the sidecar API (D
(D-645) and avoid duplicating infrastructure. Use the "durable role" logic 
(D-599) to ensure redundancy without violating the "no second bus" rule.  

---

## 4. Open Decisions  
### **Naming Model (naming entry)**  
**Recommendation:** Finalize the naming model by adopting "two identities p
per level" (operator proposal) and explicitly defining "2a" (e.g., "project
"project canonical id = minted `pid`", "project slot = minted id"). This en
ensures clarity for vendor integration (IN-010@250).  

### **Urgency Logic (SQ-4 Contradiction)**  
**Recommendation:** Amend SQ-4 to define a clear urgency fallback (e.g., "u
"urgent bypass only if the agent is unresponsive for >15s"). Record this as
as a superseded rule to resolve the contradiction.  

---

## 5. Build Order  
**First:** Implement the sidecar API (D-645) to enable cross-host communica
communication and backfill (SQ-6: backfill once, recorded as backfill).  
**Next:** Deploy the 30-second watcher (watcher.py:3-6) and urgency bypass 
logic (CONSULT:17).  
**Acceptance Test:** Two real agents (A and B) exchange messages via the si
sidecar API. Agent A sends a message to B; B confirms receipt, processes it
it, and sends a response. A negative control: Agent B is busy; urgent messa
message from A is bypassed and logged.  

---

## 6. Risks and Failure Modes  
- **First Break:** Urgent message injection into a busy agent (SQ-4 contrad
contradiction). Detection: Monitor for unprocessed urgent messages and log 
mismatches between SQ-4 and the 2026-10-03 ruling.  
- **Second Break:** Addressing conflicts (D-599 vs D-660). Detection: Test 
`inbox:<circuit-id>` and `dm:`/`inbox:` compatibility; failure to route mes
messages via sidecar: would indicate a conflict.  
- **Third Break:** Naming model ambiguity (naming entry). Detection: Vendor
Vendor agents fail to recognize `pid` or minted identities; traceability br
breaks.  

---

## 7. What Is Missing from the Framing  
- **Risk:** No explicit fallback for CONTEXT_WINDOW exceeding 900k tokens (
(T-3332). If sessions exceed `TOKEN_CRITICAL` (902k), the system may crash 
without defined behavior.  
- **Option:** Lack of authentication/authorization for agent-to-agent commu
communication. The design assumes trust but does not address security betwe
between peers.  
- **Gap:** No redundancy plan for the sidecar API (D-645). If the sidecar f
fails, cross-host communication would break.
