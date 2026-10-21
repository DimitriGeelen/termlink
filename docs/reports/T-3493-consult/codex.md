## 1. Verdict

**Choose B as an interim repair, with C committed as the O-7 implementation.** B makes an existing restriction operationally usable; A preserves a choice between unrestricted injection and broken notifications. D conflicts with the newly ratified objective.

I would change B’s acceptance criteria: ship a safe credential-delivery mechanism and explicit documentation that it provides **bearer authorization without caller attribution**. Do not describe B as an inject ACL or O-7 compliance. Keeping C’s inception open is insufficient unless it has an owner and completion criteria.

## 2. Risks of the recommended option

- **More secret holders and copies.** Giving sidecars tokens expands the credential exposure surface. A stolen token grants whatever Control permits, potentially much more than delivering a notification.
- **Credential leakage.** `--token` risks disclosure through process arguments, shell history, diagnostics, and command logging. Environment variables can propagate to children and appear in debugging output. Prefer a protected file descriptor or tightly permissioned credential file; never log credentials.
- **Same-user compromise.** Processes sharing a Unix identity may be able to read credential material or interfere with terminals, depending on host protections. B is not isolation from arbitrary same-user code.
- **Operational failure.** Provisioning, rotation, and restart mismatches can silently disable doorbells. Authentication failures need visible health reporting, without falling back to unrestricted access.
- **False confidence.** A valid token does not identify its actual user. Even a label attached to that token cannot distinguish legitimate use from theft or sharing.
- **Keyboard authority.** Inject is not merely messaging: keystrokes can reach a shell, approval prompt, editor, or agent with broader permissions. A doorbell credential should ideally authorize only a fixed notification action.

## 3. What we are missing

The key distinction is **authority to notify versus authority to type arbitrary input**. A dedicated notification RPC, consumed as structured data rather than keyboard input, could remove the sidecar’s need for Control entirely.

Relevant prior-art lessons:

- **SSH agent forwarding:** withholding a private key does not prevent misuse of an exposed signing capability.
- **tmux/screen sockets:** filesystem permissions provide a useful local boundary, chiefly between Unix users; they do not establish individual agent identities within one user.
- **X11 access control:** broad access to an input interface creates authority far beyond the apparent application interaction.
- **Capability tokens:** narrow, short-lived, target-bound capabilities reduce exposure, but bearer possession is not identity.
- **Signed RPCs:** signatures need request binding, replay protection, and policy enforcement; signing arbitrary payload bytes alone is insufficient.

Unaddressed paths include replay, target substitution during hub routing, compromised supervisors, writable ACL configuration, lifecycle calls that replace session policy, and identity-key theft. An authorized agent can also relay another agent’s instructions: attribution identifies the immediate caller, not necessarily the originator of intent.

A separate inject socket with restrictive permissions is a useful local simplification, particularly with distinct service users. It does not solve network attribution or same-user impersonation.

For C:

**(a)** Define a versioned signed envelope covering method, immutable session identifier, destination, complete arguments, caller identity, and freshness information. The session verifies both signature and authorization and rejects replay. Preserve the signed envelope across forwarding.

**(b)** Make initial policy explicit in the authenticated creation request, bounded by operator-controlled delegation rules. The spawner cannot grant authority it lacks. Restrict policy updates and protect operator status from peer assignment.

**(c)** Install policy atomically before exposing the worker. Include the supervisor’s specific delegated rights and the notification identity from creation. Fail creation visibly if policy cannot be installed; never use a permissive bootstrap interval. Give operator attach a separately authenticated, audited path.

## 4. Order of work

First define C’s security invariants and B’s credential scope. Then ship B with protected credential delivery, explicit provisioning, rotation behavior, and tests confirming denied unauthorized injection and working authenticated doorbells.

Next implement signed inject and atomic policy provisioning, then extend authorization and attribution to lifecycle writes and other O-7 operations.

B must not establish tokens as identities, embed secrets in durable messages, or create a permanent unsigned bypass. Completion of O-7 requires retiring or explicitly constraining every legacy path that lets a peer exercise operator authority.