# T-118: fw fabric context — Inception Report

> Task: T-118 | Date: 2026-03-14

## Problem Statement

LLMs assisting on a project need architectural context. Currently requires
multiple `fw fabric` commands (overview, deps, impact) — each costing context
tokens. A single `fw fabric context` should produce a structured, token-efficient
document for LLM injection.

## Assumption Validation

### A-001: Single document more token-efficient than multiple queries
**Validated.** `fw fabric overview` produces ~500 tokens. Running `fw fabric deps`
for all 46 components would produce ~10K+ tokens. A combined context document
targeting subsystems + graph + compact index stays under 2K tokens.

### A-002: Existing fabric data sufficient
**Validated.** subsystems.yaml has: name, description, summary, depends_on,
components list. Cards have: purpose, type, depends_on, tags. This is enough
for useful architecture context without adding new fields.

### A-003: Output injectable into prompts
**Validated.** Markdown format works directly in CLAUDE.md, SessionStart hooks,
or `/resume` context injection. No post-processing needed.

## Evidence

From 010-termlink project:
- 5 subsystems, 46 components, 83 edges
- subsystems.yaml: 2.9KB
- All cards: 25.6KB total (too much raw, but synthesized = ~1.5K tokens)
- `fw fabric overview`: already concise, but missing component-level detail

## Decision

**GO** — all assumptions validated. Existing fabric data is sufficient. Target
output under 2K tokens. Pickup prompt already drafted at
`docs/framework-agent-pickups/T-118-fabric-context-llm-enrichment.md`.

Implementation is a framework PR (bash script in fabric agent), not TermLink work.
