## Verdict

**A is directionally right, but its proposed exemption is too broad without an explicit scope contract.** The distinction is between file inventory and diagram-step relationships—not source versus non-source. Commands already demonstrate that non-source files can belong in the fabric.

## Answers (1-4)

1. **A type-based exemption can be principled.** The test is whether the rule can be stated independently of these nine warnings and applied consistently to future cards: does this card assert that its location belongs in the watched file inventory, or merely identify a container for a separately governed entity?

   Exempting `workflow-step` is justified only if those cards have the latter meaning, that meaning is enforced, and another check owns their integrity. The existing step↔diagram guard helps, but does not necessarily detect missing workflow adoption or stale version selection. If the exemption merely makes current warnings disappear, it is silencing. Drift reports must also disclose their scope.

2. **Separate workflow selection from file coverage.** Keep the step cards, but introduce an explicit, non-vendored manifest identifying the eight adopted workflows and their selected versions. Validate that each selected diagram exists and its steps match the cards. Historical diagrams need not acquire cards merely because they share a directory.

   “Latest version only” is a legitimate category if defined mechanically and applied consistently, but it can silently adopt new upstream semantics. Explicit version selection makes upgrades reviewable. A stable non-vendored location helps only if it is an authoritative model or manifest; redirecting locations to a proxy would obscure where steps actually live.

3. **All 34 command cards are worthwhile only if their relationships are maintained.** Header-derived purposes improve onboarding, but alone provide little impact analysis. Dependencies on scripts, tools, and other commands are the valuable content.

   Under the current rule, adopting the natural command category means registering all 34. A singleton plus documentation could be honest under a formally supported selective-registry policy, but documentation alone does not satisfy the existing coverage contract. Do not broaden solely to clear the warning; commit to maintaining useful command relationships.

4. **At the next re-vendor:**
   - **A:** Removed diagrams or changed step IDs can invalidate cards. The separate guard may catch these; the exemption cannot ensure newly released workflows or versions receive review.
   - **B:** Removed historical files can leave stale cards; new versions create registration work. File coverage still cannot establish correct step semantics or version selection.
   - **C:** Diagram risks remain unresolved, while deleting `capture.md` loses a known dependency.
   - **D:** The same risks persist, and standing warnings weaken the check’s usefulness.

## Risks

A blanket type exemption can become an escape hatch. B creates maintenance obligations around replaceable historical artifacts. Any approach can mislead if “coverage” sounds comprehensive while measuring only a declared subset.

## Recommendation

Choose **modified A**: explicitly scope file coverage, govern adopted workflows through a version-selection manifest and dedicated validation, and register commands with meaningful dependencies.

**The most important reason:** coverage should enforce a declared maintenance obligation, rather than infer one from every card’s location field.