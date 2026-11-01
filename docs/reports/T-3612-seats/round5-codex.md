<!-- T-3612 final round (r5), seat codex, RECOVERED from result.jsonl (the worker's result.md held only its closing line). Run exit 1: framework integrity check T-3986, 'the codex reviewer modified the export it reviewed' (it wrote result.md inside the checkout), so its verdict does not count as a pass; findings kept as input. -->

# Final review — draft v0.8

Reviewed against local "'`/opt/termlink`, HEAD `b6bacc4400e84789779915444b8451475fd4cf2f`. “Draft” below means `docs/reports/T-3612-round4-draft.md`.

1. **BLOCKING — §6.3 cannot implement the ruled product set.**

   The live-development-file ruling and product D1=6 are faithfully transcribed (Dialogue Log 17–18, `docs/reports/T-3612-purpose-rederivation.md:160–164`), but their proposed implementation contradicts the code. Arc drivers supplement global drivers; global weights win collisions (`.agentic-framework/agents/termlink/bvp-estimator/estimator.py:2727–2736`). Thus product D1/D2/D3 remain 9/7/5, not 6/9/7, and development F-RECALL remains included. Scoped weights are capped at 6, excluding F-TRUST=7; only three entries are allowed (`.agentic-framework/lib/arc.sh:1569,1625`). Cross-arc rankings use global weights exclusively (`.agentic-framework/lib/bvp.sh:854–889`), as do task rankings (`:476–500`). Merely creating the product YAML does not connect it to these readers. This blocks the promised operational separation, not the operator'"'s chosen priorities.

   **Replace §6.3's implementation claim with:** “Keep the development set live and record the complete product set in "'`policy/value-drivers-product.yaml`. Existing scoped drivers cannot implement the product set'"'s protected weights, F-TRUST weight, or separate rankings. Until AEF supports layer selection, product decisions must include an explicitly labelled calculation against the product file; ordinary BVP rankings remain development rankings. Operational product scoring is complete only when estimation and ranking use exactly the selected set, including tasks without an arc.” Change §3's heading to name the product file.

2. **BLOCKING — adoption leaves contradictory authoritative non-goals.**

   §6.1 changes the purpose and non-goals, but leaves "'`docs/CHARTER.md:25–26` declaring that spokes *never* communicate directly. §6.2 leaves `.context/project/objectives.yaml:106` excluding a “system of record,” contradicting the bounded authoritative record adopted in draft lines 162–164. The charter also calls the older design authoritative (`docs/CHARTER.md:58–62`), whose invariant still prohibits peer surfaces (`docs/architecture/parallel-execution-substrate.md:377`). These are conflicting instructions after adoption, not merely missing future features.

   **Add to §6:** “Update the charter'"'s load-bearing nouns, objectives "'`out_of_scope`, and referenced topology invariants alongside the non-goals. Use: ‘Hubs own their topic records and never relay topic messages between hubs. Signed directory-card exchange and hub-brokered direct circuits with hub fallback are permitted; direct circuits are not yet implemented.’ Use: ‘The hub is authoritative for coordination within its retention window; it is not an archival database.’ Label README'"'s strict-star description as current implementation.”

   Also replace §6.5 with: “Before directory-card exchange or direct circuits land, update tripwire assertions and allowlists to permit those specific paths while rejecting inter-hub topic relay and unbrokered mesh.” Merely rewording comments would not change the restrictions ("'`crates/termlink-hub/tests/no_federation_tripwire.rs:29–39`; `scripts/check-strict-star.sh:50–65`).

3. **NON-BLOCKING — the purpose is one extractable sentence, but a target rather than universal present coverage.**

   Draft lines 22–24 reproduce ruling E exactly (`purpose-rederivation.md:149–152`). The colon does not create a second sentence. The current extractor cannot recognize it: its anchor and terminus are different (`scripts/check-charter-sentence-drift.sh:78,96–107`). §6.1 correctly requires their coordinated replacement, so this is already addressed in principle.

   **Make the instruction precise:** “Set the anchor to `TermLink makes sure every message reaches the recipient it is meant for:` and the terminus to `stuck`. Update all three document copies, the test'"'s wrapped charter fixture, drift substitutions, and repeated-terminus fixture together.” The latter changes matter because tests substitute phrases absent from the new sentence ("'`scripts/test-charter-sentence-drift.sh:33–45,64–80`).

   Preserve E, but append: “This states the product'"'s intended contract; current coverage and remaining gaps are listed below.” Present code supports evidenced handover and uncertainty ("'`crates/termlink-sidecar/src/deliver.rs:16–25`), while addressing remains a static intra-project book (`book.rs:5–9,73–94`). Draft lines 69–81 already acknowledge incomplete stages and circuits; publishing E alone must not imply those gaps are solved.

4. **NON-BLOCKING — recall retirement is consistent, but guard success alone is insufficient evidence.**

   The v0.8 condition follows ruling 19 and Q3 without requiring new archival scope (`purpose-rederivation.md:79–85,165–168`; draft lines 137–149). However, the guard explicitly does not judge prover adequacy, permits allowlisted arcs, and skips exit 2 under CI (`scripts/check-arc-claim-drift.sh:40–46,159–162,210–216`).

   **Add:** “Retirement evidence must identify the bound prover and a successful execution covering all three behaviours; an allowlisted, not-run or CI-skipped result is not that evidence.” Identify the arc by file and anchor task: “`.context/arcs/arc-012.yaml`, anchored by T-3304.” Its internal `id` is currently `arc-011` (`:1–11`), so an ID-only retirement lookup is ambiguous.

5. **NON-BLOCKING — aliases are sufficient; scoring history needs an explicit boundary.**

   §6.2 supplies the complete O→P/DP mapping and preserves T-3371'"'s checks, including the required "'`out_of_scope` list (T-3371, Verification). No current arc YAML has `supports:` references. Retain these aliases as historical provenance rather than rewriting old task evidence.

   F-CONV-RECALL'"'s new ID preserves F-RECALL's meaning, but §6.6 alone does not preserve comparable totals: BVP recomputes against current weights and drops absent drivers ("'`.agentic-framework/lib/bvp.sh:345–363`). **Add:** “Preserve historical scores; label new calculations with driver-set file and revision. Mark pre-adoption totals as belonging to the old set and recalculate current decision comparisons consistently.”

The cap and recall prover do not conflict: fixture suites are explicitly excluded from the cap (draft lines 195–200). D1=6 and the cap faithfully reflect rulings 16 and 18; no additional contradiction among those numerical rulings was found.

**Verdict: ADOPT WITH THE LISTED CHANGES**, resolving findings 1–2 before adoption. The purpose and priorities are coherent; the proposed scoring mechanism and remaining authoritative text are not.

**Confidence: high on the blockers, medium overall.** Read the draft, rulings, prior review files, named adoption files, arcs, and targeted code/tasks. GitHub raw access failed; the requested local checkout was available. Did not exhaustively read all crates/tasks or verify live fleet counts/state. No builds, tests, repository edits, or external messages were performed; only this result file was written.
EOF
wc -w result.md'
