# T-163: Cross-Machine Communication — RCA Findings
## Research Artifact | Created: 2026-03-18

## Issues Discovered During Live Cross-Machine Testing

Testing TermLink cross-machine communication between macOS (192.168.10.108) and
Linux Mint (192.168.10.107) revealed 3 bugs and 1 missing feature.

### Bug 1: pty inject reports "Injected 0 bytes" (T-177)

**Symptom:** `termlink pty inject <target> "text" --enter` always prints "Injected 0 bytes"
even though text IS injected successfully.

**Root cause:** Field name mismatch. Handler returns `bytes_len`, CLI reads `bytes_written`.
- handler.rs:513 returns `"bytes_len": bytes.len()`
- main.rs:1765 reads `result["bytes_written"].as_u64().unwrap_or(0)` → always 0

**Fix:** Change `bytes_written` to `bytes_len` in main.rs:1765.

### Bug 2: Enter key doesn't submit in Claude Code TUI (T-178)

**Symptom:** Text injected via pty inject appears in Claude Code's prompt but Enter
doesn't trigger submission. Token count stays constant.

**Root cause:** `cmd_inject` sends text + Enter (0x0D) as a single write to the PTY master.
Claude Code uses ink (React TUI) in raw mode. Ink distinguishes single-byte keypresses
from multi-byte paste input. When text+CR arrive in one chunk, ink treats it as pasted
text with a newline, not as "typed text followed by Enter keypress."

**Fix (recommended):** Split into two separate `pty.write()` calls:
1. Write text bytes
2. Small delay (1-10ms) 
3. Write `\r` (0x0D) as separate write

**Also investigate:** ICRNL termios flag may convert 0x0D → 0x0A before ink sees it.
If set, ink won't recognize it as `key.return`.

**Sources:**
- ink useInput: github.com/vadimdemedes/ink/blob/master/src/hooks/use-input.ts
- Claude Code issue #15553: programmatic input submission limitation
- claude-code-better-enter patch: github.com/appositeit/claude-code-better-enter

### Missing Feature: Cross-hub TLS trust (T-179)

**Symptom:** Hub A cannot forward requests to sessions on Hub B. TLS handshake fails
because Hub A's client uses its own local cert, not Hub B's cert.

**Root cause:** `connect_addr()` in client.rs:34 looks for `hub.cert.pem` in LOCAL
runtime dir. When connecting to a remote hub, it finds the local cert which is
irrelevant. Remote hub's self-signed cert is unknown.

**Recommended approach:** TOFU (Trust On First Use), like SSH known_hosts:
1. On first connection, accept remote cert and store SHA-256 fingerprint in `~/.termlink/known_hubs`
2. On subsequent connections, verify fingerprint matches
3. Optional: `termlink hub fingerprint` + `termlink hub trust <fp>` for manual pre-sharing

**Estimated effort:** ~300 lines (custom rustls ServerCertVerifier + known_hubs file)

### Observation: Local inject has same issue

The Enter-not-submitting bug is NOT remote-specific. It affects any `pty inject --enter`
against Claude Code's TUI, including local sessions. The `termlink interact` command
works because it injects into a SHELL (cooked mode), not into Claude Code's raw-mode TUI.

### Remote Server State (192.168.10.107) as of 2026-03-18T22:20Z

- TermLink binary updated to latest (28 commits pulled, rebuilt from source)
- Hub running: `termlink hub start --tcp 0.0.0.0:9100` (PID 4033597)
- Firewall: port 9100 opened for 192.168.10.0/24 (LAN only)
- Claude Code running in tmux session `tl-claude-master` as TermLink session `claude-master`
- tmux installed (was missing)
- T-177 fix NOT deployed to remote yet (still shows "Injected 0 bytes")
