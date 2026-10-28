<!-- T-3612 round 4, seat claude, source /opt/termlink/.context/dispatch/t3612-r4-claude-0e3599f8ce26/result.md, exit 0 -->
I've written the review to `/opt/termlink/.context/dispatch/t3612-r4-claude-0e3599f8ce26/result.md`. That path came from the dispatch script, since the brief didn't name one. It is about 1,500 words by `wc`, which also counts markdown symbols, so the prose is under the limit. Nothing else in the repo was edited, and I read the local checkout at 26bccc7bf rather than GitHub.

**Verdicts:** change on every section; nothing rejected. For the purpose, I recommend the main sentence over the terminals-first one, with "or is loudly stuck" added and the operator named as a user.

**Problems in the tooling (found by reading the code):**
- **D2 = 10 cannot be entered.** `fw bvp weight --set` only accepts 0–9 (`.agentic-framework/lib/bvp.sh:962`). I proposed 9.
- **The redefined product F-RECALL would be scored by the old scorer.** The estimator maps the id F-RECALL to its existing AEF-memory scorer (`estimator.py:2631`, `:526-560`), so it would score `fw recall`, qdrant and CLAUDE.md work. It needs a new id.
- **F-TRUST and F-SURFACE have no scorer.** They fall back to a keyword match that scores 0 unless the task text contains the literal id. Each needs a scoring block and a rubric section.
- **The live driver file should stay the development set for now.** `lib/bvp.sh` reads only `policy/value-drivers.yaml`, and about 92% of commits are development-project work. Putting the product set there would score most tasks on the wrong axes. Until AEF supports per-layer sets, product arcs can use the arc-scoped drivers that already exist (`bvp.sh:1503`).
- **The O-n mapping the draft plans for is nearly free.** No arc has a `supports:` field, no code reads O-n ids, and only T-3371 and T-3477 cite them.

**Changes to the content:**
- **Restore what P-1 dropped from O-1 and O-2:** the gap notice, hub responsiveness under load, the no-ghosts listing measure, and G-088 (exactly-once only within a hub lifetime).
- **Add a P-8 for the bounded record** the operator ruled on in Q3. It is also the objective the product recall driver can be scored against. That driver's retire condition becomes "P-8 green", and "find by peer" must mean picking a thread, not filtering across topics.
- **Remove overclaims:**
  - P-5 says agents are isolated from each other, but shared root is accepted for now.
  - P-7's "removed on a schedule" decides Q6 before the data exists.
- **Fix layer leaks:**
  - D3's reason (operator interrupted only for decisions) is DP-1, which is development-project work.
  - DP-3 should say "P- or DP-objective".
  - Section 6.2 lists DP-1..DP-4 but the draft has DP-5.
- **Non-goal 3 is already violated:** poll and emoji-stats tools are still compiled, only marked deprecated (`tools.rs:1176`, `:1203-1205`). Non-goal 2 is partly violated: 75 forever topics have no owners.
- **Dropping F-ORCH fits Q2.** The draft also misses that AEF itself retired F-ORCH (T-2511).

Confidence is high on the tooling findings and medium on the wording of the objectives. I did not read the round-1 seat files in full (I relied on the synthesis), arc-011 PH-2..PH-4, `docs/ARCHITECTURE.md`, all 273 MCP tools, or the canary registry count.
