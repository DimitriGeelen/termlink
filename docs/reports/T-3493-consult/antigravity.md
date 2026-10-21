### 1. Verdict

**Accept B with modifications, but treat it purely as a bugfix—not a step toward O-7.**

Right now, `--token-secret` is broken by design: enabling it severs the sidecar doorbell and stalls mail delivery (Fact 4). Fixing `termlink inject` so the existing mechanism works end-to-end is sensible hygiene. However, I would change two things about B as proposed:

1. **Forbid `--token` as a CLI flag.** Only support passing the token via file descriptor, stdin, or environment variable (`TERMLINK_TOKEN_FILE` / `TERMLINK_TOKEN`). A `--token` flag immediately exposes the bearer credential in `ps` / `/proc/<pid>/cmdline` to every process on the host.
2. **Scope the sidecar token or replace the doorbell with an out-of-band signal.** A sidecar that only needs to ring a doorbell should not hold full `Control` scope (which permits arbitrary keystroke execution). Ideally, "wake up / check mail" is a distinct, narrow RPC (`session.notify` or a fixed newline/bell) rather than raw `command.inject` with full control.

Do **not** treat B as satisfying the peer project's request for an inject ACL (Fact 6) or O-7. A shared bearer token neither attributes actions to an identity nor prevents a peer holding the token from impersonating the operator.

---

### 2. Risks of the Recommended Option

- **Credential exposure via process table and environment:** If `termlink inject` accepts `--token <val>`, any local process can scrape it from `/proc`. Even environment variables leak into child processes and crash dumps if the sidecar or supervisor spawns sub-processes without sanitizing `environ`.
- **Privilege amplification of the sidecar:** The sidecar is a mail notifier. Handing it a `Control`-scope token makes the sidecar a high-value pivot: compromising or tricking the sidecar grants full interactive terminal takeover (`command.inject`).
- **Secret distribution sprawl:** Because B relies on a symmetric per-session secret (`--token-secret`), every legitimate caller (sidecar, supervisor, dashboard, operator) must be handed the same secret or a token derived from it. Without per-caller scoping, revocation requires restarting the session, and audit logs still show *what* token scope was used, not *who* typed the command.
- **False sense of compliance:** Shipping B reduces urgency for C while leaving Fact 2 intact—any peer with the hub HMAC secret can still reach unlocked sessions, and any peer given a session's token can inject without attribution, directly violating O-7 ("no peer can act as the operator").

---

### 3. What We Are Missing

**Prior art & lessons:**
- **tmux / screen & X11 (`xauth`):** Both rely on local socket permissions plus bearer cookies (`MIT-MAGIC-COOKIE-1`). Both notoriously fail at fine-grained attribution and suffer from "all-or-nothing" control once a cookie leaks. X11's `xhost` vs. `xauth` history shows why coarse bearer tokens quickly require capability separation.
- **SSH agent forwarding & principals (`AuthorizedPrincipalsFile`):** Rather than distributing shared secrets to target sessions, SSH signs a challenge or request envelope with an Ed25519 key, and the target checks the key fingerprint against an authorized list.
- **Macaroons / capability tokens:** Since sessions already use HMAC-derived tokens (`auth.token`, Fact 3), caveats (e.g., `op = inject_bell_only`, `caller_id = <fp>`, `exp = <ts>`) can be chained onto the HMAC token without inventing a new asymmetric handshake from scratch.

**Unconsidered attack paths:**
- **Prompt/command injection via terminal escape sequences:** `command.inject` pipes raw keystrokes. Even an "authorized" peer injecting seemingly benign text can include `\r`, Ctrl-C (`\x03`), or bracketed-paste escapes to break out of an agent prompt and execute shell commands.
- **Replay attacks:** If signed RPCs (Option C) lack a monotonic counter, timestamp, or session ID binding, a hub or peer observing a signed `command.inject` can replay those keystrokes later.

**Simpler alternatives:**
- **Dedicated doorbell RPC:** Decouple "ring doorbell" from `command.inject`. If the sidecar can wake the agent via a parameterless `session.ring` RPC (or by reading channel events directly), 90% of automated `inject` traffic vanishes.

**Answering (a)–(c) for Option C:**
- **(a) RPC signatures:** Wrap JSON-RPC requests in a standard signed envelope (or header metadata) containing `sender_pubkey`, `target_session_id`, `timestamp`, `nonce`, and an Ed25519 signature over canonicalized `(method, params, target, timestamp, nonce)`. You already do this for channel posts (Fact 5); reuse the exact signing key and canonicalization.
- **(b) Allowlist source:** The session loads an immutable base allowlist at startup (operator fingerprint + spawner fingerprint) from its startup config/flags, and only identities with `Admin`/`Operator` capability on that session can mutate the allowlist at runtime via signed RPC.
- **(c) Spawner bootstrap:** The supervisor passes `--allow-fingerprint <supervisor_fp>` (or writes it into the session's spawn descriptor) when exec'ing the worker session, alongside the operator's root fingerprint from host config. The supervisor is authorized from millisecond zero without sharing a symmetric secret.

---

### 4. Order of Work

1. **Immediate (Slice B-modified):** Add token support to `termlink inject` via `TERMLINK_TOKEN_FILE` (or env var, **no CLI flag**) so `--token-secret` doesn't break the sidecar. Optionally add a restricted `Bell`/`Notify` scope so the sidecar doesn't hold full `Control`.
2. **Next (Reuse Fact 5 for RPCs):** Generalize the Ed25519 channel-post signature into a standard JSON-RPC `auth` metadata envelope and log the verified caller fingerprint on every write/inject RPC (satisfying the *traceability* half of O-7).
3. **Then (Session ACLs — Option C):** Enforce per-session fingerprint allowlists populated at spawn time (`--allow-fingerprint`), separating `operator` from `peer` identities.

**What B must not foreclose:**
- Do not bake bare string tokens into the JSON-RPC `command.inject` method parameters; keep authentication in the connection handshake (`auth.token`) or RPC envelope headers so C can upgrade the credential from an HMAC token to an Ed25519 signature without breaking the `command.inject` schema.
- Do not make shared secrets the configuration interface for cross-project supervisors; peer projects should configure public fingerprints, not distribute session secrets.
