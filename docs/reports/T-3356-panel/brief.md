# T-3356 review panel brief: attack the arc-011 interactive agent communication design

**Task:** T-3356 (arc-011 design role chain, step 7: review panel and disposition)
**Brief version:** 1, 2026-11-11
**Material revision:** repository commit `c9aea3befd7ce44751ea7f9bf954da02a65c6845` plus the file hashes in section 2
**Your seat:** one of `claude-code`, `codex`, `opencode`, `antigravity` (the orchestrator tells you which)

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 1 | 2026-11-11 | First brief, panel A (operator ruling 2026-11-11) | T-3356 |

## 1 Your role and the rules of this round

1.1 You are an independent reviewer. You did not write this design. Your job is to find what is wrong with it,
not to summarise it or praise it. A finding you are unsure of is still worth reporting: mark the doubt in the
finding.
1.2 You answer alone. You do not see the other seats' answers and you MUST NOT read anything under
`docs/reports/T-3356-panel/` other than this brief.
1.3 You MUST write your whole answer to exactly one file, `docs/reports/T-3356-panel/<seat>.md`, where `<seat>` is
your seat name from the header. You MUST NOT edit, create, move or delete any other file, run git commands that
change state (commit, add, checkout, reset, stash, push), start services, or contact any network host. Reading files
and running read-only commands (`cat`, `grep`, `sed -n`, `git log`, `git show`, `sha256sum`) is fine.
1.4 Time-box: one session. If you run short, report what you have, and list the sections you did not reach in
section 5 of your answer. A partial answer that says what it skipped is better than none.
1.5 Do not copy any secret value, key, token or credential into your answer, even if you meet one in the repository.
Refer to it by file and line instead. Do not write exploit code; describe a bypass in prose, precisely enough that
the author can reproduce the reasoning.
1.6 Before you start, check the hashes in section 2 with `sha256sum`. If any differs, say so at the top of your
answer and review what you find, naming the hash you read.

## 2 What to read (run in the repository root)

2.1 **The material under review** (the step-7 inputs). Read these in full.

| Id | Path | Version | sha256 |
|---|---|---|---|
| REQ | `docs/design/interactive-agent-communication-01-requirements.md` | v0.5.3 | `f6ce42a2984bd18e049f1f4dda05e6eaab2cc3508396fe373060c6fc527193c9` |
| ARC | `docs/design/interactive-agent-communication-04-architecture.md` | v0.3.1 | `29c6d1f7de9f18898cab105651f17977ebf97da30f32d16501819bad217dd69c` |
| PC | `docs/design/interactive-agent-communication-06-pseudo-code.md` | v0.2 | `7d469813239e991304ddad04b35fe2dc713c1d0e2ed8142ee7c05b5ff4dbffcc` |
| PCa | `docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md` | v0.2 | `1db1532a8d1d4c1002326a4b4e948be959b7a52c5bd33b8fba0ee2f405b69ebf` |
| PCb | `docs/design/interactive-agent-communication-06b-pseudo-code-later-phases.md` | v0.2 | `6b4e5296fe7673360ff054919c15a851fdb5bdc8af366f6222de79967e092ddf` |

2.2 **Context** (read the parts you need to judge the material; findings against these are welcome too).

| Id | Path | Version | sha256 |
|---|---|---|---|
| THR | `docs/design/interactive-agent-communication-02-threat-model.md` | v0.3.7 | `88961693384966502f4ca4eb515ed4b0590a29c3d61ac0c619e17ea4562693f3` |
| FP | `docs/design/interactive-agent-communication-03-floor-and-phasing.md` | v0.3 | `e4de2e3e102ee59f6434db584e099e5bb180c6b33dfd823ae217e6ebb4e5a8c4` |
| EV | `docs/design/interactive-agent-communication-05-evidence.md` | v0.1 | `29be80ae3ad8d32b71ef53caa00f9fdc6d03426c901215eaed85c019f889526e` |
| TRC | `docs/design/interactive-agent-communication-04a-traceability.md` (optional) | v0.3 | `c516a290997e0c75957c0be95af4d611e58155cd818d2fbae961c0f4ad6d4667` |
| COV | `docs/design/interactive-agent-communication-05a-coverage.md` (optional) | v0.1 | `16a4aa77b23cad191cf1b9153e60016a266ffc763c54674cb2d6e1f6341824fc` |

2.3 **The operator's rulings O-1..O-7** (2026-11-03) are recorded in ARC: its version-history row 0.3 lists all
seven, and each is marked in the text where it applies. In short:
2.3.a O-1 A: cross-hub file delivery through `artifact put --hub` (ARC 6.4.6, DR-5); other file routes out of scope
and left working.
2.3.b O-2 D+: the doorbell types only when the session is READY, nobody is typing and no conversation is running;
an external 300 s watchdog; per-session activity and conversation signals (ARC 6.9, 6.14, DR-15).
2.3.c O-3 A with a tightening: the main lease is renewed only by human-initiated prompts, with fast takeover and a
"mail waiting, main not reading" signal (ARC 6.13, DR-9, DR-16).
2.3.d O-4 A: operator runme action and the session-message prover re-based on TermLink (ARC 6.10.2, B-18, B-26).
2.3.e O-5 A: the starting values V-1..V-13 are accepted as one package, each measured and tuned in operation
(ARC 8.1).
2.3.f O-6: approvals through Watchtower and the operator's out-of-band channels, no new hub verb; reliability
thresholds measure reliability, never usage; nothing working is switched off until its replacement is proven in the
field and the operator approves (ARC 12.2, 15.2, 16).
2.3.g O-7 A: three fixed per-project security levels (ARC 8.4, DR-13).
2.4 The project's standing rules are in `CLAUDE.md` above the line `## Core Principle`. Two bind this design
directly: reliability thresholds measure whether things work, not how often they are used; and nothing working is
switched off without a proven replacement and the operator's recorded approval.

## 3 Where to spend your effort

3.1 **Most important: the PH-2 delivery path, PC section 5, algorithms AL-1 to AL-16.** Build item B-27 (the
sidecar activity meter, AL-1..AL-4) is already being built, and B-16, B-25 and B-20 follow it. A flaw found here
now is cheap; after the build it is not. Give this part your closest reading, algorithm by algorithm, and check each
against the ARC sections it cites and against PC section 6 (the FAILS CLOSED WHEN register).
3.2 Then the PH-2 hardening (PCa, AL-17..AL-40), then the later phases (PCb), then REQ and ARC as a whole.

## 4 What to attack

For each, look for the concrete case where the design breaks, not for general remarks.

4.1 **Requirements coverage.** A requirement in REQ that no part of ARC or PC implements, or implements only in
part; an algorithm that does something no requirement asks for; a requirement whose acceptance cannot be measured.
4.2 **Threat coverage.** A threat in THR that ARC or PC does not counter, counters only by assumption, or counters
in a later phase than THR or FP requires; a residual risk the operator accepted that the design has quietly widened.
4.3 **Bypasses.** A way for a sender, a peer agent, a compromised session or a misbehaving harness to get a message
or keystrokes into a session that the design says must refuse it, to impersonate another agent or hub, to take the
main lease, or to make the operator approve something they did not mean to. Describe it in prose, not as code.
4.4 **Algorithm flaws.** Races between concurrent writers or between check and act (time-of-check / time-of-use,
especially around typing into a session); behaviour after a crash or restart at every step (what is lost,
duplicated or left locked); any place that fails open where it should fail closed, or reports "delivered", "clear"
or "healthy" without having looked; unbounded queues, retries or waits; clocks and timeouts that can drift or wrap.
4.5 **Evidence gaps.** A claim in ARC or PC that EV does not test, a test that would pass when the property is
broken (no negative control), or an acceptance that measures liveness or usage instead of the property.
4.6 **Portability.** Anything tied to one harness, vendor, operating system or terminal that the design presents as
neutral (in particular the adapter contract, ARC section 17); anything that only works for Claude Code sessions.
4.7 **Contradictions of the operator's rulings O-1..O-7** (section 2.3) or of the two standing rules (2.4):
anywhere the text, an algorithm or a build item does something a ruling forbids or fails to do what a ruling
requires.
4.8 Anything else you judge wrong, unclear enough to be built two ways, or internally inconsistent between REQ,
ARC and PC.

## 5 The form of your answer

Write `docs/reports/T-3356-panel/<seat>.md` with these sections, in this order.

5.1 **Header:** seat, the model and version you are running as (if you know it), the date, and the hashes you
checked (say "all match" or list the differences).
5.2 **Verdict:** at most ten lines. Is the design fit to plan the build from (yes / yes after the critical and high
findings are fixed / no)? What are the two or three things that matter most?
5.3 **Findings,** one block per finding, numbered `<SEAT>-1`, `<SEAT>-2`, ... (for example `CODEX-3`), in this form:

```
### <SEAT>-n
- Where: <document id from section 2> <section or algorithm id, e.g. PC 5.6 AL-6 step 4>
- Severity: critical | high | medium | low
- Class: coverage | threat | bypass | algorithm | evidence | portability | ruling | other
- What is wrong: <one or two sentences>
- Why: <the reasoning or the concrete scenario that shows it; cite the text>
- Fix: <what would fix it, and which document should change>
```

5.3.a Severity: **critical** = a message is lost or forged, keystrokes reach a session that must refuse them, or a
fail-open on the delivery path; **high** = a correctness or security flaw that will be built in unless changed now;
**medium** = a real gap with a workaround or a later phase to catch it; **low** = clarity, consistency, wording.
5.4 **What you checked and found sound:** a short numbered list, so the absence of a finding means something.
5.5 **Not reached:** sections or algorithms you did not review, if any.
