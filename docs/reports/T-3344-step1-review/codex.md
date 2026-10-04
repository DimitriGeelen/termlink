# T-3344 step-1 review — Codex

## 1. Verdict and completion conditions

**1a. Verdict: revise before treating this as the authoritative requirements baseline.** The draft provides useful traceability and preserves all earlier requirement identifiers, but introduces substantive changes while claiming everything is unchanged. It can support preliminary threat analysis if those changes remain explicitly provisional.

**1b. Condition 6.2: structurally met, substantively incomplete.** Every requirement has the required fields, including cross-references for R-43. However, several acceptance criteria lack a determinate pass/fail condition: R-3.e’s unconfirmed bound, R-14.e’s proposed two seconds, R-17.e’s unspecified tolerance, and R-37.e’s undecided threshold (§6). R-1.e explicitly leaves the operator leg uncovered. These are verification gaps, although the card literally requires only one testable criterion per requirement.

**1c. Condition 6.3: partially met.** Adversaries are named (§2.3), and many prohibitions identify them. Coverage is inconsistent: R-16.a prohibits premature flagging, R-20.a prohibits non-urgent busy injection, and R-23.a prohibits peer content, without corresponding adversary scope. More seriously, R-34.f claims protection against compromised-session impersonation while R-34.e tests only an ordinary wrong-project routing case.

**1d. Condition 6.4: met structurally.** Glossary, conflicts and earlier-requirements sections exist (§§5, 7, 10). Their accuracy needs correction as discussed below.

**1e. Condition 6.5: presence met; rendering unverified; semantics need revision.** D-1–D-3 have identifiers, captions and textual equivalents (§3). Section 11.6 asserts rendering was checked, but the supplied text contains no check result to assess. D-3 embeds unsettled terminal-state and retry semantics.

**1f. Condition 6.1 is misstated.** Contrary to the draft’s header and §11.1, the card permits questions to be “recorded as an open gap.” An unfinished interview does not itself make 6.1 unmet. Operator sign-off remains separate.

## 2. Fidelity to confirmed requirements

**2a. Explicit narrowing.** Earlier R-3.2 becomes new R-7.a with independence limited to messages from the same host. This changes the statement, not merely its acceptance criterion as §10 claims. Earlier R-9.4 becomes R-29.a, limiting “halts” to “halts message-dependent work.” Both require a marked proposed change.

**2b. Added obligations.** Earlier R-6.2 → R-18.a adds FIFO ordering within equal priorities. Earlier R-7.1 → R-21.a adds mandatory fail-closed behavior for missing/unreadable readiness records. Earlier R-8.1 → R-23.a adds a prohibition on peer content; R-23.e additionally mandates counts, identifiers and a prewritten target claim. These may be sensible existing design choices, but they are not unchanged transcriptions of the cited requirements.

**2c. Acceptance criteria also change obligations.** Earlier R-4.1 → R-11.e requires receiver SHA-256 verification before flagging. Earlier R-3.4 → R-9.e adds serial startup and verification before each next level. Earlier R-4.2 → R-12.e forbids hub-topic posting on the successful push path. None follows automatically from the earlier requirement’s wording.

**2d. Lost intent.** Earlier R-12.4 → R-38 substitutes future database readability for the stated later observability database **and learning from it**. Earlier R-11.3 → R-34 preserves project isolation but omits the affirmative requirement that messages carry `to_circuit`.

**2e. Unsupported prioritization.** P1–P3 assignments are not identified as collector proposals. Confirmed startup-chain R-9 is assigned P3, although §6.1.c defines P3 as “later or unconfirmed.” Section 10’s blanket “none changed” is therefore inaccurate.

## 3. Neutrality and undecided questions

**3a. Recommendations are not inherently decisions.** Section 9 repeatedly leaves authority with the operator. Nevertheless, OD-17.d’s instruction to take recommendations “as the default” risks treating omission as consent; require explicit dispositions.

**3b. Unfair exclusivity claims.** OD-6.d says relaunch is the only option avoiding false delivery claims, although options B/C explicitly label pull-only reachability. OD-9.d mistakes compatibility with existing binary-only distribution for proof that other implementations cannot be packaged.

**3c. Unsupported implications.** OD-10.d invokes R-34 to justify exact-session isolation, but R-34 concerns different **projects**. OD-14.d declares canary alarms compatible with urgent-only alarms without an operator ruling on that distinction. OD-15.d scopes the startup chain to sidecars and host services despite confirmed R-9 explicitly including the agent, session and project.

**3d. Incomplete interview coverage.** OD-3 omits the fourth-rung “50 minutes”/15-minute ambiguity and urgent compression, both present in earlier §10/O11. OD-14 asks where final escalation lands but its options do not answer that. OD-7 asks ownership, then introduces an ownership allocation only in the recommendation.

**3e. Evidence limitation.** The comparison files themselves were not supplied. Their quotation accuracy and complete representation cannot be independently established here.

## 4. Weakest acceptance criteria

**4a. Adjacent behavior.** R-4.e tests command execution/output rather than clearly exercising keystroke injection. R-1.e’s broadcast criterion proves receipt, not many-to-many conversation. R-36.e checks digest production but not reflection.

**4b. Weakened success conditions.** R-10.e permits reporting failure instead of demonstrating successful re-resolution. R-28.e substitutes escalation for the required reply/no-action outcome and presupposes an undecided terminal state.

**4c. Ambiguous guarantees.** R-19.e equates immediate typing with the urgent requirement although R-24 distinguishes typing from evidence-backed injection. R-30.e does not distinguish successive waiting intervals from absolute polling times. R-38.e lacks a defined consumer or compatibility contract.

## 5. Threat-model handoff corrections

**5a. Separate authority levels.** Mark confirmed obligations, historical implementation facts, collector assumptions and proposed safeguards distinctly—including glossary definitions and drawings.

**5b. Correct lifecycle assumptions.** D-3 makes ESCALATED terminal and invents deadline/retry transitions without clearly marking them provisional. D-2 labels temporary unavailability as refusal but omits explicit rejection.

**5c. Define guarantee boundaries.** Receiver deduplication alone does not establish “exactly-once” effects (GP-5/CAND-2). State durability failure scope, trusted evidence producers and adversary privileges as open questions. Do not let a proposed threat list silently become an approved security guarantee.
