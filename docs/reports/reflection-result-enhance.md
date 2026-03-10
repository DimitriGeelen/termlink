# TermLink: Top 5 Enhancement Opportunities

1. **Authentication & Authorization** — No auth on Unix sockets. Any local process can send/exec/inject into any session. Add per-session tokens or capability-based access control before production use. Without this, `exec` and `inject` are local privilege escalation vectors.

2. **Session Supervision & Auto-Recovery** — Sessions can die silently; `clean` is manual. Add a supervisor mode to the hub that detects dead sessions (heartbeat), auto-deregisters them, and optionally restarts spawned processes. Critical for unattended multi-agent orchestration.

3. **Structured Event Schema & Ordering Guarantees** — Events are free-form JSON with poll-based delivery. Add typed event schemas (protobuf/JSON Schema), sequence guarantees across hub fan-out, and at-least-once delivery with ack. Current polling (`Watch`/`Collect`) will miss events under load.

4. **Hub as Persistent Router (not CLI-embedded)** — `Hub` is a subcommand in the CLI binary with no persistence, no clustering, no state recovery. Extract it into a proper daemon with pidfile, graceful shutdown, config file, and systemd/launchd integration. The hub is the single point of failure for multi-session coordination.

5. **Error Propagation & Observability** — `main.rs` is ~500+ lines of flat command handlers using `anyhow` with minimal structured error codes. Add exit codes per failure class, machine-readable JSON output mode (`--json`), and OpenTelemetry tracing spans so orchestrators can programmatically react to failures instead of parsing stderr.

**Biggest pitfall to avoid:** Treating TermLink as "just a CLI tool" — the moment agents depend on it for coordination, it becomes infrastructure. Prioritize the hub's reliability and auth before adding more CLI features.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-066 (hub as daemon), T-008 (security model)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
