## Agent Delegation Event Schema — Review

**Schema completeness:** Covers the full happy-path lifecycle (delegate → accepted → completed → failed). Missing: `task.progress` for long-running work (progress %, heartbeat), `task.cancelled` for orchestrator-initiated cancellation, and `task.timeout` as a distinct event vs. silent expiry.

**Field naming:** Clean and consistent (`request_id`, `result_path`, `reply_to`). Uses snake_case throughout. `scope` as a free-form object is pragmatic but unvalidatable — consider a `type` discriminator field so consumers can parse scope structurally.

**Extensibility:** Topic hierarchy (`task.*`) is forward-looking and noted for `session.*`, `health.*`. Payloads are open (no `additionalProperties: false`), so adding fields is non-breaking. No schema versioning field — adding `schema_version` would protect against future breaking changes.

**Failure handling:** `task.failed` has `error` + `recoverable`, which is good. Missing: error codes/categories for programmatic routing (vs. human-readable strings only), partial failure semantics, and retry metadata (attempt count, backoff hint).

**CloudEvents / CNCF comparison:** CloudEvents mandates envelope metadata (`specversion`, `id`, `source`, `type`, `time`). This schema embeds correlation (`request_id`) but lacks timestamp, source identity, and content-type metadata. The "write to disk, return path" pattern is a pragmatic divergence from CloudEvents' `data` field — justified for LLM context budget, but reduces interoperability. Adopting a CloudEvents-compatible envelope wrapper would be low-cost and would future-proof integration with CNCF tooling (Knative, Argo Events).

**Verdict:** Solid for its scope — minimal, purposeful, well-documented. Main gaps: no progress/cancel events, no schema versioning, no structured error codes, and no CloudEvents alignment. These are refinements, not blockers.
