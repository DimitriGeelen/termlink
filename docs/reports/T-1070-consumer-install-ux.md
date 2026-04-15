# T-1070 — Consumer install/update UX — research notes

**Status:** inception in progress. No production code until GO.

## Spike 3 — audit existing install infrastructure

**Findings (2026-04-15):**

### What we ship
- `.github/workflows/release.yml` — 5-target matrix fires on `v*` tags:
  - `aarch64-apple-darwin`, `x86_64-apple-darwin`
  - `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`
- Artifacts land on GitHub Releases (via OneDev mirror).

### What we offer consumers today
| Channel | Where | Gaps |
|---|---|---|
| `brew install termlink` | `homebrew/Formula/termlink.rb`, updated by `scripts/update-homebrew-sha.sh` | Only 4 targets hashed (darwin ×2 + linux-gnu ×2). **No musl, no LXC-friendly path.** Requires Homebrew. |
| `cargo install --git` | README | Requires rust toolchain. **This is the failure mode consumers hit.** |
| `scripts/deploy-remote.sh` | SSH-based | Requires SSH. Requires a pre-built local binary. Fresh-host install only. Explicitly "out of scope" for T-1016. |

### What we **don't** have
- No `install.sh` curl-pipe (e.g., `curl -fsSL termlink.sh/install \| sh`).
- No `termlink self-update` subcommand (confirmed via `termlink --help`).
- No OCI/container image (`ghcr.io/…/termlink:latest`).
- No musl variant in the Homebrew formula (despite being built by CI).

### Target-platform matrix for our actual consumers
- Dev box (.107-class): `x86_64-unknown-linux-gnu` ✓ already has binary
- ring20 LXC containers (.109/.121/.122): `x86_64-unknown-linux-gnu` or `…-musl` (depends on container base) — **currently no install path without SSH**
- Parallel session's failing host (from summary): `x86_64-linux-gnu`, no cargo available
- Operator laptops: `aarch64-apple-darwin` — covered by Homebrew ✓

**Shape of the gap:** LXC/container-class hosts with neither cargo nor Homebrew have **no install path** except scp-from-elsewhere. That's 3 of our 3 ring20 containers.

## Preliminary recommendation (draft — will harden before `fw inception decide`)

**Direction: GO**, with a minimal scope:

1. **Ship `install.sh`** (in-repo, published at a stable URL — e.g., `https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh`):
   - Detect target triple (`uname -m` + `uname -s` + glibc vs musl heuristic).
   - `curl -fsSLO` the right artifact from the latest GitHub Release.
   - Verify against `checksums.txt` (already published per `update-homebrew-sha.sh`).
   - Install to `/usr/local/bin/termlink` (or `$XDG_BIN_HOME`), chmod +x.
   - Print exact next step (e.g., "now run `termlink hub start`").

2. **Add musl to the Homebrew formula** (or swap the linux-gnu entry for linux-musl) — the musl build already exists in CI, the formula just doesn't reference it.

3. **Defer** a `termlink self-update` subcommand — `install.sh` + cron is enough mileage; self-update is a larger surface.

4. **Defer** an OCI image — the same install.sh inside a Dockerfile gets us the container story without a second pipeline.

**Rough size:** one install.sh (~80 lines), one Homebrew formula line change, one README update. Fits a single build task.

## Go/No-Go evaluation (preliminary)

**GO criteria from task body:**
- [x] Root cause identified with bounded fix path — **yes**: missing curl-pipe installer.
- [x] Fix is scoped, testable, and reversible — **yes**: ~80-line script + formula tweak.

**NO-GO criteria:**
- [ ] Problem requires fundamental redesign — **no**.
- [ ] Cost exceeds benefit — **no**: this is ~half a day of work, unblocks every fresh-host scenario.

## What's still unknown

- **Spike 1** (release cadence): haven't checked if the last 5 tags actually produced all 5 artifacts on GitHub Releases. Need GitHub API reach or a cached manifest.
- **Spike 2** (consumer platform inventory): partly done (ring20 + dev box); still unknown exactly what the parallel session's failing host runs.

Both spikes are low-risk to defer past GO — they'd sharpen the recommendation but don't change its direction.

## Next step

Paused pending human review (`fw task review T-1070`). If GO, proceed to the build task.
