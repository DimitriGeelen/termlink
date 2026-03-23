# T-233 Q5: Independent Orchestration Layer — Pro/Con Analysis

**Question:** Should specialist agent orchestration be a standalone layer between TermLink (transport) and the framework (governance)?

## The Proposal

A separate orchestration engine — call it `agent-mesh` — that:
- Has its own manifest format defining specialist types, capabilities, and routing rules
- Uses TermLink purely as transport (send/receive/discover)
- Respects framework governance (tasks, tiers, context budget) via callbacks/hooks
- Manages specialist lifecycle independently (spawn, health-check, retire)

## Arguments FOR Independence

### 1. Portability Across Projects
An independent orchestration layer could be used with *any* project that has a CLAUDE.md, not just TermLink-based ones. A team using SSH tunnels instead of TermLink could swap the transport. A team without the agentic framework could skip governance hooks. The orchestration logic — routing queries to specialists, managing capacity, merging results — is universally useful.

### 2. Clean Dependency Boundaries
Today: TermLink knows nothing about "specialists" and the framework knows nothing about "routing." An independent layer keeps both ignorant of orchestration concerns. TermLink remains a general-purpose terminal communication tool. The framework remains a governance system. Neither accumulates orchestration-specific code that bloats their APIs.

### 3. Independent Versioning and Testing
Orchestration logic evolves faster than transport protocols or governance rules. An independent layer can iterate on routing algorithms, specialist manifests, and load-balancing without touching TermLink's stable RPC interface or the framework's enforcement hooks. Testing is cleaner: mock TermLink, mock framework, test orchestration in isolation.

### 4. Reusability Beyond Claude Code
If the orchestration layer is transport-agnostic, it could work with other agent runtimes (Cursor, Windsurf, custom SDKs). The specialist concept — "route this query to the agent best equipped to answer it" — isn't Claude-specific.

## Arguments AGAINST Independence

### 1. Another Moving Part
Three systems to install, configure, version, and debug instead of two. The orchestration layer needs its own config files, its own error reporting, its own health checks. For a single-developer project, this is overhead that may never pay off.

### 2. Integration Complexity
The layer must bridge two APIs: TermLink's session/message model and the framework's task/context model. Every abstraction boundary is a potential failure point. Message format translation, error propagation across boundaries, and distributed state (which layer owns "specialist X is busy?") add real complexity.

### 3. Premature Abstraction Risk
We have zero users of orchestration outside this project. Building for portability before proving the concept works here risks designing the wrong abstractions. The framework's dispatch protocol already exists and is tested across 230+ tasks.

### 4. Governance Enforcement Gaps
If orchestration is independent, the framework can't structurally enforce rules on specialist behavior — it can only hope the orchestration layer calls the right hooks. An embedded approach lets the framework's existing PreToolUse/PostToolUse hooks govern specialist actions directly.

## Recommendation

**Start embedded, extract later.** Build orchestration as a framework feature first (using TermLink for transport). Once the routing logic, manifest format, and lifecycle management stabilize after 20+ real tasks, extract into an independent layer with proven interfaces. This follows the "three instances before abstraction" rule and avoids premature generalization.

The portability argument is compelling but premature. The integration complexity argument is real and immediate. Independence is the right *destination* but not the right *starting point*.
