## 1. Choice and strongest objection

**C, with weaker claims about what read tracking proves.** Durable metadata belongs outside the retained log. Ownership and purpose make creation accountable; activity observations help operators investigate neglect. None requires server-managed cursors.

The strongest objection is that C risks building a small governance system around 113 topics when only 12 currently have descriptions. Mandatory fields can produce fabricated accountability, while fetch timestamps produce fabricated confidence that messages are being consumed. Keep the implementation small and label observations precisely: **“last fetch,” not “last read.”**

I would stage delivery: persistent catalog and creation rules first, fetch telemetry second. They solve separate problems.

## 2. Required owner and purpose

Require them for **new, explicitly created topics**, but do not mistake nonempty strings for useful metadata.

An owner should resolve to an accountable operator, team, or service identity—not merely whichever disposable agent happened to create the topic. Purpose should explain what goes there and why someone would consume it. Template-generated purposes are appropriate for genuinely standardized topics.

Known creation patterns should be registered templates with an ownership rule, purpose template, and retention default. Matching `dm:*` alone does not establish ownership. Generic `--ensure-topic` must use the same creation validation when the topic is absent; otherwise it becomes the escape hatch.

For existing topics, backfill only what is defensibly inferable. Mark the rest **owner unknown / purpose unverified**, rather than manufacture plausible text. Review those first when they also appear inactive.

I would defer tags until there is a concrete filtering need. Owner, purpose, and predictable names already provide substantial discovery value.

## 3. Last-fetch tracking versus hub-owned cursors

**Last-fetch tracking is sound telemetry; it is not consumption accounting.** Keep client-owned cursors unless agents need coordinated work sharing, durable delivery acknowledgments, or centrally recoverable progress.

Record separately:

- Last successful fetch request, including empty polls.
- Last time a fetch returned records.

Neither proves processing. A reader can fetch and crash, repeatedly retrieve the same records, or discard everything. If processing assurance becomes necessary, add explicit acknowledgments for those workflows.

Signed identities provide attribution, but shared identities still conceal individual readers. Define whether the identity represents an agent instance or a durable service.

Server-owned cursors enable lag reporting and restart recovery, but introduce reset semantics, group membership, and coordination machinery. They still do not prove that external work completed correctly. That is too much machinery merely to answer “does anyone appear interested?”

Bound telemetry storage and coalesce timestamp updates; do not turn every poll into a durable metadata write.

## 4. Dead-topic rule

Call it an **inactivity candidate**, not a dead topic.

Start with **N = 30 days**, configurable by topic class. That is a review threshold, not a correctness boundary; monthly or incident-only topics need longer thresholds or an explicit dormant designation. Retention duration should not determine topic lifetime.

**Flag, never automatically delete under this rule.** Deletion needs owner or operator judgment, including whether callers expect the name to remain available.

Count accepted appends as write activity and successful fetches as reader interest, but show them separately. Exclude sweeper actions, catalog inspection, and administrative metadata updates.

Crucially, the health-probe example defeats the proposed rule: a producer posting forever ensures “no read and no write” never becomes true. Add a separate flag for **ongoing writes with no observed fetches for N days**. Start its clock when observation begins; missing historical telemetry is not evidence of historical inactivity.

## 5. Wildcards and hierarchy

**Naming conventions now; wildcard subscriptions later.** Document a small set of topic families and support prefix filtering. Tens of agents do not themselves justify subscription routing complexity.

Add wildcards when a concrete workflow repeatedly maintains changing lists of topics. First define authorization and whether subscriptions include future matching topics. “Relevant to me” also needs trustworthy project or capability metadata; it is not automatically delivered by hierarchical names.

## 6. What the framing misses

**How readers discover that retention overtook their cursor.** An authoritative retained log must expose its earliest available offset and return an explicit gap when a requested position has expired.

Otherwise a topic can have excellent metadata and visible readers while agents silently miss coordination messages. That correctness issue matters more than richer discovery.