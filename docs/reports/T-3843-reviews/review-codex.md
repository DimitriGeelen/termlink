**1. Diagnosis: strong evidence of a contributor, insufficient evidence of the only reader.**

The unsigned canary can produce exactly this registry entry, and its scheduled time matches the latest reads. But the bundle overstates certainty:

- **[Medium] Unexplained first observation:** `2026-11-20T05:52:37Z` does not match the cron schedule. A manual canary run could explain it; another local scanner, legacy unsigned sidecar, or recurring diagnostic could also explain it. Nothing shown rules those out.
- **[High] Identity aggregation:** all unsigned local reads share `(unnamed)/local/claimed`. Several processes can merge into this single row; its latest timestamp cannot attribute every read to the canary.
- **[Medium] Binding qualification:** the supplied hub code refuses unsigned reads of a bound inbox *before recording them*. If these inboxes were bound at the observed times, the proposed explanation conflicts with that code. Binding history and the running binary must agree with the diagnosis.
- **[Low] Scope and timing overstatements:** the script reads only local topics with nonzero reported counts. Daily executions exactly 24 hours apart sustain the inclusive window, but execution jitter can create short gaps; “can never pass” is too absolute.

**One settling check:** capture every attempted inbox read over a complete daily cycle, including local process identity, method, topic, proof presence, binding state, and acceptance/refusal. Correlate these events with cron execution and registry updates. This distinguishes the canary from other processes sharing its row; historical attribution still requires historical logs.

**2. Options A–D**

| Option | Strongest concrete weakness | Standing-rule compliance |
|---|---|---|
| **A: skip every inbox** | **[High]** An agent receives an `artifact-chunk` directly in its inbox; the canary never inspects it, and no replacement detector is specified. | **No as proposed.** Coverage is withdrawn without a field-proven replacement and documented approval. Could allow action 37 after old rows expire, assuming no other readers. |
| **B: skip proven-single-reader inboxes** | **[High]** Today each inbox has two consumers. A literal “exactly one, proven” predicate therefore reads both again, renews the unnamed row, and prevents its own eligibility condition from ever becoming true. If “any proven reader exists” is intended instead, it removes detection coverage without establishing ownership or replacement coverage. | **No as proposed.** Signing proves identity, not file-shape detection. Automatic future exclusions also need an approved handoff policy and proven coverage. |
| **C: tolerate the unnamed canary** | **[High]** Another local unsigned process reads the inbox and merges into the tolerated row. The exception conceals it. Changing action 37 alone also leaves sidecar status and binding enforcement inconsistent. | **No as proposed.** It weakens a functioning security check without a proven replacement or approval. It can make the script pass by relaxing acceptance, not satisfy existing P-12. |
| **D: do nothing** | **[High]** A legitimate deployment keeps failing, while an actual additional reader produces no useful change in the already-red result. | **Yes regarding the no-switch-off rule.** It retires nothing, but leaves the operational conflict unresolved. |

**3. Missed alternatives**

- **Sidecar performs file-shape detection for its own inbox.** Reuse the existing signed read stream; preserve the exact shape rules, durable acknowledgments, backlog handling, alerts, and health reporting. **Cost: medium implementation and rollout effort. [High] risk:** checking only new arrivals or acknowledging before reporting loses violations across migration or crashes. **Action 37:** yes, after unsigned reads stop and existing rows expire, provided the sidecar remains the sole proven reader. This preserves P-12.

- **Signed, named audit reader with an explicit P-12 audit role.** Give the canary its own identity and narrowly authorized access; update binding enforcement, consumer classification, sidecar status, and action 37 together. **Cost: high policy and implementation effort. [High] risk:** treating every proven reader as an auditor admits unauthorized readers. **Action 37:** signing alone does **not** suffice; a second proven reader still fails, and a bound inbox rejects its different key. Passing requires an explicitly approved change to P-12.

- **Metadata-only inbox inspection.** **Cost: medium to high**, depending on whether a suitable endpoint exists. Topic names/counts cannot detect these shapes; detection needs per-message type and relevant metadata, with offsets for acknowledgments. **[High] risk:** calling topic-level metadata a replacement silently misses all file-shaped messages. **Action 37:** only if the endpoint provides sufficient detection information and is explicitly outside consumer recording under an approved policy. Using ordinary `subscribe` and discarding bodies still records a reader.

- **Detect shapes at hub ingestion.** Emit a durable finding when an offending message is accepted; let the canary inspect those findings. **Cost: high**, including historical coverage and reliable event retention. **[High] risk:** losing an event after accepting a post creates an undetected violation. **Action 37:** yes after inbox scanning ends and rows expire; no P-12 reader exception is needed.

**4. B’s races, errors, and required tests**

- **[High] Eligibility ambiguity:** test exactly-one-proven, proven-plus-unnamed, multiple-proven, wrong-agent-proven, and bound-with-no-recent-consumers cases. Assert both scan decisions and registry effects. Consumer history is neither a binding nor proof of replacement coverage.
- **[High] Binding appears after lookup:** the hub should reject the subsequent unsigned read atomically. The canary must report exit **2**, not interpret empty/error output as a clean scan. Test binding between lookup and read, and between pagination requests.
- **[High] Binding disappears after lookup:** a planned skip still happens, potentially after the sidecar has stopped. Detection coverage disappears. Test unbind and sidecar failure immediately after lookup. Rechecking only narrows this race; a coordinated handoff or enforced coverage invariant is needed.
- **[High] Consumers RPC fails or times out:** neither “read anyway” nor “skip successfully” is safe. Record incomplete coverage and exit **2**. Test timeout, nonzero status with valid-looking JSON, malformed JSON, and registry-integrity errors.
- **[High] Wrong hub:** a local lookup must not decide whether a remote inbox is covered. Pin lookup and read to the same verified hub/topic. Test mismatched hubs and routing changes. Remote inboxes are outside the current canary’s declared coverage.
- **[High] Skips and exit codes:** an unbacked skip is incomplete coverage, hence **2**. Exit **0** is defensible only for an approved, proven handoff with healthy replacement reporting; detected violations remain **1**. Report skipped topics and reasons explicitly.
- **[Medium] Acknowledgment loss:** current `--ack` rewrites state using scanned findings only. Skipping a topic drops its saved offset; resuming scanning can refire old acknowledged messages. Test skip/ack/resume and preserve skipped-topic state.
- **[High] Existing false-clean behavior:** subprocess return codes are ignored, and subscribe parsing silently discards malformed output. A refused read can therefore look empty and healthy. Test this directly. The exit heartbeat also fires on failure and cannot prove successful coverage.

**5. Choice**

I would choose sidecar-owned detection, followed by an explicitly approved handoff of inbox coverage after field evidence demonstrates equivalent detection, backlog handling, and failure reporting. It satisfies the existing single-reader rule while retaining the canary’s useful protection; I would reject B as written because its literal predicate cannot escape the current contaminated registry and its broader interpretation creates an unproven coverage gap. The strongest alternative is a signed audit reader: independent detection can catch failures in the sidecar responsible for processing the inbox. Its best case justifies the additional reader, but only through a deliberate P-12 revision with narrowly enforced audit authority—not an exception for unnamed traffic.
