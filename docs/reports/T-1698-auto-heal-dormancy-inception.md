---
task: T-1698
phase: inception
created: 2026-05-18T18:55:00Z
status: open
---

# T-1698: Auto-heal infrastructure dormant — inception

## Problem

`termlink fleet bootstrap-check --all` on .107 (2026-05-18T18:50Z) reports
`no-anchor` for every profile in `~/.termlink/hubs.toml`:

```
PROFILE                  ADDRESS                      ANCHOR  STATUS
laptop-141               192.168.10.141:9100          —       no-anchor
local-test               127.0.0.1:9100               —       no-anchor
ring20-dashboard         192.168.10.121:9100          —       no-anchor
ring20-management        192.168.10.122:9100          —       no-anchor
workstation-107-public   192.168.10.107:9100          —       no-anchor
Fleet verdict: no-anchor
```

The ten-task auto-heal stack built in late April / mid-May
(T-1666 `--include-pin-check`, T-1667 `--watch`, T-1669 `--notify`,
T-1680 `--auto-heal`, T-1681 secret-rotation gate, T-1682 watch-parser
bridge, T-1683 one-shot heal, T-1684 `--dry-run`, T-1685 heal.log,
T-1686 history-with-heals, T-1687 MCP parity for history, T-1688
bootstrap-check, T-1689 MCP parity for bootstrap-check) is **operationally
dormant**: when a hub rotates its secret or cert, no profile has a
declared anchor for the heal to fall back to. The heal subroutine
gates on declared `bootstrap_from` and skips with a stderr hint if
absent — so every hub is "skip-no-anchor" today.

This pattern recapitulates **PL-159** (config-driven mechanism shipped
without operator declaration step → dormant tooling) and **PL-168**
(canary scripts without trigger → dormant). The implementation
milestone landed but the operational milestone never did.

## Reachability

### Reachability matrix

| Profile | Address | SSH from .107 | termlink remote exec | Sessions ready |
|---|---|---|---|---|
| ring20-dashboard | 192.168.10.121:9100 | denied (no key) | works (tl-qe2pao72) | 1 |
| ring20-management | 192.168.10.122:9100 | denied (no key) | works (tl-vep3zjz4) | 1 |
| laptop-141 | 192.168.10.141:9100 | not tested (host DOWN) | unreachable | 0 |
| workstation-107-public | 192.168.10.107:9100 | localhost | works | 10 |
| local-test | 127.0.0.1:9100 | localhost | works | 10 |

Test commands:
```
timeout 5 ssh -o BatchMode=yes -o StrictHostKeyChecking=no \
  -o ConnectTimeout=3 root@192.168.10.122 ...
# → "root@192.168.10.122: Permission denied (publickey,password)."
```

```
termlink remote list ring20-dashboard
# → 1 session(s) on 192.168.10.121:9100  tl-qe2pao72  ready
```

Operator note: ssh keys to field hubs do not exist today.
`termlink remote exec` works because each hub's secret was bootstrapped
into `~/.termlink/secrets/<hub>.hex` via some prior heal/setup step.

## Path analysis (against R2 — out-of-band anchor rule)

R2 verbatim from CLAUDE.md: *"The `--bootstrap-from` source MUST NOT itself
depend on the termlink auth being healed (chicken-and-egg)."*

### Path 1 — ssh: anchors (operator wires SSH keys)

```toml
[hubs.ring20-dashboard]
bootstrap_from = "ssh:192.168.10.121"
```

`fleet reauth ... --bootstrap-from auto` then runs:
`ssh root@192.168.10.121 'cat /var/lib/termlink/hub.secret'`.

R2-clean by construction: SSH auth ≠ termlink auth, so a termlink rotation
doesn't kill the SSH path.

**Blocker:** SSH keys don't exist .107 → .121/.122/.141 today. Wiring them
is operator-bound (Ed25519 keygen + `authorized_keys` on each hub,
key-passphrase management, possibly `agent-forward` policy review).

### Path 2 — file: anchors fed by a warm-cache cron (REJECTED on analysis)

Initial appeal: cron periodically reads the live hub secret via
`termlink remote exec <hub> '<session>' 'termlink hub export-secret'`
and writes it to `~/.termlink/secrets/<hub>.hex.warm`. Profile points
`bootstrap_from = "file:~/.termlink/secrets/<hub>.hex.warm"`.

**Why this fails R2:** the fetching channel (`termlink remote exec`)
uses the same hub secret as the auth being healed. When the hub rotates,
the live session dies — the warm cache stops updating *at exactly the
moment the cache value is needed*. The cache always contains the OLD
secret post-rotation. The OLD secret is precisely what's invalid.

This isn't a degenerate failure mode — it's the fundamental shape of
rotation. The anchor must be **truly independent** of the credential
being rotated. Warm-cache via the same credential is a chicken-and-egg
violation in disguise.

### Path 3 — new anchor type `remote-exec:<hub>` shipped by termlink

Same R2 failure as Path 2: any termlink-native channel is auth-bound to
the secret being rotated. Even if the implementation is termlink's
own code instead of an external cron, the substrate is identical.

There's a sub-variant: an auxiliary side-channel (TCP port 9101?) with
**separate auth state** — a "heal-channel" credential that rotates on a
different schedule. This is real design work (D-tier), introduces a
new secret to manage, and shifts but doesn't eliminate the problem
(the heal-channel can itself rotate and need its own anchor).

### Path 4 — documented operational scope-out

`fleet doctor --auto-heal` works in principle for installations that
declare anchors; for this fleet specifically, the operator chooses
manual heal via the printed `fleet reauth <profile>` runbook (T-1054
Tier-1 path). The infrastructure stays available for future fleets
where operators wire SSH or other OOB anchors.

This concedes the **immediate value** of T-1680..T-1689 but preserves the
**framework value** (anyone deploying termlink with mature ops gets
auto-heal out-of-the-box once they wire `bootstrap_from`).

Honest. Not wasteful — the work primarily benefits downstream operators
who will wire anchors as part of standard deployment.

## Recommendation

**Recommend Path 1 (ssh: anchors) — with Path 4 as the interim default
until SSH keys exist.**

Rationale:
- Path 1 is the only R2-clean operational outcome.
- Path 2 and 3 fail R2 on analysis; further design work won't change that
  unless a genuinely separate trust channel is introduced.
- Path 4 is the right interim: no false promise of auto-heal where it
  cannot actually work today.
- Wiring SSH keys is a one-time operator action with broad benefit beyond
  termlink (general administration of the ring20 fleet).

Concrete first step the operator can take:

```
# On .107 — generate a heal-anchor key:
ssh-keygen -t ed25519 -f ~/.ssh/termlink-heal -N "" \
  -C "termlink-heal@workstation-107 $(date -I)"

# Read the public key:
cat ~/.ssh/termlink-heal.pub

# On EACH hub (.121 / .122 / .141 — via existing operator access):
# Append the public key to /root/.ssh/authorized_keys
# Verify: ssh -i ~/.ssh/termlink-heal root@<hub> 'cat /var/lib/termlink/hub.secret | head -c 12'

# On .107 — declare anchors:
# Add to ~/.termlink/hubs.toml under each profile:
#   bootstrap_from = "ssh:192.168.10.<N>"

# Validate:
termlink fleet bootstrap-check --all
# Should now show ANCHOR=ssh:..., STATUS=ok for each profile.
```

Until that happens, the operational stance is Path 4: `--auto-heal`
will continue to log "skip-no-anchor"; rotations are handled manually
via `fleet reauth <profile>` (Tier-1, T-1054).

## Decision

<!-- Fill via: fw inception decide T-1698 go|no-go|defer --rationale "..." -->

## Dialogue Log

### 2026-05-18 — Initial inception
- **Trigger:** sweep finding from `fleet bootstrap-check --all` (operator-readiness audit, agent-initiated under broad-initiative directive)
- **Discovered:** zero declared anchors; auto-heal stack dormant
- **Key insight from analysis:** Path 2 (warm-cache) fails R2 on closer look — the fetch channel and the auth being healed share the same secret. Initial appeal was misleading.
- **Action:** surface to operator, recommend Path 1 (SSH-key wiring) with Path 4 as the interim honest stance
