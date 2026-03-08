# T-002: Cross-Terminal Session Communication via Keyboard Input Injection

## Problem Statement

Can one terminal session inject keyboard input into another running terminal session, effectively enabling cross-session communication by treating keyboard input as a writable process interface?

## Core Insight

Terminal keyboard input is ultimately a process — bytes written to a file descriptor. If we can write to that fd from another process, we can "type" into a running session from outside it.

## Research Findings

### How Terminal Input Works (Unix/macOS)

1. **PTY (Pseudo-Terminal) Architecture:**
   - Every terminal session runs on a PTY pair: a **master** side and a **slave** side
   - The terminal emulator (iTerm2, Terminal.app) holds the master fd
   - The shell (zsh/bash) reads from the slave fd (`/dev/pts/N` on Linux, `/dev/ttysNNN` on macOS)
   - Keyboard input: User types → terminal emulator writes to master → kernel delivers to slave → shell reads it

2. **The Key Realization:**
   - Writing to the master side of a PTY is indistinguishable from keyboard input to the process reading the slave side
   - The `TIOCSTI` ioctl ("Terminal I/O Control — Simulate Terminal Input") historically allowed injecting characters into a terminal's input queue
   - **macOS/Linux divergence:** Linux kernels >=6.2 disabled `TIOCSTI` by default (security). macOS still supports it but it requires appropriate permissions.

### Mechanisms for Input Injection

#### 1. TIOCSTI (Legacy, Being Deprecated)
```c
// Inject one character at a time into terminal input queue
ioctl(fd, TIOCSTI, &character);
```
- **Pro:** Direct, simple, works at kernel level
- **Con:** Security risk (any process with fd access can inject), being removed from Linux
- **macOS status:** Still works but requires the process to have the terminal's fd

#### 2. Writing to PTY Master
```bash
# If you have access to the master fd, writing to it IS keyboard input
echo "ls -la" > /dev/ptmx  # (conceptual — actual fd management is more involved)
```
- **Pro:** Native mechanism, how terminal emulators already work
- **Con:** Getting the master fd from outside the terminal emulator process is non-trivial

#### 3. tmux / screen (Multiplexer Approach)
```bash
# tmux already solves this elegantly
tmux send-keys -t session_name "echo hello" Enter
```
- **Pro:** Battle-tested, cross-platform, handles escaping
- **Con:** Requires both sessions to be inside tmux

#### 4. AppleScript / osascript (macOS)
```bash
osascript -e 'tell application "Terminal" to do script "echo hello" in window 1'
```
- **Pro:** Works without modifying the target session
- **Con:** macOS only, executes a new command rather than injecting keystrokes

#### 5. Programmatic Keystroke Injection (macOS)
```bash
# Using CGEventCreateKeyboardEvent via Swift/ObjC
# Or: using osascript with System Events
osascript -e 'tell application "System Events" to keystroke "hello"'
```
- **Pro:** Simulates actual keystrokes at the OS level
- **Con:** Requires Accessibility permissions, targets the focused app only

#### 6. Named Pipes / Unix Sockets (Cooperative)
```bash
# Session A creates a pipe and reads from it
mkfifo /tmp/session_bridge
cat /tmp/session_bridge | while read cmd; do eval "$cmd"; done

# Session B writes to it
echo "ls -la" > /tmp/session_bridge
```
- **Pro:** Simple, no special permissions, cooperative
- **Con:** Requires target session to opt-in

#### 7. File Descriptor Passing via Unix Sockets
- One process can pass an open fd to another process via `SCM_RIGHTS` on a Unix domain socket
- This could allow sharing the PTY master fd between processes
- Advanced but very powerful

### Security Considerations

| Mechanism | Permission Required | Security Risk |
|-----------|-------------------|---------------|
| TIOCSTI | Terminal fd access | High (being deprecated for this reason) |
| PTY master write | Master fd access | Medium (contained to PTY) |
| tmux send-keys | tmux socket access | Low (tmux manages auth) |
| osascript | Accessibility perms | Medium (system-wide keystroke injection) |
| Named pipes | Filesystem perms | Low (cooperative, file-perm controlled) |
| fd passing | Socket access | Low (explicit handoff) |

### Prior Art

- **tmux/screen:** The gold standard for inter-terminal communication
- **Expect:** TCL tool that scripts interactive terminal sessions by controlling PTY master
- **reptyr:** Reparents a running process to a new terminal by manipulating PTYs
- **abduco:** Session management via PTY detach/attach
- **dtach:** Minimal detach/attach (like screen but single-window)
- **neercs:** Experimental terminal multiplexer with novel PTY tricks

### The Deep Insight

The concept is sound and well-established at multiple levels:

1. **Kernel level:** PTYs are bidirectional byte streams. Writing to master = keyboard input. This is not a hack — it's the design.
2. **Process level:** Any process that holds the master fd can inject input. Terminal emulators do this every time you press a key.
3. **IPC level:** Unix provides multiple mechanisms (pipes, sockets, shared memory) to bridge between processes.

The real question isn't "is this possible?" — it definitely is. The question is "at what level of abstraction do you want to operate?"

| Level | Approach | Complexity | Power |
|-------|----------|------------|-------|
| Application | tmux send-keys | Low | High (battle-tested) |
| OS | osascript/Accessibility | Medium | Medium (macOS only) |
| PTY | Direct master fd write | High | Very High (universal) |
| Kernel | TIOCSTI | Low (but deprecated) | High |
| Cooperative | Named pipes/sockets | Low | Medium (requires opt-in) |

## Potential Applications

1. **Agent-to-agent communication:** One Claude Code session sends commands to another
2. **Orchestration:** A master process coordinates multiple terminal sessions
3. **Session bridging:** Pass context/commands between separate terminal environments
4. **Remote pairing:** Inject input from a remote source into a local session
5. **Automated testing:** Script complex multi-terminal workflows

## Assumptions to Validate

- A-001: macOS still allows TIOCSTI without kernel restrictions
- A-002: PTY master fd can be obtained for an arbitrary terminal session
- A-003: tmux send-keys latency is acceptable for real-time communication
- A-004: Accessibility permissions are grantable programmatically

## Go/No-Go Criteria

- **Go if:** At least one mechanism works reliably on macOS without excessive permissions
- **No-go if:** All mechanisms require root or system-level permissions that users won't grant

## Dialogue Log

### 2026-03-08 — Initial exploration request
- **Human asked:** Can we investigate passing information between terminal sessions by injecting keyboard input, recognizing that terminal input is fundamentally a process/fd operation?
- **Core insight validated:** Yes — PTY architecture means keyboard input is just bytes on a file descriptor. Writing to the master side is indistinguishable from physical keystrokes.
- **Outcome:** Research documented, multiple viable mechanisms identified at different abstraction levels.
