# Consultation brief: who may become "main" before signed eligibility profiles exist

You are an independent reviewer. Give your own view; do not assume the recommendation below is right. Answer in under
500 words, structured as: (1) your choice and why, (2) the strongest argument against your choice, (3) failure modes
you see in each option, (4) anything the options miss. Write your answer to the file named in your task.

## System
An agent-to-agent messaging system. Each PROJECT has one or more AI agent SESSIONS (interactive terminal sessions;
some projects run several at once, e.g. one the human operator works in plus forgotten background ones). Peer mail
addressed to the project must be handled by exactly ONE session, the "main", so it is never taken by a session the
operator does not look at (this happened: a 17-day-old background session took a password request and sat on it).

Built and live: a lease per project at the project's home hub. A session claims it on start and on each prompt; the
holder renews it; it lapses after 15 min without renewal; takeover after lapse + 5 min quiet + 10 min cooldown; an
operator "pin" overrides; every change is a signed, audited record. Headless batch workers never claim. Peer mail is
surfaced only in the session that holds the lease; others leave it waiting.

Planned later: signed per-session/agent eligibility profiles ("may this session ever be main?"), default "no".

## Question: until profiles exist, who may become main?
- A. Any interactive session of the project; first to claim wins; the operator can pin to override. (As built.)
- B. Only sessions of the agent id named in the project's address book entry. Note: in practice all sessions of a
  project share one agent id; and if no agent is named, no session can become main.
- C. Only a session the operator pinned explicitly; without a pin there is no main and mail waits.

The project team recommends A. The operator leans A and notes that B (and C) can leave a project with no main at all,
which stops delivery entirely. Challenge or confirm.
