# T-1491 AC1 — by-project table operator-readable

Reviewer: judge-t-1491-r3-38facb214fd3, revision 54e0b9bfb3ef2dc59e9c85d4bfbd96f65588a80c (HEAD matched).

Ran from /opt/termlink: `target/release/termlink agent presence --by-project --window-secs 86400`
(binary built 2026-10-26 22:54)

```
PROJECT                     POSTS    PEERS TOP_PEER            LAST_SEEN
010-termlink                   24        1 d1993c2c3ec44c94    51m ago
root                            6        1 d1993c2c3ec44c94    2h ago

2 project(s) active in window=86400s
```

Findings:
- PROJECT is the leftmost column; columns are aligned, numeric right-aligned, humanised LAST_SEEN. Scannable.
- Footer reports project count (2) and window (86400s). Source: crates/termlink-cli/src/commands/agent.rs:3686.
- Empty case (`--window-secs 60`): clear message "no projects active in window=60s — fleet has no tagged posts".
- `--json` gives structured equivalent (view, window_secs, projects[]).

Minor, non-blocking suggestions: footer could render window humanised (e.g. "24h"); "root" as a project name may be a cwd-derived artefact worth a glance.

Verdict: green.
