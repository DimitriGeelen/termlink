# T-3385: out-of-band approval confirmation (OTP-style), research artifact

**Task:** T-3385 (inception, owner: operator; high priority) · **Arc:** arc-011 · **Status:** exploration started 2026-10-10
**Ruling behind it:** OQ-14 D‴ (T-3351 Decisions, 2026-10-10); requirement R-76 in requirements v0.5.2.
**Partner:** ring20-manager (manages the Cloudron estate), in close cooperation, not delegated.

## 1 Question

How does a high-impact approval get an active, out-of-band confirmation from the operator, through a pluggable
provider that each framework user can configure, with ring20-manager building the facility for this estate?
One inception, one go/no-go: adopt a confirmation-provider contract plus a first engine, or not.

## 2 What is already ruled (do not re-decide here)

1. High-impact approvals never take effect on a timeout or on silence; unconfirmed within PN-15 (10 min,
   starting value) they expire visibly (R-76.a (3)).
2. The confirmation is bound to the approval's digest (executor and target included) and is single-use.
3. A pluggable confirmation-provider contract: ask (digest + human summary), verify once against that digest,
   configured per installation as a signed policy record (R-73/CR-19), changing the provider is high-impact,
   fail closed when the provider is unreachable, prefer remote verification so secrets stay off agent hosts
   (agents run as root today, RR-2).
4. Interim until a provider exists: explicit double approval at the terminal plus a notice to the operator's
   channel (shown as "no confirmation provider").
5. Vendor-agnostic: nothing in the contract may depend on one harness or one vendor channel (CR-21).
6. The operator approval key (GP-11, still open) and the confirmation are different things: the key signs what
   the operator approves; the confirmation proves a second, independent channel agreed.

## 3 What the contract must answer (draft, to be explored)

1. Ask: `confirm(request_id, digest, summary, risk_class, expires_at)` -> pending handle.
2. Verify: `verify(handle, proof)` -> ok | wrong | expired | used; once only; bound to the digest.
3. Health: `status()` -> configured / reachable / last success, for needs-attention and the fail-closed rule.
4. Configuration: provider kind, endpoint, the identity of the verifier, how its secret is held (never on agent
   hosts where avoidable), signed as a policy record.
5. Audit: every ask, verify and failure recorded, visible to the operator.

## 4 Candidate engines (to research; nothing chosen)

| Kind | How the operator confirms | Where the secret lives | Notes to check |
|---|---|---|---|
| TOTP (RFC 6238) authenticator app | types a 6-digit code shown on the phone | shared seed: phone + verifier | code is not bound to a digest by itself; binding needs the verifier to accept the code only for one pending request; phishing-prone |
| Push approval with the summary on the phone (e.g. a self-hosted push service on Cloudron) | taps Approve on a notification that shows the summary | verifier service | binding to the digest is natural; needs a trusted app or signed callback |
| WebAuthn / FIDO2 hardware key | touches the key on a page that shows the summary | private key never leaves the hardware | strongest binding (challenge = digest); needs a browser page, e.g. Watchtower |
| An identity provider already on the estate (if Cloudron runs one) | step-up login with its MFA | IdP | reuse vs coupling to one estate; must stay pluggable |

## 5 Questions for ring20-manager (first consult)

1. Which authentication / MFA / push components already run on the Cloudron estate, and which are planned?
2. Can a service there verify a confirmation remotely and return a signed result, so no secret sits on agent hosts?
3. What would they build, on what timescale, and what interface would suit them?
4. Constraints: availability (the confirmation fails closed, so its uptime matters), backups, who administers it.

## 6 Questions for the operator (later, one at a time)

1. Which device(s) do you want to confirm on (phone app, hardware key, browser)?
2. Is a step-up through an existing estate login acceptable, or must it be independent of the estate?
3. Default for installations without any provider: the interim double approval (ruled) or something stronger?

## 7 Exploration plan

1. Consult ring20-manager (section 5); record the answer here unedited.
2. Survey the candidate engines against the contract (section 3) and the ruled constraints (section 2).
3. Draft the contract with a reference adapter shape; one spike only if a binding question cannot be settled on
   paper.
4. Present the go/no-go to the operator with a recommendation.

## Dialogue log

1. 2026-10-10: operator ruled OQ-14 D‴: high-impact approvals must not continue automatically ("I might not be
   watching the channel"); OTP-style approval "absolutely want to put in place", separate inception, research
   needed, working with ring20-manager who manages the Cloudron estate. Then: high priority; not completely
   delegated to ring20-manager but close cooperation; ring20-manager builds the facility, TermLink caters for it
   with a way to configure which OTP engine and how to connect, "because different users of the framework might
   have different OTP engines".
