# T-3654 rollout handback

1. Action **4d** in `runme.sh` (after 4c, before 5): sets `TERMLINK_MAIN_PROJECT=010-termlink` in `/etc/termlink-sidecar/<instance>.env` (the unit's EnvironmentFile; `daemon.rs` reads config `main_project`, then this env), backup `<env>.bak-<UTC>` first, restarts via the shared `w1_restart_unit`, and verifies `sidecar status` reports `main.lease.state` held|vacant. Skips when env and live status are already right. FAILED when the state is still not held or vacant after the restart, or when the project is set but the state is unknown for another reason (that case is not restarted).
2. `scripts/arc011-w1-provision.sh` (action 17) now writes the same last line, so 17 never reverts 4d (provision fixtures 21/21).
3. Dry-run on .107: `[DRY] would set TERMLINK_MAIN_PROJECT=010-termlink in /etc/termlink-sidecar/010-termlink--claude-termlink{,-alt}.env and restart termlink-sidecar@010-termlink--claude-termlink{,-alt}.service`. rc 0.
4. Fixtures: 8 new hermetic 4d cases. `tests/runme-fixtures.sh` 258 passed, 0 failed.
5. Hooks in `.claude/settings.json` (matcher-less groups): SessionStart and UserPromptSubmit run `termlink main claim --session-from-stdin`, and SessionEnd runs `termlink main release --session-from-stdin`. They have a 5 s `timeout` (when available), a 10 s hook timeout, all output silenced, and always `exit 0`. Tested silent with rc 0 against the installed binary (no `main` verb yet), no termlink on PATH, and fakes that hang, print or fail.
6. `fw enforcement baseline` re-saved; `fw doctor` rc 0 (hook paths 43/43, baseline intact).
7. Note: until the operator's next runme run, action 4 installs the new binary and 4b restarts the sidecars onto it, so 4d restarts them once more. If the installed binary still lacks the lease, 4d reports FAILED (`state=absent`). That is correct.
8. Pre-existing, not from this work: `check-pipefail-grep-q` FIRES on `runme.sh` (3 sites at lines ~2631/2646/2654, action 26 from 2938bf6f8 / T-3657).
