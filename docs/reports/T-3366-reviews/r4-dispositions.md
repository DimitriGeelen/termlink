# T-3366 confirmation round (operator ruling A) — verdicts and dispositions

Both seats ran, independent, on bundle 4 (deployed commit fe72c850c):

- **opencode: PASS** (`review-4-opencode.md`), five MINOR notes.
- **claude-code: FAIL** (`review-4-claude-code.md`): one BLOCKING point, on evidence (not code); "I found no blocking
  code defect"; it confirmed rows 1, 3a, 4, 5a, 5b, 5c, 2b, 2c and criteria 1–4.

Per ruling A, a FAIL from either seat goes to the operator with these dispositions, not to another round.

| # | Seat | Finding | Disposition |
|---|---|---|---|
| cc-1 | claude-code | BLOCKING (evidence): disposition 2a said every withheld test body is named by a killed mutant; false for `e36_key_without_id_and_no_journal_is_refused` and `e166_concurrent_first_starts_converge_on_one_id`; bodies of three round-3 tests also withheld | **Agreed, the reason was wrong as written.** The two tests have no mutant; their claim rests on the green result line (26/26, now 27/27) and the code shown. The withheld bodies are in the repository at the reviewed commit; the 120,000-byte argv limit of the isolated-run is why they were cut. Corrected wording, not a code change. |
| cc-2 / oc-2 | both | mutants ran on 2b3759e02, not on the deployed fe72c850c; section 8 is prose | **Accepted with evidence on disk:** `git diff --stat 2b3759e02 fe72c850c -- crates/termlink-hub/src crates/termlink-sidecar/src crates/termlink-sidecar/tests crates/termlink-cli/tests/t3366_hub_identity.rs` is empty; the runme fixtures (which carry `adopt_hub_id`) were re-run on fe72c850c, 392/0. |
| cc-3 | claude-code | `read_key` accepts a '+'-prefixed seed, panics on a non-ASCII key file, accepts several trailing newlines | **Fixed after the round** (commit following this file): 64 ASCII hex digits and at most one newline, checked before parsing; test `e37_fcw82_malformed_key_variants_are_refused_not_panicking` (27/27). |
| cc-4 | claude-code | a stale marker beside prior state with BOTH id and key lost re-keys (three faults); evidence map wording for FCW-81 | **Accepted as a residual, stated:** three faults, the id is preserved (the marker's), and a new key is caught by every sidecar's pin (FCW-83, needs-attention naming both keys). Evidence map wording corrected. |
| cc-5 / oc-4 | both | only one W1 sidecar's pin shown; evidence map stale (0043aff40, 23 mutants) | **Fixed on disk:** the evidence map is updated to the final counts; the alt sidecar's pin and admin row match (`claude-termlink-alt` state files, same fingerprint `ed25519:ef7bed96…`, admin row digest `871cf9e7…`). |
| oc-1 | opencode | a planted `.hub-identity.pending` in a legacy dir mints past FCW-79 without adopt-id | **Accepted, reason:** it needs root write to the runtime dir, the same trust as editing `hub.id` (AL-17 edge case 2); peers refuse a changed id (FX-2). |
| oc-3 | opencode | some test bodies credited by name only | Same as cc-1. |
| oc-5 | opencode | zombie writer's pid counts as alive in the temp sweep | **Accepted:** temps are never read; the sweep only ever removes dead writers' temps; a lingering temp is inert. |
