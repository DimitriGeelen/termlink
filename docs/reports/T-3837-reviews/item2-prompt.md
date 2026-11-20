You are an independent design reviewer for a distributed job-dispatch contract. You have no tools and no other
context; judge only from the bundle.

The bundle holds a DRAFT clause ("item 2: optional dispatch closure") for a contract three independent backends must
implement identically, plus excerpts of one backend (allocation and the fenced state transitions). The owners are
deciding whether to adopt it. Your job is not a verdict: it is to make the clause STRONGER before adoption, judged on
three properties:

- Reliability: no lost or duplicated work, no silent divergence between backends, every refusal explicit and
  attributable, races decided deterministically.
- Flexibility: backends that close and backends that never close can coexist and interoperate; the clause does not
  force features on a backend that does not need them; it can evolve without a version break.
- Antifragility: failures and disagreements make the system better (visible, recorded, learnable), never quietly
  worse; a wrong or premature closure can be detected and recovered from.

Report:
1. The 3-6 most important weaknesses of the clause as drafted, each as: <weakness> — <a concrete sequence or
   scenario that shows it> — <severity: high/medium/low>.
   Consider at least: races (closure vs a concurrent T1 retry, closure vs a late lost->done), who may close and how that
   is authenticated, whether closure can be undone or must be terminal, what a reader sees on a closed dispatch whose
   last attempt is paused/unknown, what happens when two backends disagree on closedness for the same dispatch id,
   idempotency of repeated closes, what the MAY does to cross-backend conformance, and evidence/audit.
2. For each weakness, a concrete amendment to the clause (wording-level, short), and the conformance vector(s) that
   would pin it (one line each: arrange / act / assert).
3. One paragraph: should the owners adopt the clause as drafted, adopt it with your amendments, or not adopt it — and
   why. Steelman the opposite position in two sentences.
Be concrete and brief. No credit for intent.
