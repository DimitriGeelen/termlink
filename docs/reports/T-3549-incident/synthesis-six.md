# T-3549 — synthesis of six reviews (operator: "accept six reviewers")

Sources: TermLink `docs/reports/T-3549-incident/{codex,antigravity,claude}.md` and AEF
`/opt/999-Agentic-Engineering-Framework/docs/reports/T-4635-review/results/{claude-code,codex,opencode}.md` (AEF commit
6b9b76c79). Earlier local synthesis: `synthesis-local.md`.

## Unanimous
1. Defect class: project identity taken from ambient, inherited environment, crossing a shared process (the tmux
   server). A name list cannot hold the invariant. T-3445 even removed the detector (it stripped the T-2289 sentinel
   `_FW_PATHS_DERIVED_BY` and left `TASKS_DIR`/`CONTEXT_DIR`).
2. Build the child environment by construction at EVERY spawn path: `env_clear()`/`env -i` plus an exact allow-list
   of host settings. The tmux server's global environment holds no project identity, credentials, routing state or
   override flags. Project values go only to the one session (`-e`), from the target's validated launch spec.
3. Detect: block mismatches synchronously at spawn/registration, and audit server globals and live processes
   periodically.
4. 14:55 cleared only new sessions. Exposure lasts until each carrier is relaunched, and starts on 2026-10-16 01:40.

## Strong majority
5. Per-project tmux sockets with a single controlled launcher for create and recreate (codex ×2, claude, AEF
   claude-code/opencode). Antigravity alone prefers one dedicated socket per hub. Real isolation needs separate
   non-root Unix users (everything runs as root today).
6. Supervise lifetimes: tmux servers and the hub as supervised units, not children of a login session. Today's kill
   was an ordinary root-slice teardown on an ssh logout (AEF correction: 13:38:43.889).
7. Framework side (AEF): refuse, don't warn, on conflicting identity; a launcher version floor (no dispatch into
   pre-1.8.13 vendored projects); an inherited `FW_TASKS_DIR_OVERRIDE=1` must not grant an exception.

## New scope items for TermLink
8. The hub's own environment: a hub started from a project shell seeds every spawn. The allow-list applies there too.
9. Every TermLink path: `tmux_new_session` (execution.rs ~700, dispatch.rs ~962), the background backend
   (`detached_command` ~747-760), `scripts/tl-claude.sh` (`env -u PROJECT_ROOT` only, ~171/249).
10. Socket-aware attach and list for cockpits; migrate by controlled replacement, never by killing the shared server.
