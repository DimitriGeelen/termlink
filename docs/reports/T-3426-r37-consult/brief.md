# Independent consult: when should routine "needs attention" entries escalate? (arc-011 R-37)

You are an independent reviewer. Work from the text below only; do not read other files and do not modify
anything. The operator asked for an outside opinion before ruling. Disagree where you see reason to; a
confirmation without reasons is of no use. Answer in at most 600 words, with numbered points.

## Context

1. TermLink runs per-agent "sidecars" that receive, store and hand messages to AI coding agents (Claude Code
   sessions) on one host. Each sidecar writes "needs attention" entries when something goes wrong (about 15 kinds:
   dead-letter, receive-stalled, claim-failed, hub-id-mismatch, outbox-unreadable, quarantined record, ...).
2. Operator rule (R-37): "Liveness alarm only for urgent messages. Everything else accumulates and escalates when it
   piles up, like audit warnings." A later ruling: a failure of the daily end-to-end canary is an escalation entry,
   never an alarm; the last escalation rung is the main agent's session-start "needs attention" list, plus a
   `/canaries` view, a daily digest and (later) a cockpit. The escalation THRESHOLD was left open.
3. Built today: an urgent message overdue at the receiver raises an alarm at once. Routine entries are listed at
   each session start as counts per kind ("3 x receive-stalled, latest: ..."); the reader marks them seen. Nothing
   escalates: a fault that recurs daily stays a count line.
4. Precedent in the same project ("like audit warnings"): code-check warnings escalate when a check stays red more
   than 14 days, or when more than 5 checks are red at once; escalation files one task per check and makes a canary
   log fire.
5. Attention budget already set (H-20, measured over 14 days in this phase): at most 5 needs-attention entries per
   day and at most 1 urgent alarm per day; more than 3x these values counts as evidence against the approach.
6. Phase: the first real-agent wave (one pair of agents on one host) is about to start; 14 days of real data come
   after it.

## Options being considered

- A: escalate when a sidecar raises more than 5 routine entries in 24 h (H-20 budget), OR the same kind appears on
  3 or more separate days within 7 days; both numbers configurable and retuned after the first 14 days of data.
- B: copy the code-warning numbers: a kind still occurring after 14 days, or more than 5 kinds open at once.
- C: volume only: more than 5 routine entries in 24 h.
- D: defer: counts only now; set the threshold from the first 14 days of measured data.

"Escalate" in A-C = an ESCALATED line first in the session-start list, a canary log entry (shown as firing), and
one task filed per kind (de-duplicated). Never an alarm.

The proposer recommends A (scored A +48, C +20, B +15, D -17 against weighted value drivers: antifragility 9,
reliability 7, usability 5, failure-surfaced 5, operator-simplicity 4, unattended-operation 4).

## Questions

1. Which option would you pick, and why? Is there a better option not listed?
2. Are the triggers in A well chosen? Specifically: is "per sidecar" the right unit (vs per host / per kind across
   sidecars); is counting "separate days" robust (time zones, a burst at midnight); is 5/24 h sensible given H-20 is a
   14-day AVERAGE, not a daily cap?
3. What failure shape would A miss, or what would make A noisy (alert fatigue)?
4. Is filing one task per escalated kind the right action, or does it create clutter? What should de-escalate /
   clear an escalation?
5. Anything in the proposer's framing you think is biased or wrong.
