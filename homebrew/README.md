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

## From source (developers)

If you need to modify or debug TermLink, install from source instead:

```bash
brew install rust  # if not already installed
git clone https://github.com/DimitriGeelen/termlink.git
cd termlink
cargo install --path crates/termlink-cli
```
