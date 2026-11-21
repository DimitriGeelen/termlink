You are an independent design reviewer. You have no tools and no other context; judge only from the bundle.

The bundle describes a conflict between two checks in one system. A daily canary (detects agents posting files as
chat messages) reads every channel topic, including agents' private inboxes, without signing. Rule P-12 says an inbox
has exactly one reader, its own signed sidecar, and a deploy-time check fails while any second reader was seen in the
last 24 h. The canary is that second reader, every day. The operator must choose how to resolve it. The agent proposed
four options (A-D) and recommends B. Do not anchor on the agent's recommendation or scores; test them.

Report, concretely and briefly:
1. Is the diagnosis right? Name anything in the bundle that contradicts "the canary is the recurring unnamed reader",
   or any other plausible recurring unsigned reader of `inbox:*` topics the evidence does not rule out (for example
   the first-seen time 2026-11-20T05:52:37Z, which is not at HH:47). Say what one check would settle it.
2. For each option A-D: its strongest real weakness (a concrete sequence that goes wrong), and whether it respects
   the rule "nothing working is switched off without a proven replacement and explicit operator approval".
3. Options the agent missed (for example: the canary reads inboxes as a signed, named reader and P-12 learns to
   classify an audit reader; the sidecar takes over file-shape detection for its own inbox; the canary reads only
   topic metadata, not content, for inboxes). For each: what it costs, and whether it actually lets action 37 pass.
4. For option B specifically: races and failure modes (binding appears or disappears between the consumers call and
   the read; the consumers call errors or times out; an inbox on another hub; the canary's exit code when it skips),
   and what tests would pin it.
5. One paragraph: which option you would choose and why, and the steelman of the strongest alternative in two
   sentences. Severity-tag each weakness high/medium/low. No credit for intent.
