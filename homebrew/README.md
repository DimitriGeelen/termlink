# Homebrew Tap for TermLink

Cross-terminal session communication tool.

## Install

```bash
brew tap DimitriGeelen/termlink
brew install termlink
```

## Verify

```bash
termlink --version
```

## Upgrade

```bash
brew update
brew upgrade termlink
```

## How the formula is updated

Nobody edits the version or the sha256 values by hand (T-3754). On every `vX.Y.Z` tag,
the `homebrew-tap` job in `.github/workflows/release.yml` runs after the GitHub release
is published:

1. It runs `test-render-formula.sh` and `test-update-tap.sh` (hermetic: temp dirs and local git repos only).
2. It downloads the release's `checksums.txt`.
3. `update-tap.sh` renders `Formula/termlink.rb` with `render-formula.sh` from the template in this
   directory. Each `sha256` gets the hash of the artifact named in the `url` line above it. A missing
   or malformed hash stops the job before anything is pushed.
4. It pushes the formula to `DimitriGeelen/homebrew-termlink` only if it changed. It never replaces
   a newer version with an older one.
5. It clones the tap again **without** credentials (what `brew tap` sees) and fails unless the formula's
   version equals the tag. The last line of a green run is `tap formula version == vX.Y.Z`.

Pre-release tags (anything other than `vX.Y.Z`) are skipped with a notice.

**One-time setup (operator, on github.com):**

1. Create the **public** repository `DimitriGeelen/homebrew-termlink`. It can be empty.
2. Create a fine-grained personal access token limited to that repository, with **Contents: Read and write**.
3. Add it to `DimitriGeelen/termlink` → Settings → Secrets and variables → Actions as the repository secret `HOMEBREW_TAP_TOKEN`.

Until both exist the job is red, and its `::error::` line names what is missing. The binaries are
still published, because the tap job runs after the release job and cannot block it. After setup,
re-run the failed `Homebrew tap` job of the latest release from the Actions page. No new tag is needed.

`Formula/termlink.rb` in this repo is the template. Its version and hashes are the last values
rendered into this repo and can lag the tap. The tap is the published copy.

## From source (developers)

If you need to modify or debug TermLink, install from source instead:

```bash
brew install rust  # if not already installed
git clone https://github.com/DimitriGeelen/termlink.git
cd termlink
cargo install --path crates/termlink-cli
```
