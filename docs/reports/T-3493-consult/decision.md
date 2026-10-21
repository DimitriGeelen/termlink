# T-3493 — inject access: decision record

## Dialogue Log (2026-10-21)

1. Agent presented decision 3 as a brief: options A defer, B token on the inject path (C shaped next), C full
   signed-inject design now, D no-go; scored on the joint value drivers (A −20, B +20, C +26, D −31); recommended B
   because C's design questions (signed RPCs, allowlist source, supervisor bootstrap) were open. Corrected its own
   earlier DEFER note: written before O-7 existed.
2. Operator: "elaborate". Agent explained in plain terms: inject is sitting at the agent's keyboard; any hub-secret
   holder can reach an open session; the existing lock (`--token-secret`) breaks the sidecar doorbell because
   `termlink inject` cannot present a token; B fixes that, C is the real O-7 fix.
3. Operator: "Please get external review." Brief written (`brief.md`, no hosts/IPs/secrets) and sent to subscription
   reviewers: codex (`codex.md`), Antigravity (`antigravity.md`); opencode failed twice with a provider server error.
4. Both reviewers chose B as interim only, not O-7, and independently improved it: no CLI token flag (process list
   leak), a narrow notify scope so the sidecar never holds keyboard rights, keep authentication out of the call's
   parameters so C can swap token for signature. They differed on environment variables (codex stricter) and added
   risks (escape-sequence injection; token rotation silently breaking the doorbell). Both gave matching answers for C's
   open questions (signed envelope with nonce reusing the Ed25519 identity key; allowlist fixed at session creation;
   supervisor fingerprint present from the first moment).
5. Agent revised the recommendation to B′ (B plus those modifications) and kept the decision open.
6. Operator: "as recommended" — ruled **B′**. Recorded in the task's Decisions; IW-1 answered.

## What the ruling authorises

1. A build task for B′ (token from a protected file or fd; narrow notify scope/RPC for the doorbell; visible doorbell
   health failure; documented as interim, not O-7; auth kept out of `command.inject` params).
2. A separate plan for C with an owner and finish criteria, to be reviewed by AEF and ring20.
3. Telling ring20 they can lock their sessions once B′ ships.

## Left open

1. C's design (IW-2..IW-4), returned to the operator as its own decision.
2. Whether new sessions default to `--token-secret`.
