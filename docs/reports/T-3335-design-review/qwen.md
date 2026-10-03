The provided table and notes document a series of technical decisions, task
tasks, and configurations related to a system (likely an AI agent framework
framework) involving **AEF** (Agent Execution Framework), **TERMLINK**, and
and other components. Below is a structured summary of key points and conte
context:

---

### **Key Technical Decisions & Tasks**
1. **AEF Roadmap & Design**:
   - **T-3396** (2026-09-20): Focus on "long-term road" with **PTY inject o
on idle**.
   - **T-3397** (2026-09-21): Cross-host communication is live, with **urge
**urgent bypass** enabled.
   - **T-3330** (2026-10-03): 
     - Telemetry is standardized for **all vendor agents**.
     - **Sidecars** are deployed with every deployment.
     - **Urgent-only alarms** and daily digest for events.
     - **Observability database** planned for later.

2. **Circuit & Addressing**:
   - **D-599** (2026-09-22): `inbox:<circuit-id>` addressing with 5 levels,
levels, durable roles, and exact circuit matching.
   - **D-660** (2026-09-27): AEF adopts `dm:`/`inbox:` addressing (not `sid
`sidecar:`), superseding D-599's wording.
   - **Circuit V9 grammar**: Five-part circuit with dual-read capability.

3. **Polling & Retry Mechanisms**:
   - **Ladder** (2026-10-03): Standard polling ladder with intervals from *
**15 seconds to 1 year**, each rung executed **twice**.
   - **Watcher** (2026-10-02): 30-second watcher with **urgent bypass**.

4. **Injector & Prompt Handling**:
   - **SQ-4** (2026-09-23): Urgent injections **never** occur in busy promp
prompts (contradicted on 2026-10-03, but not marked as superseded).
   - **SQ-1** (2026-09-23): Injector remains in **TermLink**, with sidecar 
API as local control.

5. **Backfill & Logging**:
   - **SQ-6** (2026-09-23): Backfill is done **once**, recorded explicitly 
as a backfill.
   - **SQ-5** (2026-09-22): Wake-supervisor cron installed on operator appr
approval.

---

### **Contextual Notes**
1. **Naming Model**:
   - **Two identities per level** (operator ruling, 2026-10-03): Each level
level (e.g., project, circuit) has two identities.
   - **Project canonical ID**: Minted `pid` (unique identifier for projects
projects).
   - **Project slot**: Carries the minted ID (T-3751 IW-1 = C).
   - **7-vendor review**: A review process for naming, but **no operator ru
ruling** on specific naming rules (e.g., "2a" in note 2).

2. **CONTEXT_WINDOW**:
   - **Token budget thresholds** govern agent session limits:
     - **950,000 tokens**: Maximum context window.
     - **900,000 tokens**: Working limit.
     - **`TOKEN_CRITICAL`**: ~902,000 tokens (trigger for stopping tasks).
   - **Not part of communication design**, but critical for session termina
termination policies (T-3192, CLAUDE.md).

3. **Contradictions & Superseded Rules**:
   - **SQ-4** (2026-09-23): Contradicted on **2026-10-03** but not marked a
as superseded.
   - **D-599** (2026-09-22): Superseded by **D-660** (2026-09-27) regarding
regarding addressing schemes.

---

### **Operator Rulings & Agent Actions**
- **Operator rulings** are explicitly documented (e.g., "operator decided: 
urgent NEVER injects into a BUSY prompt").
- **Agent-recorded** actions (e.g., SQ-2, 2026-09-22) are noted when operat
operators approve or confirm changes.
- **Standing directives** (e.g., D-700, 2026-10-02) require ongoing contact
contact with specific components (010-termlink, 055).

---

### **Key Files & References**
- **`decisions.yaml`**: Central repository for operator rulings (e.g., D-59
D-599, D-660).
- **`watcher.py`**: Implements 30-second watcher for urgent bypass.
- **`.tasks/completed/T-xxxx-…`**: Task completion logs (e.g., T-2876, T-33
T-3332).
- **`IN-010@xx`**: Documentation on circuits and addressing (e.g., IN-010@3
IN-010@39, IN-010@53).
- **`T-3330R:xx`**: Review and confirmation of telemetry standards.

---

### **Summary**
This system involves a complex interplay of **AEF**, **TERMLINK**, and **si
**sidecar APIs**, with strict rules for addressing, polling, and session li
limits. Operator rulings and agent actions are meticulously logged, but con
contradictions (e.g., SQ-4) and unresolved naming rules (T-3751) highlight 
areas requiring further clarification. The **CONTEXT_WINDOW** and **polling
**polling ladder** are critical for ensuring reliability and scalability, w
while **telemetry** and **observability** are key for vendor agent standard
standardization.
