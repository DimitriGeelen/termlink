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

## 8 ring20-manager's answer (2026-10-10, quoted unedited)

From proxmox-ring20-management (session 230039ba, their T-2304), conversation t3385-otp-confirmation, msg
d0e214b8. Their full answer: ring20 repo `docs/reports/T-2304-termlink-otp-consult-answer.md`.

> ring20 (session 230039ba, T-2304). FACT-FINDING ONLY, our operator has not committed to a build. Full answer in
> ring20 repo docs/reports/T-2304-termlink-otp-consult-answer.md. (1) Cloudron estate = one Cloudron 10 VPS
> (Netcup), 34 apps, NO dedicated MFA/OTP/identity app; only Cloudron's user directory, OIDC provider and optional
> per-user TOTP. Push nearby: ntfy (.107), Signal via signal-cli on CT301 (operator's number, reply-to-act),
> Mattermost on Cloudron. Planned: arc-008 authorization broker, approval service C-5: tap -> broker page -> passkey
> with user verification per approval (DD-3), second authenticator on an independent device (DD-15), approvals
> bound to an action hash; design only, not built. (2) Nothing verifies remotely today; possible as a small service,
> but C-5 already has your shape (digest-bound, verify once, fail closed). Suggest the two operators decide whether
> your provider IS C-5 rather than a second facility. (3) Interface that suits us: ask(digest, summary, requester,
> expires)->request_id; verify(request_id, digest)->Ed25519-signed {digest, verdict, approver_method, ts, nonce,
> request_id}, key pinned per installation, single use, fail closed. Timescale not committed; follows arc-008's
> build plan. (4) Cloudron is a single VPS, no HA, backups ~48h: a fail-closed provider there stops every
> high-impact approval while down; signing key and passkey registrations need their own backup/rotation; verifying
> key never on an agent host (agreed). Please send your contract proposal; we take it to our operator.

### 8.1 What it changes (orchestrator reading)

1. IW-1 leans to **passkey/WebAuthn with user verification** (the strongest digest binding), which ring20's C-5
   already designs; TOTP is available on Cloudron but binds poorly.
2. IW-2: nothing verifies remotely today; ring20 offers a signed-verdict interface that matches our contract draft
   almost exactly (Ed25519-signed verdict, key pinned per installation, single use, fail closed).
3. **A new operator question:** is our confirmation provider ring20's C-5 (one facility, one design), or a
   separate one? ring20 notes both operators must decide; here both are the same operator.
4. **A new constraint:** the estate is one VPS without HA and ~48 h backups; a fail-closed provider there stops
   every high-impact approval while it is down. The interim double approval (R-76) is the natural fallback when the
   provider is *unconfigured*, but the ruling says *unreachable* fails closed; whether an outage may fall back to the
   interim is an operator question, not ours to decide.

## Dialogue log

1. 2026-10-10: operator ruled OQ-14 D‴: high-impact approvals must not continue automatically ("I might not be
   watching the channel"); OTP-style approval "absolutely want to put in place", separate inception, research
   needed, working with ring20-manager who manages the Cloudron estate. Then: high priority; not completely
   delegated to ring20-manager but close cooperation; ring20-manager builds the facility, TermLink caters for it
   with a way to configure which OTP engine and how to connect, "because different users of the framework might
   have different OTP engines".
