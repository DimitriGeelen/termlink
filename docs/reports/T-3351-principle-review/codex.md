## 1. Sound direction, inconsistent access rules

1a. **Adopt with changes.** Encrypting content while leaving routing metadata visible is a sound direction. It would stop ordinary topic-token holders and hub-database readers from reading content, provided they cannot obtain endpoint keys or substitute trusted recipient keys.

1b. P1 and P3 conflict. P1 grants the project access to every agent’s mail; P3 encrypts agent-addressed mail only to that agent. Implementing P1 requires an additional project decryption path. Conversely, agent-only encryption means the project cannot necessarily read or recover that mail.

1c. Delivery fallback does not imply decryption fallback. A session or project can accept custody of agent-encrypted ciphertext without understanding it. If another agent must *act on* the message, that requires prior authorization and a suitable decryption path. P3 also omits session-addressed mail and leaves unclear how canonical identities, runtime instances, and key lifetimes relate.

1d. “Mailman” describes normal hub operation, not an absolute security boundary. A hub can observe metadata, withhold or replay traffic, and substitute unverified first-contact keys. An operator controlling an endpoint can obtain plaintext there. Off-host escrow does not prevent that.

## 2. Project access is a policy choice

2a. A project-wide reading entitlement makes the project the confidentiality boundary. Agent identity still supports attribution and delivery, but agent-addressed mail is not private from the project’s authorized readers. If every project agent holds the project key, compromising one exposes everything encrypted to that key.

2b. Prefer **explicit reader sets**, with the project as a default collaboration group. Distinguish agent-private mail from project-shared mail. Session groups can be added when useful; five addressing levels need not produce five cryptographic hierarchies.

2c. If universal project access is intentional, state it plainly: “Agent addressing selects the worker; project membership grants reading access.” Define whether new members inherit history and whether departing members retain access to previously received content. Previously learned plaintext cannot be revoked.

## 3. Smallest workable encryption scheme

3a. Use established authenticated encryption and envelope encryption: generate a content key per message, encrypt the payload once, and wrap that key for each authorized recipient and, if enabled, recovery. Keep signing and encryption keys separate. Bind relevant immutable envelope fields to the authenticated message; mutable delivery-stage metadata needs separate treatment.

3b. For a small lab, individual recipient key wraps avoid distributing one shared project private key. Project-shared messages target a recorded membership snapshot. This costs more header space but simplifies exclusion of departing members from future mail. A shared project key remains possible, with greater rotation and compromise costs.

3c. Bootstrap with manually approved, pinned identity-to-key bindings through an independent trusted channel. Hub-served signed cards alone do not establish first-contact trust. Record key identifiers and versions. Accept routine replacement through an authenticated transition; compromised-key replacement needs independent approval.

3d. Rotate keys for future messages and retain old decryption keys securely for the promised recovery period. Membership removal affects future recipient sets; addition grants future access by default. Historical access requires an explicit grant.

3e. An encrypted hub copy recovers a lost store only if decryption keys survive elsewhere. Define retention, replay authentication, deduplication, and how missing messages are detected. If STORED promises recoverability after receiver-store loss, acknowledge that promise only after the required ciphertext copy is durably committed.

3f. First settle reader policy and fallback semantics, then establish OS isolation and trusted key enrollment. Prototype encrypted send, durable storage, and restore before adding sophisticated group management.

## 4. Break-glass recovery

4a. An off-host recovery key is reasonable if operator recovery is an explicit confidentiality exception. Messages need recovery key wraps created in advance; possessing a recovery key alone cannot decrypt arbitrary ciphertext.

4b. Approval must control actual key use outside agent-controlled hosts. The tool should display the project, messages, purpose, and destination before approval. Prefer project-scoped recovery keys or constrained recovery tooling over unrestricted fleet-wide decryption.

4c. Log authorization and the attempted operation durably outside the affected host, followed by its outcome. Logs provide accountability, not proof that plaintext was never copied. “Single use” can describe an authorization or short-lived recovery capability; it cannot make disclosed plaintext or an exported private key single-use. Keep the private recovery key off recovered hosts.

## 5. Costs and compatibility

5a. Hub search, content digests, semantic telemetry, and content-based canaries move to authorized endpoints or a deliberately trusted reader. Metadata-based routing, timeout detection, and synthetic transport canaries remain possible. End-to-end canaries need recipient confirmation of successful decryption.

5b. Debugging becomes more dependent on message identifiers, authenticated receipts, error codes, and deliberate plaintext disclosure. Envelope fields and logs still need minimization: identifiers can themselves reveal sensitive information.

5c. Encryption complements mandatory signatures and circuit authentication. Urgent-mail downgrade can operate on authenticated sender identity and visible priority. Crash survival remains a storage requirement; encryption supplies neither durability nor loss detection. Project-contained fallback remains compatible as custody transfer, but requires extra authorization for substitute readers.

## 6. Recommendation

6a. **Adopt with changes:** separate routing from reading authority, resolve P1/P3, specify fallback decryption, and define recovery durability and key retention. The main benefit is removing routine plaintext access from hubs and token holders; the main unresolved choice is whether confidentiality belongs to each agent or to its entire project.
