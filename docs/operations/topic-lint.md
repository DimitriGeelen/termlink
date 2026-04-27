# Topic↔role soft-lint (T-1300 / T-1301)

> **Quick start:** copy `docs/operations/examples/topic_roles.yaml` and
> `docs/operations/examples/relay_declarations.yaml` into your hub's
> `<runtime_dir>/` (default `/var/lib/termlink/`), edit, then send the hub
> a `SIGHUP` to load. See § Hot-reload below.


The hub runs a **soft-lint** at every `event.broadcast` and `event.emit_to`
that compares the originating session's role(s) against a per-prefix policy.
Violations dual-write a `routing.lint.warning` envelope to bus topic
`routing:lint`. **The originating emit always succeeds regardless** — lint
is observability, not enforcement.

## Where rules live

`<runtime_dir>/topic_roles.yaml` (typically `/var/lib/termlink/topic_roles.yaml`).
If the file is absent, the hub installs the built-in defaults from
T-1297 § Spike 3 (10 prefix rules + 4 exempt categories).

## Schema

```yaml
rules:
  - prefix: "framework"
    roles: [framework, pickup]

  - prefix: "infra"
    roles: [ring20-management, infrastructure]

  - prefix: "oauth"
    # Caller's role-list must contain a role equal to the prefix
    # (used for product-named prefixes like `oauth`, `email-archive`).
    roles_from_originator_role: true

exempt_prefixes:
  - "agent."
  - "session."
  - "worker."
  - "test."
  - "help."
  - "channel.delivery"
```

Match semantics:
- A topic matches a `prefix` rule when it equals the prefix or starts with
  `prefix.` / `prefix:`. `framework` matches `framework:pickup` and
  `framework.gap` but not `frameworkz`.
- The most-specific (longest) matching rule wins.
- `exempt_prefixes` are checked first — exempt topics never warn.
- When `roles_from_originator_role: true`, the rule's `prefix` becomes the
  required role.
- A caller with no `from` field skips lint entirely (debug log).

## Hot-reload

Send `SIGHUP` to the hub process; the watcher re-reads the file and swaps
the rule set atomically. Parse failures keep the previous rules in place
and log at warn level.

```sh
pkill -HUP -f 'termlink-hub'
# or, if you have the pid:
kill -HUP $(cat $TERMLINK_RUNTIME_DIR/hub.pid)
```

## Reading warnings

Subscribers can pull from `routing:lint` via the channel surface, e.g.:

```sh
termlink event subscribe --topic routing:lint --timeout-ms 5000
```

Payload shape:

```json
{
  "type": "routing.lint.warning",
  "method": "event.broadcast",
  "topic": "framework:pickup",
  "from": "session-abc",
  "rule_prefix": "framework",
  "expected_roles": ["framework", "pickup"],
  "actual_roles": ["product"]
}
```

## Per-session opt-in (T-1301 — `relay_declarations.yaml`)

Multi-purpose sessions like `framework-agent` legitimately emit topics
that fall under several roles (governance + cross-project relay). Without
an opt-in, every legitimate emit would generate a warning. The
**relay_declarations** file lets operators whitelist specific prefixes
per session.

`<runtime_dir>/relay_declarations.yaml`:

```yaml
sessions:
  - name: "framework-agent"
    relay_for:
      - "channel.delivery"
      - "task.complete"
      - "learning"          # prefix-match: covers learning.x and learning:y

  - name: "ring20-management"
    relay_for:
      - "infra"
      - "outage"
```

Match semantics: same boundary rules as the `Rules` engine — `learning`
matches `learning.captured` and `learning:foo` but not `learnings`. When
the caller's declared `relay_for` covers the topic, lint returns
suppressed (debug-logged, no envelope written to `routing:lint`).

The file is hot-reloaded by the same SIGHUP signal that reloads
`topic_roles.yaml`. Both reloads are independent — a parse failure on
one keeps the other's state in place.

Sessions are looked up by **display_name** (the stable name visible in
`termlink list-sessions`), not by session_id (which rotates per
registration).

## Disabling

Out of scope for T-1300 v1. Workarounds:
- Edit `topic_roles.yaml` to remove rules you do not want enforced.
- Add the offending prefix to `exempt_prefixes`.
- Configure subscribers to ignore `routing:lint` (warnings have zero blast
  radius beyond the bus topic).

A global on/off switch is a candidate for follow-up if the warning volume
ever needs throttling — current expectation is <50 entries / hour at
steady state.
