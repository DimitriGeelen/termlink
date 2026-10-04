# T-3335 external design review — comparison

**Reviewed:** `docs/design/interactive-agent-communication-requirements.md` (R-1.1..R-14.4, O1..O16), with the full design and the traceability map, against the 7 questions in `brief.md`.
**Date:** 2026-10-04 · **Task:** T-3335 (committed under T-3211; T-3335 is at its inception commit limit).

## Who answered

| Reviewer | Source on disk | Quality |
|---|---|---|
| Codex (OpenAI) | `codex.md` | Full, specific, answers all 7 |
| GLM-5.3 (opencode) | `glm.md` | Full, specific, answers all 7; disagrees most |
| 055-agentic-fleet-cockpit (peer agent) | `/opt/055-agentic-fleet-cockpit/docs/reports/T-445-termlink-design-review.md` (inbox @386) | Full, grounded in 055's own measured failures E1..E7 |
| qwen3 (local) | `qwen.md` | Shallow; cites decision ids that do not exist in the document (e.g. "D-700"); not weighted |
| gemma4 (local) | `gemma4.md` | Stops mid-answer at question 2; not weighted |
| AEF (peer agent) | — | Acknowledged (@383, @389, @394); answer deferred until AEF's vector-index repair is done |
| 832-Workflow-designer (peer agent) | — | No reply, no receipt |

The weighted reviewers below are Codex, GLM and 055. The two local models add nothing the three do not, and are listed for completeness only.

## 1. Where all three agree

1. **Verdict: not yet.** Codex: "Not yet, and not reliably as specified." GLM: "As a requirements record, yes; as a design for the operator's goal, not yet." 055: sound "for Claude Code agents launched armed on one host, once built", but "every step confirmed to the sender" is not yet met.
2. **The hub carries cross-host mail.** All three reject sidecar-to-sidecar push across hosts:
   2a. Codex: "retain hub-mediated transport … If cross-host delivery while hubs are down is mandatory, explicitly amend the charter: assigning the second transport to AEF does not remove it from the architecture."
   2b. GLM: "Cross-host sidecar-to-sidecar push *is* a second bus … The T-3330 ruling handing cross-host to AEF should be reopened."
   2c. 055: "Keep the hub as the cross-host carrier; sidecar-to-sidecar calls on one host only … A second bus adds a path, and E4 shows two paths diverge."
3. **Readiness comes from harness hooks, never from screen inspection**, and a ready flag is only an observation. The proof of delivery is evidence in the receiver's transcript (GLM, Codex), or evidence supplied by a harness adapter (055).
4. **The year-long polling ladder (R-10.1) is wrong.** Topics are retention-bounded, so late rungs poll for messages that no longer exist. All three want two separate mechanisms: bounded retry for re-sending, and a short wait that ends in a visible "stuck" escalation.
5. **The durable record is the truth; callbacks are an optimisation.** GLM: "nothing may depend on [the callback] — it fails exactly when the sender is down." 055: "Settlement must come from the durable record", keyed by message id. Codex: "Receiver progress must not depend on sender availability. Pull repairs missed notifications."
6. **One receipt vocabulary, evidence-backed.** Nothing may be called INJECTED/HANDED_OVER without transcript evidence (GLM: otherwise "ATTEMPTED"). All three add a terminal "no action" state.
7. **One owner.** Codex: "TermLink should own the transport-neutral message lifecycle … AEF should supply harness readiness/context adapters." GLM: "one owner, either TermLink's supervisor or AEF's watcher, not both." 055: "The framing still asks 'what to build', not 'whose build is the one'."
8. **The same acceptance test.** Two real agents, started normally; a fresh nonce must appear in B's transcript with B idle and with B busy in a long tool call; A's ledger shows each stage with timestamps; negative control: B's injector or receiver stopped, and A must see "waiting/overdue", never "delivered".
9. **What breaks first is deployment drift**: healthy-looking sidecars with no scheduler, wrong hub, missing hooks. Detected only by an end-to-end canary that sends a real message, not by process heartbeats.

## 2. Where they disagree (with each other, or with the confirmed design)

| Question | Confirmed design (read-back) | Codex | GLM | 055 |
|---|---|---|---|---|
| **Urgent into a busy agent** (O4, R-6.3) | Injected immediately, hard bypass | Against typing into a busy terminal; prefer an authenticated harness interrupt, else report the limit and escalate. Keeping busy-PTY typing must be "an explicit risk acceptance" | Against: "Typing into a busy PTY should remain forbidden, full stop." Deliver urgent via the Stop-hook context channel | **For** the bypass: record SQ-4 superseded; type only the fixed one-line doorbell, never content, message durable first; measure re-injects |
| **RECEIVED and STORED** (O2, R-5.1/5.2) | Two calls | **Keep both**, defined precisely: RECEIVED = volatile, STORED = durable, only STORED releases the sender | Merge: "answer once, after fsync-and-rename" | Merge: "RECEIVED after the durable write" |
| **Already-running sessions** (O3, R-1.3) | "No agent has to be attached" | Inventory capabilities; relaunch through the reachable launcher; UNREACHABLE until an end-to-end challenge succeeds | Hooks-first; relaunch through one wrapper; an unwrapped session is "pull-only — stated, not papered over" | **No relaunch** ("not realistic for a mixed fleet"); a harness-side pull channel at the harness's yield points, marked pull-only |
| **Sidecar independent of the hub** (R-3.2) | Yes | (not addressed) | Contradicted by the design itself, untestable | Untestable: "has no stated failure it buys" |
| **Packaging** (O9) | Ships with every deployment | One versioned runtime, ideally `termlink sidecar` | Option C: in the binary or an installed, versioned package | (one owner; no packaging view) |

Two of these cut against what you confirmed:

10. **Urgent bypass: two of three reviewers disagree with you.** Codex and GLM say typing into a busy prompt is the known silent-loss class (T-2396). 055 agrees with you, on the condition that only the fixed doorbell line is typed and the content is already stored. GLM offers a route none of the documents considered: deliver urgent content through the harness's own hook-context channel, which cannot be lost as unsubmitted input.
11. **Cross-host push: all three disagree with you.** Your read-back says sender sidecar to receiver sidecar first, with the hub as fallback. All three say the hub should be the carrier, and that the alternative is a charter change that must be decided openly, not inherited through the T-3330 ruling.

## 3. What only one reviewer raised

12. **Reachability as a first-class, visible state per agent** (055, its biggest weakness): receiver up, right hub, harness adapter present, last surface time, readable by peers and by you. Every 055 failure was "a receiver that looked fine and was not".
13. **Harness neutrality** (055): R-7.1 and R-8.2 name Claude Code's hooks and transcript. 055 runs opencode too. Wants an adapter contract with READY/BUSY/NOT RUNNING plus evidence, two adapters from day one, and a per-release parity test.
14. **An explicit mail-hub setting** separate from `TERMLINK_RUNTIME_DIR` (055, E2): agents started with `env -i` read an empty hub today.
15. **Session incarnation and fencing** (Codex): readiness generation, exclusive injection ownership, invalidation on restart; never silently redirect an exact-session message to another instance.
16. **Sender-side send-and-wait** (GLM): the four-call ladder gives visibility, but no way for the sending agent to yield and wait for the answer, which was asked for in April.
17. **Security and consent** (GLM, also 055 and qwen): sidecar-to-sidecar calls have no trust model; one host key serves every agent, so any project can impersonate another; urgent bypass lets a peer force-interrupt a working session.
18. **Attention budgets** (Codex, also 055): rate limits, bounded outstanding requests, expiry, cancellation; "successful delivery can itself make the agents unusable".
19. **Clock skew** across hosts corrupts the timestamped timeline (GLM); **version skew** makes a deaf agent look like an old one (055).
20. **Document drift** (Codex): the two documents number their open decisions differently; section 14 is still labelled unconfirmed; the full design proposes a screen fallback that R-7.2 forbids.

## 4. What this means for the next step

21. The two decisions where reviewers disagree with your confirmed design, urgent bypass (O4) and cross-host push (O1), go first, one at a time, each with the reviewers' arguments quoted.
22. Points 3 to 9 have unanimous support and do not conflict with anything you said, except the polling ladder (point 4), which changes requirement R-10.1 and therefore needs your ruling too.
23. The single-reviewer points (12 to 20) become proposed requirement changes under O16, for you to accept or reject.
24. Codex's drift point (20) is ours to fix: the two O1..O16 lists get merged into one numbering before the walk-through starts.
25. AEF's review and any reply from 832 will be added here when they arrive. AEF's matters most for points 7 and 13, because AEF built the parallel receiver.
