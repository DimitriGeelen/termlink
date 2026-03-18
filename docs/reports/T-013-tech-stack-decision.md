# T-013: Tech Stack Decision — Implementation Language

## Decision: GO — Rust

**Date:** 2026-03-08

## Context

TermLink needed a language choice for implementation. Protocol (T-005), identity (T-006), and composition (T-004) were already designed.

## Requirements

- Unix domain sockets (control + data plane)
- JSON-RPC 2.0 serialization
- Binary frame parsing (22-byte header, big-endian, zero-copy desirable)
- Filesystem operations (inotify/FSEvents)
- Cross-platform (macOS + Linux)
- Sub-millisecond data plane latency
- Single binary distribution

## Candidates Evaluated

| Language | Fit |
|----------|-----|
| **Rust** | Best: zero-copy, no GC, static binary, type-safe protocol, tokio async |
| Go | Good: fast dev, goroutines, but no zero-copy without unsafe |
| TypeScript | Good DX: best MCP ecosystem, but runtime overhead for binary parsing |
| Python | Fast prototyping, but GIL, slow binary parsing, heavy distribution |

## Decision Rationale

Rust chosen for:
1. **Zero-copy binary framing** — 22-byte header parsing without allocation
2. **No GC pauses** — critical for data plane latency
3. **Single static binary** — `brew install termlink` deployment
4. **Type-safe protocol code** — compile-time correctness for JSON-RPC + binary frames
5. **Excellent async** — tokio runtime maps well to session concurrency
6. **Memory safety** — no UB in session/socket handling

Aligns with constitutional directives: D1 (antifragility via memory safety), D2 (reliability via type system), D3 (usability via single binary), D4 (portability via cross-platform compilation).
