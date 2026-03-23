# T-233 Q1: The Case FOR Persistent Specialist Agents

**Position:** Specialist agents should be persistent (always-running TermLink sessions).

## Core Argument: Amortized Startup Cost

The single biggest cost in agent orchestration is **cold start**: spawning a process, loading context, establishing TermLink session registration, and warming up the agent's working knowledge. For a coding specialist, this means re-reading CLAUDE.md, scanning relevant source files, and rebuilding mental models of architecture. In our framework, `fw context init` + handover read + focus set takes 10-15 seconds and consumes ~5K tokens before any real work begins.

A persistent specialist eliminates this entirely. The second request to a coding agent costs zero startup — it already knows the codebase topology, the active task context, and recent decisions. Over a typical session with 5-10 delegations to the same specialist, this saves 50-150 seconds and 25-50K tokens of redundant context loading.

## State Accumulation: The Specialist Gets Smarter

Persistent agents accumulate **within-session learning** that ephemeral agents cannot:

- **File familiarity:** After touching `crates/termlink-session/src/rpc/handlers.rs` once, the specialist remembers the dispatch pattern, the `needs_write()` convention, and the test structure — no re-discovery on the next request.
- **Decision memory:** If the orchestrator says "use `dispatch_mut` for this handler," the persistent specialist remembers for all subsequent handler work in the session.
- **Error pattern recognition:** A persistent test specialist that has already debugged the `ENV_LOCK` flakiness won't waste time re-investigating it on the next test failure.

This is the difference between a consultant who visits weekly (re-reads notes each time) and one who sits in the office (absorbs ambient context continuously).

## Resource Cost: Manageable with TermLink

The primary objection to persistence is resource consumption. But TermLink sessions are lightweight:

- **Idle cost:** A registered TermLink session is a Unix socket file + a small metadata entry. No process runs when idle — the session is a registration, not a daemon.
- **Wake-on-demand:** `termlink agent ask` can wake a specialist by injecting a prompt into a Claude Code session. The LLM inference only runs when work arrives.
- **Natural lifecycle:** Sessions can have TTLs. A specialist unused for 30 minutes can auto-deregister via `termlink clean --ttl 30m`. Re-registration on next need is cheaper than full cold start because the session process may still be alive.

## Session Management Overhead

Persistent specialists require lifecycle management, but this is tractable:

1. **Registration:** One-time `termlink spawn --name code-specialist --role coder --persistent` at session start.
2. **Health monitoring:** `termlink ping code-specialist` — already built, ~1ms cost.
3. **Graceful shutdown:** `termlink signal code-specialist terminate` at session end.
4. **Crash recovery:** If a specialist dies, the hub detects disconnection via heartbeat. The orchestrator re-spawns with the last-known context from `fw bus manifest`.

The orchestrator already tracks active tasks and focus — tracking 2-3 persistent specialists is marginal overhead.

## When Idle: Not Wasted

An idle persistent specialist is not wasted — it's **available**. The alternative (on-demand spawn) means the orchestrator must wait for startup before getting results. In time-critical flows (debugging a failing test, iterating on a UI), the 10-15 second spawn latency compounds across multiple delegations.

## Recommended Model

**Hybrid persistent:** Start specialists on first use (lazy initialization), keep them alive for the session duration, auto-clean on session end. This captures 90% of the persistence benefit without the cost of pre-spawning specialists that may never be needed.

## Summary

| Factor | Persistent | On-Demand |
|--------|-----------|-----------|
| First-request latency | Same | Same |
| Subsequent latency | Near-zero | Full startup each time |
| State accumulation | Yes | No |
| Idle resource cost | Minimal (socket + metadata) | Zero |
| Management overhead | Low (3-4 lifecycle commands) | None |
| Session-end cleanup | Required | Automatic |

**Verdict:** Persistent specialists are the stronger default for sessions with repeated delegation to the same domain. The startup amortization and state accumulation benefits outweigh the marginal lifecycle management cost.
