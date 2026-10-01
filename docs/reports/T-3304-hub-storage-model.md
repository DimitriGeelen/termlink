# T-3304 — Should the TermLink hub be a queryable source of truth?

Status: inception, consulting three non-Anthropic agents (operator request, 2026-10-01).
Raised while discussing SQ-11 (T-2573, subscribe deadline drops collected messages).

## The operator's question (paraphrased faithfully)

Does the hub store messages? Is there retention, is there pruning — or do agents
keep their own old messages and just subscribe to topics? Agents can choose that
for themselves. But do we want the hub to be a **source of truth that can be
queried**? If yes: how do we facilitate it, do we need pruning mechanisms, how do
we chunk, how do subscriptions work, how do topics work, how does an agent select
or discover topics, and how do selections work (time windows, other cuts) and at
what cost?

## Brief sent to each agent

`T-3304-consult/brief.md` (identical text to all three).

## Answers

- `T-3304-consult/codex.md` — OpenAI Codex (ChatGPT subscription)
- `T-3304-consult/glm.md` — Z.AI GLM-5.3 via opencode (Coding Plan subscription)
- `T-3304-consult/qwen.md` — local qwen3:14b via Ollama

## Synthesis

(pending answers)

## Dialogue Log

- 2026-10-01 — Operator, during SQ-11: "do we want the hub to have a source of
  truth that can be queryable? … ask three of our … non-Anthropic agents … there's
  real value in there." SQ-11 left open pending this.
