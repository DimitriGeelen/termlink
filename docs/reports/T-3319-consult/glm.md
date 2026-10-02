**1. Is it worth doing?**

Yes, but far narrower than the framing suggests. The one demonstrated failure — a bug report unread for three months — is an attention/routing failure, not a knowledge-extraction failure, and mining the corpus is orthogonal to it. Look at what the corpus actually is: roughly half of it (≈2,200 of 4,367 records) is heartbeat traffic; `channel:learnings` carries fixture pollution; and attribution is broken (see §6). Of the wishlist items, "learnings that should graduate" duplicates AEF's existing harvest, and "recurring failure classes" is new only for material that never became learnings — mainly `framework:pickup`. So: do unread/stalled detection, export pickup before trimming, and stop there for now. Strongest argument against: 299 substantive records can be read by a human in an afternoon. Automating interpretation of a tiny, noisy, badly-attributed corpus on one non-federated hub (every "fleet-wide" claim from one hub is a biased sample) builds a third brain whose value over AEF's existing audits is unproven — and the social-tools episode shows exactly how fast this drifts off-charter.

**2. Options**

- **A. Hub-side operational views.** Extend the activity table: oldest-unread, unanswered-thread age, writes vs. distinct readers. Produces live, regenerable coordination state — charter-legitimate. Cost: small, hub-core. Fails when metrics get gamed, "thread" depends on inconsistently-filled metadata, or it creeps toward the removed analytics.
- **B. AEF-side batch miner over exports.** A cron audit reads exported pickup/learnings, clusters duplicate reports, proposes findings via the existing audits→findings→tasks and harvest pipelines. Produces AEF artefacts. Cost: trivial compute at this size; real cost is false-positive review. Fails on attribution garbage-in, on becoming a second, worse memory outside AEF's consolidation, and on export staleness.
- **C. Capture-then-learn.** One-shot export of irreproducible record classes at trim time; the learner's store owns them thereafter. Cost: low if genuinely one-shot. Failure mode: archive-by-the-back-door if it becomes rolling and unbounded.
- **D. Fix producers, mine nothing.** Per-agent fingerprints, mandatory validated `msg_type`, test traffic off the production hub, heartbeats out of append logs. This dissolves most of the wishlist: unread becomes queue age (an AEF audit), duplicates become task dedup. Fails only if producer fixes are voluntary and unevenly adopted — which is why A's detection is still needed.

Recommendation: D now, A thin, C for pickup only; B only if §5 says yes.

**3. Where derived knowledge lives**

Not on the hub. Derived knowledge must outlive the retention that produced it; storing it there means trimming your own conclusions and turns the bus into a system of record — the explicit non-goal. The hub keeps only regenerable live state (current unread counts). Durable knowledge goes into AEF memory, because AEF already owns the lifecycle: fleet-relevant results ride harvest into framework practices; project-specific ones stay in project learnings; cross-cutting findings become audit findings. A separate store is the wrong answer — AEF already has consolidation and dedup; a third location would itself need deduplicating.

**4. What to capture before trimming**

Less than the question implies. Mail isn't at risk: inbox (486 records / 32 topics) and dm (509 / 24) sit far below the 1,000-per-topic cap. The 14-day rule threatens `framework:pickup` — most of its 299 records are already older than that, and this is communication that exists nowhere else (upstream's router work also depends on this class). Export pickup in full, once, now. `channel:learnings` is a mirror of project `learnings.yaml`; a deduped export is cheap insurance. Let heartbeats and fixtures burn — reproducible or worthless. Decision rule against accidental archives: export only what exists nowhere else and cannot be regenerated; one-way, one-shot, into the learner's store where AEF's own retention governs. Lift the compaction hold once the pickup export lands.

**5. First measurements**

(a) Hand-code all 299 pickup records into three buckets: duplicates (of each other or of existing learnings), unanswered >30 days, genuinely new failure classes seen in 2+ projects. If fewer than ~2 repeated cross-project classes exist, there is nothing to mine — just add an AEF audit for unanswered pickup. (b) From the new activity table, count non-heartbeat topics with zero distinct readers in 30 days. If unread-ness dominates, the problem is routing and attention (A + D), not learning.

**6. The unasked mistake**

The per-host sender fingerprint. Nearly every derived question on the wishlist — who reads what, reply latency, which projects report which failure class, cross-project duplicate detection — depends on attribution, and today almost every record claims to be the same sender. Fix that first (along with its sibling: tests posting placeholder learnings to the production hub 146 times); otherwise every "who" in any future analytics is fiction.
