# T-1290 Inception: Why does ring20-management hub.secret keep rotating?

**Status:** in-progress (exploration)
**Created:** 2026-04-26
**Inception task:** [T-1290](../../.tasks/active/T-1290-investigate-ring20-management-hubsecret-.md)

## TL;DR (working hypothesis)

The ring20-management hub on CT 200 / .122 most likely has no systemd unit
setting `TERMLINK_RUNTIME_DIR=/var/lib/termlink`, so its `runtime_dir` falls
back to the legacy default `/tmp/termlink-0` — which is tmpfs inside the LXC
container and therefore wipes on every CT reboot. The hub's
persist-if-present logic is structurally correct (T-985, T-1028) but cannot
help when the directory itself is volatile.

This is **scenario (b)** from the original framing ("systemd restart landing
in a different runtime_dir") — but degenerated, because the runtime_dir
doesn't just differ, it disappears.

Spike 1 (operator console inspection) is needed to confirm:
1. `<runtime_dir>/hub.secret` location on .122 (is it `/tmp/termlink-0/...`?)
2. Whether systemd unit exists and sets `TERMLINK_RUNTIME_DIR=...`
3. Mount type of the runtime_dir parent (`mount | grep ...`)

If confirmed, fix is the same one-liner T-931/T-935 already shipped for the
.107 deploy: install/update the systemd unit, restart hub once, re-pin from
clients (one-off heal, but never again).

## Evidence collected (2026-04-26 session)

### Spike 3 (cross-check peer hubs) — partial, in-progress

| Hub                          | Address               | Persist working?              | Notes |
|------------------------------|------------------------|-------------------------------|-------|
| self-hub (claude-dev .102)   | 192.168.10.102:9100    | YES                            | `hub.secret` mtime 2026-04-12 00:55 (~14 days old, unchanged across many container restarts). runtime_dir = `/var/lib/termlink`, persistent overlay-fs. |
| ring20-dashboard (.121)      | 192.168.10.121:9100    | YES (auth still valid)         | Last pin held; HMAC handshake passing 2026-04-26. Binary slightly older — `T-1235` legacy-inbox-fallback fired (channel.* unavailable), but persist-if-present + secret survival pre-date channel.*. |
| ring20-management (.122)     | 192.168.10.122:9100    | NO — recurring rotation       | Both TLS fingerprint AND HMAC secret rotated since prior pin. Required `tofu clear` + auth-mismatch indicates BOTH cert and secret regenerated. CT 200 known to reboot under proxmox /var/log pressure (G-009 / T-1137). |

A-3 ("other fleet hubs do NOT rotate") — **CONFIRMED**. Two distinct hubs
on different code generations both preserve. The bug is ring20-management
specific, not a fleet-wide regression.

### Persist-if-present timeline (git evidence)

```
c5a4366a  2026-04-12  T-985   TLS cert persist-if-present
8e3fef2e  2026-04-12  T-985   Complete — 179/179 hub tests pass
accedcc0  2026-04-12  T-1028  Remove tls::cleanup() from shutdown
eacb38dc  2026-04-12  T-935   Document runtime dir /tmp/termlink-0 → /var/lib/termlink
                              "the move happened structurally when T-931 installed the
                               systemd unit with Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink"
b36f2732              T-1051  Spike: persist-if-present is correct, design gap is no rotation protocol
```

So the persist mechanism has been in place since 2026-04-12. Any hub
deployed after that date and configured via the standard systemd unit
should preserve. The gap is in **deployment configuration**, not code.

### Prior learning: PL-021 already saw this pattern

A `fw work-on T-1290` lookup surfaced:

> **PL-021** — When a remote hub rotates BOTH secret and TLS cert (container nuke / clean ru...

Past observation captured under T-1067 already noted the
both-secret-AND-cert rotation pattern as indicating "container nuke / clean
runtime_dir." That aligns 1:1 with the volatile-tmpfs hypothesis. PL-021
deserves a re-read once spike 1 confirms the runtime_dir on .122.

### What we already know about .122 deployment

- Memory: `ring20-management` is a container (CT 200) on proxmox .180, currently .122 (4 IP renumbers in 5 days, latest 2026-04-20).
- Memory: hub at .122:9100.
- Memory: ring20-dashboard is a sibling container at .121.
- T-1137 root-cause chain: proxmox .180 /var/log fills → CT 200 reboots → hub on .122 regenerates cert. The fact that the SECRET also regenerates (not just cert) tells us the entire `runtime_dir` is being lost across CT reboot — not selectively cleaned.

If `runtime_dir` were `/var/lib/termlink` on a normal LXC mount, a CT
reboot would NOT wipe it (LXC root filesystems persist by default). So
either:
- runtime_dir is `/tmp/termlink-0` (tmpfs inside CT) — fits everything we observe
- or `/var/lib/termlink` is on a tmpfs/zram inside CT 200 specifically (less likely; would need to be intentionally configured)

The first hypothesis matches the structural pattern: an old deploy that
predates T-935's systemd-unit migration, or a deploy that was done
manually without the env var.

## Mapping to scenarios

CLAUDE.md enumerates three rotation scenarios:

> Rotation still happens in three scenarios: first-time deploy of
> persist-if-present onto a hub that previously regenerated on restart, a
> systemd restart landing in a different runtime_dir, or an intentional
> operator regeneration.

`.122` does NOT match scenarios 1 (it has been deployed for weeks, repeated
rotations) or 3 (operator hasn't been intentionally regenerating). It maps
cleanly to scenario 2, in the degenerate sub-case where the runtime_dir
isn't merely *different* but *volatile*.

## Recommendation (preliminary — pending spike 1 confirmation)

**Recommendation:** GO (after spike 1 confirms tmpfs runtime_dir on .122).

**Fix (small, scoped, reversible):**
1. On .180 console → enter CT 200: `pct enter 200`
2. Verify symptom: `ls -la /tmp/termlink-0/ /var/lib/termlink/ 2>&1` and `mount | grep termlink`
3. If `/tmp/termlink-0/hub.secret` exists and `/var/lib/termlink/` does not contain the live hub.secret: install systemd unit (or override) with `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink`, ensure `/var/lib/termlink` exists with mode 700, then `systemctl daemon-reload && systemctl restart termlink-hub`.
4. Document the one-time heal in T-1137 / G-011 episodics — every client must re-pin once.
5. Verify with `termlink hub status` on .122 and `termlink remote ping ring20-management` from peers.

This is the same migration T-935 already documented; `.122` simply hasn't
received it yet.

**Rejected alternative:** Generalize the fix to a "runtime_dir validator at
hub start" that refuses to start if runtime_dir is on tmpfs. Useful belt-
and-suspenders, but out of scope for T-1290 — file as a follow-up if the
operator agrees.

## Open evidence gaps

- Spike 1 is unrun (operator console inspection of .122 needed). Without
  it, the recommendation rests on negative evidence (everything else
  matches scenario 2; nothing else fits as cleanly).
- We do not have a copy of CT 200's hub binary version / build commit. If
  .122 is older than 2026-04-12, even setting the runtime_dir won't help —
  the binary itself needs to ship persist-if-present. A version check
  (`termlink hub version` or `<binary> --version`) on .122 closes this.

## Next actions

- [ ] Operator: when console-accessing .122, run the spike 1 commands and
  paste output back. The fix-or-not decision flips on this.
- [ ] If spike 1 confirms tmpfs runtime_dir: record GO and create build
  task for the migration on CT 200.
- [ ] Either way: update CLAUDE.md "Hub Auth Rotation Protocol" section
  with the volatile-runtime_dir sub-case so future agents recognize it
  faster.

## See also

- `docs/operations/termlink-hub-runtime-migration.md` (T-935) — the exact
  migration this CT needs.
- `docs/reports/T-1051-termlink-auth-reliability-inception.md` — original
  Option D framing.
- T-1284 (G-011 cache value-comparison) — addresses the receiving-side
  drift; this task addresses why the rotation happens upstream in the
  first place.
