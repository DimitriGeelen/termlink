## Verdict

This is **ambient project authority escaping into a shared, persistent process**. Adding two variables fixes today’s omission, not the defect class.

The source confirms the five-name deny-list, session forwarding, and default-server selection (`crates/termlink-cli/src/commands/tmux_spawn.rs:19–65`). Both callers use it (`execution.rs:700`, `dispatch.rs:962`, same directory). The incident timeline and damage claims remain supplied evidence, not independently verified.

**14:55 is not necessarily the exposure cutoff.** Clearing global variables does not repair existing processes, their descendants, or explicit session overrides.

## Root cause (ranked)

1. **Shared mutable environment across project boundaries.** Whichever project starts the server becomes an unintended configuration authority for subsequent projects.
2. **Incomplete source-side sanitisation.** A deny-list cannot establish the claimed invariant that the server “never carries a project.” Unknown variables and project-specific values in ordinary variables defeat it.
3. **Consumers trusting inherited paths as authority.** This converts contamination into foreign reads and writes. A validated project identity must control storage selection.
4. **Environment-based location itself.** Environment variables are usable transport for validated configuration; treating them as authoritative identity is the problem.

Same-UID execution magnifies all four: socket separation prevents accidents but does not enforce access isolation.

## Remediation (prioritised, concrete)

1. **Contain and preserve evidence.** Pause affected writers, capture protected process/session/environment inventories and storage snapshots, then restart contaminated agents from trusted launch specifications. Patch `TASKS_DIR`/`CONTEXT_DIR` immediately, but classify this as temporary containment. Inventory deployed binaries and vendored framework versions.

2. **Replace subtraction with clean construction.** Use Rust `Command::env_clear()`—equivalent in intent to `env -i`—then supply a small, documented set of trusted values: service account identity/home, fixed system `PATH`, approved locale, fixed shell, and necessary runtime location. Resolve the tmux executable explicitly. Do not copy arbitrary caller values merely because their names are allowed. Server globals should contain no project paths, agent identities, credentials, sidecar endpoints, or project toolchain paths.

   Construct session configuration separately from the **target’s validated launch specification**, not blindly from the spawning agent. Pass project settings per session; deliver secrets through scoped mechanisms rather than visible command arguments.

3. **Use a socket per project.** Derive its identity from registered project identity, including worktree distinctions where relevant. A per-hub socket only provides equivalent containment if the hub serves one project. A per-user socket changes nothing here.

   Deliberately bootstrap each server through a managed launcher with a clean environment, controlled configuration, and explicit keepalive/lifecycle policy. Every allowed first-client path must use the same bootstrap contract after crashes. Prestarting alone leaves a restart race; manual clients remain a bypass under the same UID.

   Use `-f /dev/null` or a reviewed minimal configuration at startup; it does not sanitise environment or reconfigure an existing server. Restrict `update-environment`, and use `-E` where appropriate: attachment can otherwise import client values into a session. `set-environment -g -u` is cleanup, not prevention or process repair. These distinctions follow the [tmux manual](https://man.openbsd.org/tmux#GLOBAL_AND_SESSION_ENVIRONMENT).

   Provide project-aware attach/list wrappers; make cockpits enumerate registered sockets. Drain and recreate existing sessions. Separate servers still die together if their shared systemd slice is killed.

4. **Enforce destination validation.** Before any context read or mutation, compare canonical storage paths against independently established project identity. Refuse mismatches by default; allow external stores only through explicit validated configuration. Warning plus rederivation is safe only when the reference root is independently trustworthy. Upgrade or quarantine old vendored copies; a launcher guard is interim protection, not coverage for every direct invocation.

5. **Test the invariant.** Existing live coverage checks only `PROJECT_ROOT` (`tmux_spawn.rs:147–189`). Add unknown-variable canaries, polluted existing servers, absent caller variables, cross-project dispatch, attachment, startup configuration, concurrent bootstrap, and crash/restart tests.

## Detection

Gate launch and worker readiness **before project access**, comparing trusted registration with resolved root, framework, tasks, context, and socket identity. CWD is supporting evidence, not authority: processes legitimately change directories.

Check server globals against the baseline at bootstrap and before dispatch. Independently inspect sessions and obtain effective configuration attestations from workers; server inspection alone misses surviving contamination and shell startup changes.

Run an external fleet sweep at least every minute, alert immediately, and block affected launches. Record variable names and redacted mismatch metadata, not secret values.

## Damage assessment

Extend investigation until every contaminated process and descendant terminated or was repaired. Examine all projects, not just 1409.

Preserve tracked, untracked, ignored, and deleted-file evidence. Compare snapshots, Git history/status, mtimes/ctimes, task sequences, episodic records, embedded repository references, transcripts, dispatch records, and pre-existing audit logs. Correlate writes with process/session lifetimes and target identities.

Timestamps and unusual IDs identify candidates, not authors. Without prior auditing or snapshots, some attribution and overwritten/deleted content may be unrecoverable. Report confirmed, probable, and unresolved damage separately; include foreign reads and downstream actions.

## What we missed

Shared shell startup files, tmux hooks, executable search paths, SSH agents, credential helpers, hub runtime directories, `/tmp` sockets, caches, and crontabs repeat this pattern. Durable protection against compromised agents requires separate Unix identities or enforced sandbox boundaries with restricted filesystem and credential access.