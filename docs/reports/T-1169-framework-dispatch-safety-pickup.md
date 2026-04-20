# T-1169 — Framework Dispatch Safety Pickup Delivery

**Status:** work-completed (2026-04-20T19:02) · meta-inception · owner: human (for formal `fw inception decide` marker)
**Task file:** `.tasks/completed/T-1169-pickup-to-framework-multi-agent-dispatch.md`
**Framework-side task:** T-1365 (`Pickup: Multi-agent dispatch safety model …`)
**Delivered envelope:** archived at `/opt/999-Agentic-Engineering-Framework/.context/pickup/processed/P-T-1169-framework-dispatch-safety.yaml`

## One-sentence framing

Meta-inception: formulate and deliver a pickup envelope to the framework so they can scope worktree-isolation + parallelism-metadata + dispatch-gate primitives that make multi-agent dispatch safe for concurrent code work.

## Triggering dialogue

While planning parallelized work on T-1155 bus follow-ups (T-1158 bus-crate scaffold + T-1159 ed25519 identity keyring), we hit the worktree-collision problem: both tasks edit workspace `Cargo.toml`. Agent listed what parallelizes safely today (scope-only, research spikes, independent-file work) and what doesn't (concurrent crate/Cargo.toml edits). User asked:

> "ok fair:: can we formulate improvement which we can send as inception pickup taks using termlink to our framework agent?"

This task is the response.

## What the envelope proposes (P1..P5)

Not implementing — asking framework to scope and decide.

- **P1. Worktree isolation primitive** — `fw worktree spawn T-XXX` → `.worktrees/T-XXX/` from HEAD, registered in task frontmatter, auto-cleaned on completion.
- **P2. Task parallelism metadata** — frontmatter `touches: [paths]` + `parallelism_class: scope-only | file-isolated | worktree-required | serial`.
- **P3. Dispatch gate** — `fw dispatch T-XXX` checks class + active-worker table; refuses unsafe combos.
- **P4. Reconciliation primitive** — extend `fw bus` with parallel-merge mode that flags overlapping edits.
- **P5. Bootstrap note** — document that today's coordination rides `event.broadcast`/`event.collect` until T-1155 bus lands; this is fine but circular.

## Delivery trail

1. Task T-1169 created (`fw work-on T-1169`, status started-work, owner agent).
2. Envelope drafted with `type: inception-proposal` — rejected by processor (invalid type).
3. Corrected to `type: feature-proposal` — accepted.
4. Sent via `termlink file send tl-ismotg7j ...` (2× sends, both `Transfer complete`).
5. Belt-and-suspenders direct drop to `/opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/`.
6. Processor cycle (~30s) moved envelope to `processed/`; dedup.log entry `2026-04-20T19:01:05Z|87166a7e…`.
7. Framework-side task T-1365 auto-created (captured / inception / agent-owned / horizon:next).
8. Verification gate 3/3 PASS, Agent ACs 4/4 checked, status → work-completed.

## Recommendation

**DELIVERED** — scope decision owned by framework via T-1365. No further termlink-side action until framework decides GO/DEFER/NO-GO on the proposed primitives.

## Dialogue Log

### 2026-04-20T18:55..19:02Z — formulate + deliver

- Surfaced the problem (worktree collisions when parallelizing T-1158+T-1159).
- User asked for a framework-directed pickup.
- Task created, envelope drafted with detail covering P1..P5 + composes-on list + scope fence + out-of-scope notes.
- First delivery rejected on `type` field — fixed to `feature-proposal`, redelivered.
- Processor picked up within ~20s of the corrected envelope landing in inbox.
- Framework T-1365 visible immediately; closes the termlink-side loop.

## Learning captured

**PL-040:** Pickup envelope `type:` field is a closed vocabulary: `bug-report | learning | feature-proposal | pattern`. Arbitrary values (including semantically reasonable ones like `inception-proposal`) are auto-rejected. Source: `lib/pickup.sh:92` + `:353`.

---

_Artifact written 2026-04-20T19:05Z under T-1170 (C-001 housekeeping) after T-1169 closed before artifact existed. P-002 prevents writing under completed tasks — pattern for future: write at least a skeleton research artifact under inception tasks before closing, even for meta-inceptions._
