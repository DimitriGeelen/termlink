Seat: codex; model: GPT-6; date: 2026-11-12 UTC

1. Ranking A–D

A > D > C > B, for the options as written. Prefer E below for the eventual shared-hub solution.

A preserves the operating-system privilege boundary and enables autonomous reviews now. Separate discovery is an explicit limitation, not necessarily a failure: the design session can collect results on its own hub and report through its main-hub token. Configure worker endpoints explicitly, supervise the user hub, and test request/result delivery and restart recovery. Risks include stolen user credentials, malicious reviews accessing that user's files, resource exhaustion, and accidental routing to the wrong hub. Keep main-hub secrets out of this environment. Root services must not execute artifacts or instructions from this hub merely because they came from a familiar project; that would create a privilege-escalation bridge.

D is a temporary operational fallback, not a secure automation boundary. A root agent interpreting messages can become a confused deputy through malicious prompts, repository instructions, or generated commands. Availability, duplicate requests, and ambiguous completion also depend on that agent. It needs explicit restrictions and durable request tracking; project membership is insufficient authorization.

C gives structured admission, attribution, quotas, and predictable operation, but explicitly launches the worker as root. An allowed “review” can execute tests, hooks, plugins, or commands selected by attacker-controlled repository content. Fixed worker kinds and task labels do not constrain that execution. Other escalation paths include shell interpolation, writable launch scripts, inherited environment variables, symlink races, and caller-selected output paths. I would not approve C as written; dropping worker privileges changes the security assessment substantially.

B grants the largest continuous authority to the design session and everything it reads or launches. It may fix dispatch immediately, but neither contains compromises nor provides a narrower privilege boundary. Being able to use sudo does not make routinely running an AI agent as root equally safe.

2. Option E

Separate worker execution from registration authority. Run workers as the caller or a dedicated restricted project account. Let a small authenticated registrar create only validated, caller-owned session records on the main hub. Keep secrets and TLS keys root-only. The root hub can own registry writes without making workers root.

This requires a real connection and ownership design: validate endpoints, authenticate the worker, prevent registration over existing identities, and enforce ownership on updates and removal. Otherwise registration becomes session impersonation, arbitrary endpoint access, or a route for root to trust attacker-supplied metadata. Use leases and reconciliation for stale records. Verify bidirectional messaging, not just presence in a listing.

A group-writable directory is weaker: peers may forge or replace records, and a child directory remains inaccessible through a 0700 parent without additional access changes. Per-user federation can preserve isolation but introduces routing, identity, and delivery complexity. Socket activation improves lifecycle management; it supplies no authorization policy. A sudo/polkit wrapper is acceptable only with the same narrow validation and privilege drop; allowing a general dispatch command is not sufficient.

3. Admission, authentication, and failure behavior

Policy must bind each authenticated principal to projects, immutable launcher profiles, worker UID/GID, permitted inputs and output destinations, network/credential access, and concurrency, duration, CPU, memory, storage, and spending limits. Trusted binaries and configuration must be unwritable by callers. Never accept arbitrary shell strings, executable paths, environment variables, or root-run repository hooks.

Requests need a schema version, unique idempotency key, project/task identifier, profile identifier, immutable input reference, bounded parameters, deadline, and authorized result destination. Treat all prompts and repository content as untrusted data.

Locally, authenticate through kernel-provided Unix peer credentials. Remotely, use verified TLS and a separately issued, short-lived dispatch/register capability bound to a principal, audience, and project. Do not promote an interact token or trust a sender field in message text.

Log principal, request ID, policy version, admission decision, input revision, effective worker identity, session ID, lifecycle, exit status, and result reference; redact credentials. Persist accepted requests before spawning and reconcile crashes to prevent duplicate execution.

Reject unknown profiles, ownership mismatches, expired credentials, unsafe paths, missing isolation, exhausted quotas, unavailable admission auditing, and unverified registration. Never fall back to root or silently report success before registration and connectivity succeed.

4. Framing corrections

Registration location and execution privilege are separate problems. Main-hub visibility does not inherently require root workers.

The mismatch refusal is conditional and explicitly overridable, not an absolute prohibition (crates/termlink-cli/src/commands/session.rs:305). Its hub detection uses local pidfile status (same file:287), so it is not proof of TCP reachability. An override alone does not solve discovery.

A sessions subdirectory already exists (crates/termlink-session/src/discovery.rs:29); permissions and trust rules, rather than merely adding a directory, need redesign.

RANKING: A > D > C > B