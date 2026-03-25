# T-283: Cross-Session Failure Blindness — Investigation Synthesis

## Executive Summary

The .107 remote access failure is not a one-off bug. It's a symptom of a **systemic framework gap**: cross-session failures are invisible because every observability mechanism is session-scoped. The escalation ladder (A→B→C→D) is an aspirational rule, not a structural guarantee — violating the framework's own P-002 ("Structural Enforcement Over Agent Discipline").

## Evidence (Agent 1)

**3+ documented occurrences** across sessions:
1. **2026-03-18 (T-163):** Live testing discovered 3 bugs during first .107 connection
2. **2026-03-21 (T-209):** Inception revealing .107 unreachable without TermLink pre-installed
3. **2026-03-25 (this session):** SSH'd instead of TermLink, fumbled with missing secret — again

**8 tasks (T-197–T-204)** have unchecked Human ACs for .107 testing. Zero escalated. Zero patterns registered.

## Memory System Audit (Agent 2)

- Reference memory exists but lacks **actionable commands** (rules without solutions)
- `learnings.yaml`: **EMPTY** — zero learnings captured in entire project
- `patterns.yaml`: **seeded-only** — no project-specific failure patterns
- `concerns.yaml`: **EMPTY** — zero gaps registered
- Memory format problem: "don't SSH" without "do THIS instead (exact command)" is useless

## Hook/Observability Audit (Agent 3)

**Within-session detection: functional**
- `error-watchdog.sh`: detects failed Bash commands, advisory only
- `loop-detect.sh`: detects 5+ repetitions of same tool, session-scoped
- `checkpoint.sh`: token budget monitoring

**Cross-session detection: ABSENT**
- `loop-detect.json` is destroyed between sessions
- `error-watchdog` findings are not persisted
- Handover agent checks episodic completeness but **never checks for recurring failure patterns**
- `concerns.yaml` is empty — gaps register unpopulated

**Vendor permission bug** (51 scripts missing +x) meant loop-detect and other 1.3.0 hooks were DOA — observability was silently disabled.

## Escalation Ladder Audit (Agent 4)

**Critical finding: The escalation ladder violates P-002.**

- P-002 says "Structural Enforcement Over Agent Discipline" — don't rely on agents to remember rules
- The escalation ladder is a markdown rule that agents must remember to follow
- Compare: task gate (PreToolUse hook) is **structural** — blocks Write/Edit without a task
- Escalation ladder is **aspirational** — no hook, no gate, no enforcement

**T-103 (Error Escalation auto-population)** was previously explored and **DEFERRED** — the very feature needed was deemed out-of-scope.

**The systemic pattern:** "Failures that appear once per session never escalate because each session treats them as novel."

## Remediation Scoring (Agent 5)

| Option | Antifragility | Reliability | Usability | Portability | Weighted |
|--------|:---:|:---:|:---:|:---:|:---:|
| A: Profile + Memory Fix | 1 | 2 | 4 | 5 | **3.1** |
| B: Cross-Session Failure Register | 4 | 5 | 3 | 4 | **4.1** |
| C: Handover Pattern Flagging | 3 | 4 | 4 | 5 | **4.0** |
| D: Pre-Connection Hook | 2 | 3 | 2 | 3 | **2.6** |
| E: Full Escalation Automation | 5 | 5 | 2 | 3 | **3.8** |

**Recommended: Option A (immediate) + Option B (structural)**

- A fixes the symptom NOW (profile saved, memory updated — already done this session)
- B fixes the root cause (persistent failure register checked at session start)
- B is also the foundation for E if needed later

## Layers Still Missing (acknowledged)

The 7-layer analysis identified issues the agents confirmed, but there may be more:
- How does this interact with multi-project workflows? (agents working across repos)
- What about failure patterns that span PROJECTS, not just sessions?
- Is the handover format itself part of the problem? (too much noise, recurring issues buried)
- Should `fw doctor` include a "known failures" health check?

## Immediate Actions Taken This Session

1. ✅ Remote profile `mint` saved to `~/.termlink/hubs.toml`
2. ✅ Memory updated with exact working TermLink commands
3. ✅ T-284 fixed: remote inject/send-file clap panic
4. ✅ Pickup sent to fw-agent on .107 about vendor permission bug
5. ✅ Framework upgraded to 1.3.0, fixed vendor +x permission bug
6. ✅ GitHub issue #12 filed on framework repo

## Recommended Next Steps

1. **GO/NO-GO decision** on Option B (cross-session failure register)
2. If GO: create build task for `.known-failures.yaml` + handover integration
3. Register this entire investigation as a learning in `learnings.yaml`
4. Register gap in `concerns.yaml`: "Escalation ladder is aspirational, not structural (violates P-002)"
5. Send this synthesis to fw-agent on .107 for framework-side remediation
