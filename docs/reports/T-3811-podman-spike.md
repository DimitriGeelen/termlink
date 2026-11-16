# T-3811 — rootless Podman worker reaching the .107 hub (spike for ring20 T-2722)

Measured 2026-11-16 ~22:30Z on .107, as the non-root user `dimitri-mint-dev` (uid 1000, subuid 100000:65536),
Podman 4.9.3 (Ubuntu archive, installed by runme action 29, operator ruling A), netavark backend, pasta from
`passt`. Image `docker.io/library/alpine:3`; a static (musl) termlink 0.12.13+126.gfe72c850c mounted read-only.
Scripts: `.context/working/t3811/{spike,mapgw,claim}.sh`; raw output beside them.

## Results

| Measurement | Result |
|---|---|
| Image pull (alpine:3) | 2.25 s |
| Container start, `podman run --rm --network pasta alpine true` | 0.75 s, 0.75 s, 0.72 s |
| Default rootless network in Podman 4.9 | **slirp4netns** (not installed, so every run failed: "could not find slirp4netns"); pasta is the default only from Podman 5. Name it explicitly: `--network pasta`. |
| Hub probe, `--network pasta`, to the host's LAN address | **Connection refused.** pasta gives the container the host's own IP, so the host's address is the container itself. |
| Hub probe, `--network pasta`, to `host.containers.internal` | **Refused** as well: Podman 4.9 maps that name to the same host IP. |
| Hub probe, `--network pasta:--map-gw`, to the default gateway (192.168.10.2) | **OK**: correct fingerprint `sha256:cacc73ea…`, 0.73 s incl. start. With `--map-gw`, pasta forwards the gateway address to the host. |
| Hub probe, `--network host` | OK, correct fingerprint, 0.63–0.69 s incl. start. |
| Claim round-trip, `pasta:--map-gw`, scoped Interact token only (no hub.secret in the container, asserted) | **OK**: `channel claim` + `channel release --ack` on a scratch topic, `{"ack":true,"ok":true}`, 0.77 s incl. start. |

## What this means for T-2722 (IW-1, network half)

1. Rootless Podman works for a TermLink worker on a node like .107, with either `--network host` (simplest; no
   isolation of the network namespace) or `--network pasta:--map-gw` with the hub addressed through the gateway
   address (network namespace isolated; the hub sees the connection from the host).
2. Pin Podman ≥ 5, or set `--network pasta` (or `default_rootless_network_cmd = "pasta"` in containers.conf):
   Podman 4.9 falls back to slirp4netns.
3. The worker needs only a scoped Interact token; claim/release and posting work without `hub.secret`.
4. TLS trust: with the worker's TermLink home read-only, the TOFU pin cannot be stored, so every start re-trusts
   on first use. A worker image should ship the hub's fingerprint pre-pinned (known_hubs) or mount a writable
   pin store; otherwise a swapped hub certificate would be trusted silently on the next start.
5. Start cost is small (~0.75 s); this spike did not measure image size for RAM-only live workers (ring20's IW-2).
