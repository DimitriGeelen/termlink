# T-3823 mutants (FP 3.1.b) — 6/6 killed, 2026-11-19

Runner: `crates/termlink-session/tests/mutants/run-t3823.py` (scratch TERMLINK_RUNTIME_DIR, T-3822). Item 4 (orphan claims) is held by T-3703 mutant c18 plus `t3823_an_orphan_claim_is_closed_at_restart_and_the_revocation_sweep_then_refuses`; item 1's deployment half by `tests/arc011-w1-provision-fixtures.sh` 2e/7c and the live forge (task file).

| mutant | what | verdict |
|---|---|---|
| d1-no-digest-in-stamp | the hub list cache ignores the file digest (same-length rewrite served stale) | killed |
| d2-fallback-cache-stale | the sidecar fallback cache ignores the stored list's stamp | killed |
| d3-no-baseline | a fresh sidecar raises an entry for every existing card | killed |
| d4-no-mode-entry | a change of the check setting raises nothing | killed |
| d5-outside-edit-weakens | an outside edit of the setting can weaken the check | killed |
| d6-own-key-capped | the card's own key can be pushed into other by the cap | killed |

6/6 killed
