# T-206: Remove sudo from Installer — PATH-based Install

**Decision:** GO (2026-03-21)
**Rationale:** No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

## Problem

`install.sh` calls `sudo ln -sf` to create a symlink in `/usr/local/bin`. This fails in non-interactive shells (Claude Code, piped installs, CI) where sudo cannot prompt for a password.

## Key Findings

1. **`~/.local/bin` is wrong for macOS** — it's an XDG/Linux convention, not native to macOS. macOS doesn't add it to `$PATH` by default.
2. **`~/.agentic-framework/bin/` already contains the binaries** — symlinks elsewhere are pure indirection.
3. **Auto-modifying `.zshrc`/`.bashrc` is invasive** — causes duplicates on reinstall, conflicts with shell frameworks (oh-my-zsh, starship).
4. **Precedent:** rustup, cargo, and Homebrew's Linux installer all print a PATH line for the user to paste — they don't auto-modify RC files.

## Resolution

Remove all sudo calls. Print PATH instruction at end of install. Add `--modify-path` flag for opt-in RC file modification with idempotency (grep before append).
