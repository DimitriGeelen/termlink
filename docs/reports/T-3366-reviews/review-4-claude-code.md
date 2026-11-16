**T-3366: FAIL — the "Accepted" reason for row 2a is false for two of the tests it covers, and the tests behind three round-3 "Fixed" rows are again shown by name only; I found no blocking code defect.**

I could not run anything; this is from reading the bundle only.

1. **BLOCKING (evidence) — disposition 2a, section 10.** The reason says each test whose body is withheld "is named by a mutant that the runner shows killed". That does not hold for two of them:
   - `e36_key_without_id_and_no_journal_is_refused` is named by no mutant in section 7.
   - `e166_concurrent_first_starts_converge_on_one_id` is named by no mutant either (m21 names the sidecar test `e37_concurrent_first_contacts_*`). The E-166 convergence claim therefore rests on a test name and the "26 passed" count.
   - The same gap now covers the new round-3 tests: `e37_stale_marker_beside_an_id_without_key_is_refused_not_rekeyed` (m24), `e166_failure_at_the_key_write_is_finished_by_the_next_start` and `e36_id_mode_is_0644_under_a_strict_umask` (m26) have no body shown.
   - The fixes for rows 1, 5a and 5c are visible in the `finish_pending` and `atomic_write` code, so this is an evidence failure, not a code one.

2. **MINOR — sections 7 and 8, acceptance criterion 5.** The criterion says the mutants were "run on the deployed commit", but the report says 2b3759e02. Section 8 is a prose summary, not git output, and does not show that 2b3759e02 lies inside 995ac840a..fe72c850c. It also says "(none)" for changed T-3366 files while stating that T-3811 touched `runme.sh`, which carries `adopt_hub_id`. The runme fixtures were re-run on fe72c850c (392 passed), so the gap is limited to the Rust mutants.

3. **MINOR — `hub_identity.rs` `read_key`, FCW-82 "malformed → refuse".**
   - `u8::from_str_radix` accepts a leading `+`, so a 64-character file such as `+a+b…` is accepted as a seed instead of refused.
   - A 64-byte file containing a multi-byte UTF-8 character makes `&s[2*i..2*i+2]` slice off a character boundary and panic. The hub still does not start, but it crashes without the stated refusal (OBS-35).
   - `trim_end_matches('\n')` also accepts any number of trailing newlines.

4. **MINOR — `hub_identity.rs` `finish_pending`, residual of r3 #1.** The new refusal covers only id present with key absent. With a stale `.hub-identity.pending` beside prior state and both `hub.id` and `hub.signing.key` lost, `load_or_mint` generates a new key and rewrites the id, logged as "an interrupted mint was finished". That is a silent re-key outside AL-18's "first start only", though it needs three faults and the id is preserved. Separately, a failed `hub.id` publish now leaves the marker and the key on disk. The test asserts this, but the evidence map still says "leaves no file or temp" (FCW-81 wording).

5. **MINOR — sections 6 and 20.** Row 3b is "Stated" rather than shown: only the `claude-termlink` sidecar's pin and admin row appear, not `claude-termlink-alt`. The evidence map the criteria cite (`T-3366-evidence.md`) is still the round-3 text: 0043aff40, "23 of 23", 23 unit and 8 sidecar tests, no m24–m26.

Everything else is borne out by what is shown:
- **Rows 1, 3a, 4, 5a, 5b, 5c:** the code and the quoted tests match the dispositions, and the live admin row is present.
- **Rows 2b and 2c:** the help text is quoted and the residual's reason holds.
- **Criteria 1–4:** the pre-deploy ids in section 18 match those served after both deploys (sections 19 and 20), and all four hubs serve `hub_id` and `signing_key`.
- **Test result lines:** 26, 3 and 9 passed on fe72c850c.
