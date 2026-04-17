# T-1101: Deep Value Assessment — What Does TermLink Actually Give the Operator?

## Problem Statement

We've built 67 MCP tools, 1,121 tests, 162 CLI integration tests, fleet management,
hub-based RPC, file transfer, remote doctor, deploy pipelines, TOFU auth, and more.
But: does any of this actually serve the human operator's daily needs?

**Core question:** If you're a human running infrastructure across ring20 (.107, .109,
.121, .122), what do you reach for daily? What's working end-to-end? What's cargo cult?

## Spike 1: Current Fleet State (live evidence, 2026-04-17)

### Fleet doctor output
```
3 hubs configured:
  local-test     127.0.0.1:9100     OK     (97ms latency)
  ring20-dash    192.168.10.121:9100 ERROR  "Secret mismatch — hub was likely restarted"
  ring20-mgmt    192.168.10.122:9100 ERROR  "Cannot connect — is the hub running?"
```

**Reality:** 1 of 3 hubs working. The two remote hubs have been broken for 2+ days.
Fleet doctor DIAGNOSES this perfectly but cannot FIX it. Human must SSH in.

### Local session inventory
36 sessions on .107:
- 11x `upg2-*` sessions (framework upgrade workers, tagged `task=T-1238`)
- 10x `upgrade-*` sessions (another upgrade batch, tagged `task=T-1240`)
- Named sessions: `framework-agent`, `termlink-agent`, `perf-audit`, `route-audit`, etc.
- Sessions span projects: `/opt/termlink`, `/opt/999-Agentic-Engineering-Framework`, `/opt/050-email-archive`

**Reality:** Rich session data exists but there's no command that says "here's your
topology" — you get raw JSON. A human wants: "what's running, is it healthy, what needs attention?"

### Doctor output (local)
```
9/9 checks PASS:
  runtime_dir, sessions_dir, 36 sessions responding, hub running (PID 1750950),
  ufw allows 9100/tcp, no orphaned sockets, no dispatch manifest, no pending inbox,
  version 0.9.79 with 67 MCP tools
```

**Reality:** Local health is perfect. Remote health is broken. The operator needs BOTH.

## Spike 2: Operator Morning Check — What Would a Human Do?

**Step 1: "Is everything up?"**
- Today: `termlink fleet doctor` → works but output is JSON, not scannable
- Want: `termlink fleet status` → one-screen summary with colors and actions

**Step 2: "What's running where?"**
- Today: `termlink discover --json` → wall of JSON for 36 sessions
- Want: Grouped by project, showing task tags, session age, role

**Step 3: "Can all my nodes talk to each other?"**
- Today: Nothing. No VPN mesh test. No latency check between nodes.
- Want: `termlink net test` → mesh matrix with latency and status

**Step 4: "What needs my attention?"**
- Today: Multiple commands: fleet doctor + task review-queue + gaps
- Want: Single dashboard or `termlink fleet status` that rolls up all alerts

**Step 5: "Fix the broken thing"**
- Today: Fleet doctor says WHAT's wrong. Then manual SSH.
- Want: Actionable next-step printed with the diagnostic. Or self-heal.

## Spike 3: VPN / Network Testing Gap

### What exists
- `termlink remote ping <hub> <session>` — pings a specific session
- `termlink remote doctor <hub>` — health checks one hub
- `termlink fleet doctor` — checks all configured hubs

### What's missing
- **Mesh connectivity matrix** — can every node reach every other node?
- **Latency measurement** — how fast are the paths?
- **VPN-specific diagnostics** — is the tunnel up? What route is used?
- **Port reachability** — can hub port 9100 be reached from each node?

### What would be valuable
A `termlink net test` that:
1. For each hub: TCP connect + TLS handshake + auth + RPC roundtrip latency
2. Cross-hub: ask each reachable hub to ping every other hub
3. Report: matrix of connectivity with latency
4. Highlight: unreachable paths with diagnostic hint

This is achievable today — the hub RPC already supports forwarding requests.
We'd need a new RPC method `net.ping` that hub-to-hub connectivity testing uses.

## Spike 4: Information Architecture Audit

### Current Watchtower pages (framework-focused)
- `/` — Dashboard (tasks, metrics, quality)
- `/tasks` — Task list
- `/review/T-XXX` — Single task review
- `/approvals` — Tier 0 approvals
- `/metrics` — Project metrics
- `/sessions` — Session timeline
- `/discoveries` — Audit discoveries
- `/quality` — Code quality
- `/costs` — Token costs
- `/cron` — Cron audit
- `/config` — Framework config
- `/enforcement` — Policy enforcement

### Missing (operations-focused)
- **`/fleet`** — Fleet overview (hubs, sessions, health, connectivity)
- **`/fleet/<profile>`** — Hub detail (sessions, events, latency history)
- **`/session/<id>`** — Session detail (events, metadata, tags, health)
- **`/topology`** — Network map showing nodes and connections

### Clickability audit
- Task IDs in handovers → link to `/review/T-XXX` ✓ (Watchtower supports this)
- Hub profiles → NO clickable links
- Session names → NO clickable links
- IP addresses → NO clickable links
- Error diagnostics → NO "fix this" action buttons

## Assessment Summary

| Capability | Works E2E? | Daily Value | Operator Experience |
|-----------|------------|-------------|---------------------|
| Fleet doctor | YES | HIGH | Good diagnostic, but JSON-heavy, no fix actions |
| Remote doctor | YES | HIGH | Works per-hub, good detail |
| Hub restart | YES | HIGH | Zero-downtime proven, great |
| Fleet reauth | YES | MEDIUM | Prints fix steps, doesn't auto-fix |
| Session discovery | YES | MEDIUM | Raw JSON, no grouping/summary |
| File transfer | YES | MEDIUM | Reliable when hub up |
| Remote exec | YES | MEDIUM | Works, useful for automation |
| VPN/mesh test | NO | Would be HIGH | Nothing exists |
| Fleet status overview | NO | Would be HIGH | No single-screen summary |
| Operations dashboard | NO | Would be HIGH | Watchtower is framework-only |
| Binary deploy (remote) | NO | Would be HIGH | T-1016 unfinished |
| Topology view | NO | Would be MEDIUM | No visualization |
| 67 MCP tools | YES | HIGH (AI) | Agents use these heavily |
| 1,121 tests | YES | DEV only | Operator never sees these |
| 162 CLI integration tests | YES | DEV only | Operator never sees these |

## Key Insight

**We've been building DEVELOPER infrastructure (tests, MCP tools, framework governance)
while the OPERATOR experience is half-baked.** The foundation is solid — fleet doctor,
remote RPC, hub architecture, auth — but the last mile to "human sits down, sees
everything, acts on problems" is missing.

## Recommendations (prioritized by operator value)

### R1: `termlink fleet status` — the morning-check command [HIGH VALUE, MEDIUM EFFORT]
One command, one screen. Shows every hub, its status, session count, version, and
if broken: what to do. Human-readable, not JSON. Color-coded.

### R2: Watchtower `/fleet` page — operations dashboard [HIGH VALUE, MEDIUM EFFORT]
Real-time fleet overview in the browser. Clickable hubs drill to sessions.
Clickable sessions drill to detail. Action buttons for restart/reauth.

### R3: `termlink net test` — mesh connectivity [HIGH VALUE, HIGH EFFORT]
Needs new hub RPC method. But validates the network fabric that everything else
depends on. Would have caught the .121/.122 outage earlier.

### R4: Fix .121 and .122 NOW [IMMEDIATE, ENABLES EVERYTHING]
Nothing remote works with 2/3 hubs down. This is the current blocker.
- .122: SSH in, start hub → `ssh root@192.168.10.122 systemctl start termlink-hub`
- .121: SSH in, fetch secret → reauth with `--bootstrap-from`

### R5: Clickable references in Watchtower [MEDIUM VALUE, LOW EFFORT]
When any page shows a task ID, hub profile, or session name, make it a hyperlink.

## GO Decision

**GO** — The architecture supports all of this. The gaps are in the presentation layer
(fleet status command, Watchtower ops page) and one missing RPC method (net.ping).
The foundation (hub RPC, fleet doctor, auth, discovery) is solid.

**Priority order:** R4 (unblock fleet) → R1 (fleet status CLI) → R2 (ops dashboard) → R5 (clickability) → R3 (mesh test)

## Dialogue Log

### User directive (2026-04-17)
> Please evaluate if what we are building is valuable, focus on making work what's
> in here (e.g. VPN testing) and rethink what the user wants for operations and
> knowledge. How will the human user use this functionality and information?
> Also if there's a card or reference, make it clickable to the background of
> entity or artifact. Deep incept this, create action and execute.

**Interpretation:** Stop mechanical grinding. Evaluate real operator value. Build what
matters. Make information navigable and actionable. Execute, don't just plan.
