# Herdr adoption — distribution & onboarding (2026-08-15)

**Scope:** what makes herdr's onboarding frictionless, and what is adoptable without
giving up the guarantees TermLink's complexity buys.

**Licence:** every recommendation below is **category (a) — idea only**. No herdr code
is proposed for copying. No attribution/NOTICE handling is triggered by anything here.
If any of these is later implemented by lifting herdr source, that becomes category (b)
and needs re-review.

**Evidence tags:** [VERIFIED-PAGE] = read from a live project-owned page (unpinned,
summarised through WebFetch — quotes reliable, enumerations carry transcription risk).
TermLink claims are `file:line` from files I opened in this worktree.

---

## Tooling note

No hook blocked me. Grep, Read and WebFetch all ran. Two greps returned zero and I
re-queried both rather than reporting the zero:

- `auto_start|ensure_hub|spawn_hub` → 0 matches. Re-checked via the connection-failure
  path; `cli_integration.rs:1583,2196,4187` assert commands **fail** when no hub is
  running. The absence is real: **nothing auto-starts a hub.**
- I assumed there was no `termlink init`. **Wrong** — `cli.rs:1786` has `Init`. Reading
  it (`cli.rs:1784-1794`) shows it is `identity init` under `IdentityAction`, scoped to
  an ed25519 keypair. There is no top-level onboarding command. The corrected claim is
  narrower than the one I nearly wrote.

---

## What herdr actually does

| Mechanism | Evidence |
|---|---|
| Single binary, user-level install, **no sudo** — `$HOME/.local/bin`, overridable via `HERDR_INSTALL_DIR`; detects os/arch from `uname -s`/`uname -m` | install.sh [VERIFIED-PAGE] |
| Installer writes **no config file at all** — downloads and places the binary only; warns if the dir is not on PATH but **does not edit shell rc** | install.sh [VERIFIED-PAGE] |
| Config is **optional**: *"Herdr works without a config file. Add one when you want custom keys, themes, sidebar layouts, notifications, or advanced behavior."* | docs/configuration [VERIFIED-PAGE] |
| Invalid config **degrades, not refuses**: *"If a config value is invalid, Herdr falls back to a safe default and shows a startup warning."* | docs/configuration [VERIFIED-PAGE] |
| Config generated **on demand**, never at install: `herdr --default-config` → save to `~/.config/herdr/config.toml` | docs/configuration [VERIFIED-PAGE] |
| Agent detection ships **bundled**; overrides are per-agent files with clear precedence: *"Local overrides can replace a remote or bundled manifest… `~/.config/herdr/agent-detection/<agent>.toml`"*, *"Local overrides always win"* | docs/agents [VERIFIED-PAGE] |
| First run is literally `herdr`; reconnect is *"running `herdr` again"* | README [VERIFIED-PAGE] |
| State lives under `~/.config/herdr/` — socket at `~/.config/herdr/herdr.sock`, named sessions under `sessions/<name>/` | docs/socket-api [VERIFIED-PAGE] |

**One claim I am NOT relying on.** The getting-started fetch mentioned an
`auto_detect_launch()` that "connects to an existing running server or spawns a new
one". The socket-api page, fetched directly, **does not document auto-spawn** — it
explicitly left "Automatic server launch if not running" unaddressed. So the
connect-or-spawn behaviour is *suggested by two sources and confirmed by neither*.
Recommendation D1 below is written to stand on the README's observable
`herdr` → detach → `herdr` loop, not on that function name.

**Also: herdr's install is easy partly because it defends nothing.** No secret, no TLS,
no pinning, no multi-host identity. The socket-api page describes **no auth, token, or
pinning of any kind** [VERIFIED-PAGE]. Most of TermLink's install cost buys something
herdr does not offer. The findings below are the subset where we pay cost and get
*nothing* in return.

---

## Finding 1 — Our default runtime_dir IS the PL-021 footgun (highest value)

`crates/termlink-session/src/discovery.rs:10-26` resolves, in order:
`$TERMLINK_RUNTIME_DIR` → `$XDG_RUNTIME_DIR/termlink` → `$TMPDIR/termlink-$UID` →
`/tmp/termlink-$UID` (line 25).

**Three of the four candidates are volatile.** `/tmp` is the documented PL-021 class;
`$XDG_RUNTIME_DIR` is `/run/user/$UID`, a tmpfs destroyed at logout. Only the explicit
env var is durable. The single largest production failure mode in this repo is
**what happens when the operator configures nothing** — the opposite of a happy path.

Herdr's equivalent state root is `~/.config/herdr/` [VERIFIED-PAGE] — persistent by
construction, so the entire volatility class cannot occur regardless of configuration.

**Adopt:** default to a persistent per-user path (`~/.local/state/termlink` or
`~/.termlink/runtime`) and demote `/tmp` to last-resort-with-warning. This changes a
default, not a guarantee: secret + TLS pinning are untouched, and every existing
`TERMLINK_RUNTIME_DIR` deployment is unaffected because the env var still wins at
line 11.
**Effort:** ~1 day for the change; the real cost is a migration path for hubs already
holding a live `hub.secret` under the old default, plus updating preflight Check 1.
**Directive:** D1 Antifragility (removes the failure class rather than detecting it),
D3 Usability.

## Finding 2 — The hub's volatile-default warning fires for root only

`crates/termlink-hub/src/server.rs:52-75` warns on the volatile default, but returns
early unless `uid == 0` (lines 57-59). The comment at lines 40-41 justifies this:
non-root `/tmp/termlink-UID` is "the documented default for interactive sessions and
not a footgun."

That reasoning does not survive Finding 1. A non-root hub on `/tmp` regenerates its
secret and cert on the same reboot a root one does; PL-021's consequence is identical.
Meanwhile `substrate-preflight.sh:242-282` FAILs on `/tmp` **regardless of uid** — so
the two guards disagree about whether the same state is dangerous.

**Adopt:** align the hub warning with preflight (warn for any uid on a volatile
default). **Effort:** ~1 hour — one early-return removed, the truth-table tests at
`server.rs:52` already inject uid so they extend cleanly. **Directive:** D2 Reliability.

## Finding 3 — Preflight models the default wrongly

`scripts/substrate-preflight.sh:240` reads:

```bash
local rd="${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}"
```

This hardcodes one fallback and **does not replicate `discovery.rs`'s resolution
order** — it ignores `$XDG_RUNTIME_DIR`, `$TMPDIR`, and the real `$UID`. Consequences
on a normal systemd user session (XDG set, env var unset): the binary uses
`/run/user/1000/termlink`, preflight reports on `/tmp/termlink-0`, and the two are
unrelated paths. Preflight's verdict is then about a directory nothing uses — and
because `/run/user/...` matches neither `/tmp*` nor `/var/tmp*` in the case statement
at line 242, the genuinely-volatile XDG path would report **PASS: "not on /tmp —
persists across reboot"** if it were resolved at all.

This is our own guard-reporting-green class, in the check that exists to prevent our
worst failure mode.
**Adopt:** have preflight ask the binary (`termlink info --json` exposes `runtime_dir`
— `cli_integration.rs:967` asserts the field) instead of re-deriving it, and add
`/run/user/*` to the volatile set. **Effort:** ~2 hours. **Directive:** D2 Reliability.

## Finding 4 — Defer hubs.toml; it is not needed on the local happy path

`config.rs:112-128` treats a missing `hubs.toml` as a normal empty config, and the
local hub is reached at `runtime_dir()/hub.sock` (`server.rs:25`) with no profile
involved. **A purely-local install never needs the file.** Yet preflight Check 2
(`substrate-preflight.sh:287-304`) WARNs whenever it is absent — so a correctly
configured single-host operator is told something is wrong, on first run, when nothing
is.

Herdr's rule is the one to take: config exists for the multi-host/customised case, and
its absence is silence, not a warning [VERIFIED-PAGE].
**Adopt:** make Check 2 conditional — WARN only if a fleet verb has been used or more
than one hub is expected; otherwise PASS with "local-only install, no fleet config
needed". **Effort:** ~2 hours. **Directive:** D3 Usability (this is alert-fatigue
prevention, the PL-219 class the repo already names).

## Finding 5 — No install path that does not require a Rust toolchain

Preflight's own remediation (`substrate-preflight.sh:407,415,428`) is
`cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/`. That
is a **from-source** install as the documented path. `release.yml` already publishes
macOS + Linux binaries (CLAUDE.md § CI/Release Flow) and a Homebrew formula exists — so
the artefacts are built but the operator-facing instructions do not use them.

**Adopt:** point remediation strings and getting-started at the released binary, and
add a `curl | sh` installer that resolves os/arch and drops into `~/.local/bin` with no
sudo (herdr's shape, independently implemented — idea only).
**Effort:** ~1 day for the installer, ~1 hour for the remediation strings (which is
most of the benefit, and independent).
**Directive:** D4 Portability, D3 Usability.

## Finding 6 — Bundled-with-local-override as a config pattern

Herdr ships detection manifests bundled, lets a user drop one file in a known directory,
and states the precedence in one line: *"Local overrides always win"* [VERIFIED-PAGE].
The user edits one small file, never a monolith.

TermLink's nearest analogue is the guard allowlists, which T-2681 **already moved to
this shape** (`.context/checks/<name>-allowlist`, tracked, with an env/flag override
winning over both). The pattern is validated here; it is `hubs.toml` — one file
carrying every profile — that has not adopted it.
**Adopt:** allow `~/.termlink/hubs.d/<name>.toml` alongside `hubs.toml`, merged with a
stated precedence. **Effort:** ~1 day. **Directive:** D3 Usability, D4 Portability.
**Caveat:** this is the weakest item — it is ergonomics for an operator who already got
through onboarding, so it does not serve the question asked. Rank it last.

---

## What is NOT adoptable

- **Zero-auth connection.** Herdr's socket has no auth, token, or pinning
  [VERIFIED-PAGE]. Our secret + TOFU pinning is the cross-host trust model; dropping it
  to shorten install would trade a guarantee for convenience.
- **Screen-scraped agent detection.** Already settled in the synthesis (§4): herdr's
  agent state is heuristic pattern-matching against a rendered screen, and its own docs
  concede misclassification. Our `agent-presence` heartbeat is a protocol. Not a
  distribution question, and not an upgrade.
- **Auto-spawning a hub on first client call.** Attractive, and the closest thing to
  herdr's `herdr`-just-works loop — but a hub silently self-starting under the wrong
  `runtime_dir` is precisely how PL-021 propagates. **Finding 1 is the prerequisite:**
  fix the default first, and only then consider auto-start. Sequenced, not rejected.

## Suggested order

1. **Finding 3** (~2h) — a guard reporting on the wrong directory is the most dangerous
   item, and it is cheap.
2. **Finding 2** (~1h) + **Finding 4** (~2h) — small, self-contained.
3. **Finding 5** remediation strings (~1h) — high visibility, no new machinery.
4. **Finding 1** (~1 day + migration) — the real prize; needs a decision on the
   migration path for live hubs, so it is human-owned.
5. Findings 5-installer, 6 — optional.

Findings 1 and 4 change defaults that existing deployments rely on. Both are
**inception-shaped**, not build-shaped: they want a human go/no-go before code.
