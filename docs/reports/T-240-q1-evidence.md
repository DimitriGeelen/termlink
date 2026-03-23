# T-240 Q1: Evidence of Format Correction in TermLink History

**Task:** T-240 — Negotiation Protocol (4-Phase Format Negotiation)
**Question:** What evidence exists of agent outputs being corrected, reformatted, or rejected due to format issues?
**Date:** 2026-03-23

---

## Summary

Across 233 completed tasks, 215 episodic summaries, and the full project memory corpus, **zero instances of iterative agent-to-agent format correction** were found. What does exist is a rich history of **format-adjacent bugs** — field mismatches, schema gaps, serialization failures — that were discovered through testing or human observation, not through runtime negotiation.

This is the central finding: the project has no format negotiation mechanism, so format problems manifest as bugs rather than corrections.

## Evidence Inventory

### Category 1: Field Name / Schema Mismatches (3 instances)

| Task | Problem | Discovery | Cost |
|------|---------|-----------|------|
| **T-177** | CLI read `bytes_len` but RPC returned `bytes_written` — display always showed 0 | Human noticed silent failure | 1 bug-fix task, ~1 hour |
| **T-171** | `event.poll` cursor initialized to `Some(0)` instead of `None` — skipped first event | Test failure | 1 bug-fix task |
| **T-198** | `poll_cursor` started at `None`, replayed 30-min-old stale `FileInit` events | Human observed stale data during file transfer | 1 bug-fix task |

**Pattern:** Field name and initialization mismatches between producer and consumer. These are structural format errors — the data shape was wrong, not the content. All were one-shot fixes once the root cause was identified.

### Category 2: Serialization / Encoding Failures (3 instances)

| Task | Problem | Discovery | Cost |
|------|---------|-----------|------|
| **T-139** | Raw ANSI escape sequences in terminal output broke JSON parsers | T-136 spike during framework self-test | 1 build task (strip_ansi server-side + CLI flag) |
| **T-167** | Agent message protocol schema had serde roundtrip failures | Caught by roundtrip tests during development | Fixed in-flight, no separate task |
| **T-248** | Bypass registry truncate+write race caused silent data loss under concurrent writes | Human investigation after promotion data vanished | 1 high-severity bug-fix (atomic rename + file locking) |

**Pattern:** Serialization assumptions that hold for single-writer/single-reader but break under real conditions (concurrency, binary data, special characters). T-139 is the closest analog to format negotiation — terminal output contained unexpected encoding that broke downstream consumers.

### Category 3: Schema Evolution Gaps (2 instances)

| Task | Problem | Discovery | Cost |
|------|---------|-----------|------|
| **T-069** | Missing `task.progress`, `task.cancelled` events; no `schema_version` field; no structured error codes | Design review | 1 build task — added version field, error enum, 2 new event types |
| **T-061** | No formal schema for agent delegation events | Inception research | 1 specification task — defined 4-stage lifecycle with JSON schemas |

**Pattern:** Schema designed for initial use case, then extended when new consumers needed fields that didn't exist. Not corrections per se — more like schema negotiation happening at design time rather than runtime.

### Category 4: Output Format Enforcement (1 instance)

| Task | Problem | Discovery | Cost |
|------|---------|-----------|------|
| **T-128** | Mesh worker agents produced inconsistent output (no commit instructions, wrong cargo path, variable format) | Pattern observed across multiple dispatches | 1 build task — prompt template wrapping all mesh worker prompts |

**Pattern:** This is the most relevant precedent for T-240. The solution was **preventive** (standardized prompt template) rather than **reactive** (negotiate/correct at runtime). The template enforces format by injecting instructions before the agent sees the task.

### Category 5: Recorded Learnings

From `learnings.yaml`:

- **L-005:** "CLI field name must match RPC response field exactly — bytes_written vs bytes_len mismatch caused silent display bug (always showed 0)" (T-177)
- **L-001:** "Component fabric requires typed edges (target+type format), not plain strings" (T-043)

From `patterns.yaml`: No format-failure patterns recorded. Format issues were treated as regular bugs, not as a recurring failure class.

## Analysis

### How many instances? **9 total**, across 5 categories.

### What kinds of corrections?

| Type | Count | Examples |
|------|-------|---------|
| **Structural** (wrong field names, missing fields) | 4 | T-177, T-171, T-198, T-069 |
| **Encoding** (binary data in text channels) | 2 | T-139, T-167 |
| **Atomicity** (format correct but write corrupted) | 1 | T-248 |
| **Schema evolution** (format incomplete for new use case) | 1 | T-061 |
| **Consistency** (agents producing variable formats) | 1 | T-128 |

### Were corrections iterative or one-shot?

**All one-shot.** Every instance was: discover problem → diagnose root cause → fix. No evidence of multiple correction rounds or back-and-forth negotiation. This is expected — without a negotiation protocol, there's no mechanism for iterative correction.

### What was the cost?

| Cost type | Count |
|-----------|-------|
| Full dedicated bug-fix task | 6 |
| Fixed in-flight during development | 2 |
| Design-time prevention (template) | 1 |

Average cost: **1 task per format issue**, typically a few hours. No catastrophic failures, but several were **silent** (T-177 showed 0 instead of failing, T-248 lost data without error).

## Implications for T-240

1. **Silent failures are the real cost.** Format mismatches don't crash — they produce wrong results silently. A negotiation protocol's primary value would be making format incompatibility loud and fast.

2. **Prevention > Correction.** T-128 (prompt template) is the only preventive measure. Everything else was reactive. A negotiation protocol could shift more issues to prevention.

3. **No iterative correction precedent exists.** The project has never done multi-round format negotiation. T-240's 4-phase protocol would be entirely new capability, not formalization of existing practice.

4. **Schema versioning exists but is unused for negotiation.** T-069 added `schema_version` to events, but it's a documentation field, not a negotiation signal. T-240 could build on this.

5. **The format problem scales with agent count.** With 1-2 agents (current mesh), prompt templates suffice. As orchestrator.route (T-237) enables dynamic specialist discovery, format assumptions between unknown agent pairs become untenable.
