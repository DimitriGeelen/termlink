# CLI UX Analysis: termlink-cli/src/main.rs

- **Surface area is too flat.** 28 top-level subcommands with no grouping. PTY commands (output, inject, attach, resize, stream) and event commands (events, emit, broadcast, collect, watch, wait, topics) should be nested subcommands (e.g., `termlink pty attach`, `termlink event watch`) to reduce cognitive load and improve discoverability.
- **Help text is good per-command** — doc comments are concise, args have descriptions, defaults are documented. The top-level `about` ("Cross-terminal session communication") is vague though; it doesn't hint at what commands are available or suggest a getting-started flow.
- **Naming is mostly consistent** but has overlaps: `watch` vs `collect` vs `events` all poll events with subtle differences that aren't obvious from names alone. `send` (raw JSON-RPC) vs `exec` (shell command) vs `request` (event request-reply) are three different "send something" verbs that require reading help to distinguish.
- **Error messages are adequate** — `anyhow::Context` wraps most operations with human-readable context (e.g., "Session 'X' not found", "Failed to connect to session"). Exit codes are properly forwarded from `exec`.
- **Target resolution is consistent** — all session-targeting commands accept "Session ID or display name", which is good. However, there's no shell completion support or `--format json` flag for scripting, limiting programmatic use.
- **`kv` is the only properly nested subcommand**, proving the pattern works. Applying it to PTY ops and event ops would cut the top-level list roughly in half.
- **Missing:** No `--quiet`/`--json` output flags, no shell completions (`clap` supports `clap_complete`), no `help` examples in doc comments. These are table-stakes for CLI discoverability.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-068 (CLI restructuring)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
