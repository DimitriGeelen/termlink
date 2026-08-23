use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::unix::AsyncFd;
use tokio::io::Interest;
use tokio::sync::Mutex;

use crate::scrollback::ScrollbackBuffer;

/// Terminal mode flags detected via tcgetattr on the PTY master fd.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalMode {
    /// Canonical (line-editing) mode — ICANON flag is set.
    pub canonical: bool,
    /// Echo mode — ECHO flag is set.
    pub echo: bool,
    /// Raw mode — neither ICANON nor ECHO is set.
    pub raw: bool,
    /// Whether the terminal is in alternate screen buffer mode.
    pub alternate_screen: bool,
}

/// Escape sequences that move a terminal in (`true`) or out (`false`) of the
/// alternate screen buffer (T-2738). See `scan_alternate_screen` for why all
/// three DECSET variants are treated as equivalent and why `?1048` is excluded.
const ALT_SCREEN_SEQUENCES: &[(&[u8], bool)] = &[
    (b"\x1b[?1049h", true),
    (b"\x1b[?1049l", false),
    (b"\x1b[?1047h", true),
    (b"\x1b[?1047l", false),
    (b"\x1b[?47h", true),
    (b"\x1b[?47l", false),
];

/// Length of the longest entry in [`ALT_SCREEN_SEQUENCES`], which sizes the
/// cross-read carry. Asserted against the table in `alt_screen_max_seq_len_matches_table`
/// so adding a longer sequence cannot silently under-size the carry.
const ALT_SCREEN_MAX_SEQ_LEN: usize = 8;

/// Errors from PTY operations.
#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("failed to create PTY: {0}")]
    Create(std::io::Error),

    #[error("failed to fork: {0}")]
    Fork(std::io::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("child process exited with status {0}")]
    ChildExited(i32),
}

/// Wrapper around an OwnedFd for use with AsyncFd.
struct AsyncPtyFd(OwnedFd);

impl AsRawFd for AsyncPtyFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

/// A PTY-backed session that owns a shell process.
///
/// Provides read access to terminal output and write access for input injection.
pub struct PtySession {
    /// Async wrapper for the PTY master (read side).
    master_read: AsyncFd<AsyncPtyFd>,
    /// Raw fd for the PTY master (write side, duplicated).
    master_write_fd: Arc<Mutex<OwnedFd>>,
    /// Child process PID.
    child_pid: u32,
    /// Scrollback buffer for output capture.
    scrollback: Arc<Mutex<ScrollbackBuffer>>,
    /// Whether the terminal is in alternate screen buffer mode.
    alternate_screen: Arc<Mutex<bool>>,
    /// Rolling tail of the last read (< one escape-sequence length) so an
    /// alternate-screen sequence split across a read boundary is still detected.
    scan_carry: Arc<Mutex<Vec<u8>>>,
    /// Last known terminal mode (for change detection).
    last_mode: Arc<Mutex<Option<TerminalMode>>>,
}

/// Initial PTY row count, applied at `openpty` time (T-2727).
///
/// Not a preference — a floor. `openpty` supplies 0x0 when handed a NULL
/// `winp`, so without this a fresh session reports a zero-sized terminal to
/// its child. 80x24 is the VT100 default every terminal emulator falls back
/// to. Any real size arrives later via [`PtySession::resize`].
pub const DEFAULT_PTY_ROWS: u16 = 24;

/// Initial PTY column count, applied at `openpty` time (T-2727).
/// See [`DEFAULT_PTY_ROWS`].
pub const DEFAULT_PTY_COLS: u16 = 80;

impl PtySession {
    /// Spawn a new PTY session running the given shell command.
    ///
    /// If `shell` is None, uses the user's default shell from $SHELL (or /bin/sh).
    pub fn spawn(shell: Option<&str>, scrollback_bytes: usize) -> Result<Self, PtyError> {
        Self::spawn_with_env(shell, scrollback_bytes, &[])
    }

    /// Spawn a new PTY session, injecting the given env-var pairs into the child.
    ///
    /// Each pair is set via `setenv(KEY, VALUE, overwrite=1)` after `fork()` and before
    /// `execvp()`, so the child shell (and anything it execs) inherits them. Used by
    /// `termlink register --shell` to seed `TERMLINK_SESSION_ID` for whoami auto-resolve.
    pub fn spawn_with_env(
        shell: Option<&str>,
        scrollback_bytes: usize,
        env: &[(String, String)],
    ) -> Result<Self, PtyError> {
        let shell = shell.map(String::from).unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        });

        // Pre-allocate CStrings for env injection so the child only does signal-safe work.
        let env_c: Vec<(std::ffi::CString, std::ffi::CString)> = env
            .iter()
            .filter_map(|(k, v)| {
                let kc = std::ffi::CString::new(k.as_str()).ok()?;
                let vc = std::ffi::CString::new(v.as_str()).ok()?;
                Some((kc, vc))
            })
            .collect();

        // Create PTY pair
        let mut master_fd: libc::c_int = 0;
        let mut slave_fd: libc::c_int = 0;

        // T-2727: seed an initial window size.
        //
        // `openpty` with a NULL `winp` leaves the pty at rows=0, cols=0 — the
        // kernel supplies no default (verified: `os.openpty()` + TIOCGWINSZ
        // reports 0x0). Every session therefore started life claiming a
        // zero-sized terminal until some caller happened to drive `resize()`,
        // and nothing in the spawn path does. Full-screen children — vim,
        // less, top, and the agent TUIs this tool exists to host — query
        // TIOCGWINSZ at startup and get a degenerate answer.
        //
        // This is a FLOOR, not a pin: `resize()` (and the `command.resize` RPC
        // / client Resize frame that call it) still override it freely. 80x24
        // is the historical VT100 default and what every terminal emulator
        // falls back to, so a child that never gets a real size behaves the
        // way it would under any other terminal rather than uniquely badly.
        let initial_ws = libc::winsize {
            ws_row: DEFAULT_PTY_ROWS,
            ws_col: DEFAULT_PTY_COLS,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        let ret = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &initial_ws,
            )
        };
        if ret != 0 {
            return Err(PtyError::Create(std::io::Error::last_os_error()));
        }

        // Fork
        let pid = unsafe { libc::fork() };
        if pid < 0 {
            unsafe {
                libc::close(master_fd);
                libc::close(slave_fd);
            }
            return Err(PtyError::Fork(std::io::Error::last_os_error()));
        }

        if pid == 0 {
            // === Child process ===
            unsafe {
                libc::close(master_fd);
                libc::setsid();
                libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0);
                libc::dup2(slave_fd, 0);
                libc::dup2(slave_fd, 1);
                libc::dup2(slave_fd, 2);
                if slave_fd > 2 {
                    libc::close(slave_fd);
                }
                // Inject env pairs (T-1302) before exec so the new program sees them.
                for (k, v) in &env_c {
                    libc::setenv(k.as_ptr(), v.as_ptr(), 1);
                }
                let shell_c = std::ffi::CString::new(shell.as_str()).unwrap();
                let args = [shell_c.as_ptr(), std::ptr::null()];
                libc::execvp(shell_c.as_ptr(), args.as_ptr());
                libc::_exit(127);
            }
        }

        // === Parent process ===
        unsafe { libc::close(slave_fd) };

        // Set master to non-blocking for async I/O
        unsafe {
            let flags = libc::fcntl(master_fd, libc::F_GETFL);
            libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        // Duplicate the fd for separate read/write handles
        let master_fd_dup = unsafe { libc::dup(master_fd) };
        if master_fd_dup < 0 {
            return Err(PtyError::Create(std::io::Error::last_os_error()));
        }
        // Set dup to non-blocking too
        unsafe {
            let flags = libc::fcntl(master_fd_dup, libc::F_GETFL);
            libc::fcntl(master_fd_dup, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        let read_fd = unsafe { OwnedFd::from_raw_fd(master_fd) };
        let write_fd = unsafe { OwnedFd::from_raw_fd(master_fd_dup) };

        let async_read =
            AsyncFd::with_interest(AsyncPtyFd(read_fd), Interest::READABLE)
                .map_err(PtyError::Create)?;

        Ok(Self {
            master_read: async_read,
            master_write_fd: Arc::new(Mutex::new(write_fd)),
            child_pid: pid as u32,
            scrollback: Arc::new(Mutex::new(ScrollbackBuffer::new(scrollback_bytes))),
            alternate_screen: Arc::new(Mutex::new(false)),
            scan_carry: Arc::new(Mutex::new(Vec::new())),
            last_mode: Arc::new(Mutex::new(None)),
        })
    }

    /// Run the PTY read loop, feeding output into the scrollback buffer.
    ///
    /// This should be spawned as a task. Returns when the child process exits
    /// or the PTY master is closed.
    pub async fn read_loop(&self) -> Result<(), PtyError> {
        self.read_loop_with_broadcast(None).await
    }

    /// Run the PTY read loop with an optional broadcast channel for data plane streaming.
    ///
    /// Output is always written to the scrollback buffer. If a broadcast sender is provided,
    /// output chunks are also sent to data plane clients.
    pub async fn read_loop_with_broadcast(
        &self,
        broadcast_tx: Option<tokio::sync::broadcast::Sender<Vec<u8>>>,
    ) -> Result<(), PtyError> {
        let mut buf = [0u8; 4096];

        loop {
            let mut guard = self
                .master_read
                .readable()
                .await
                .map_err(PtyError::Io)?;

            match guard.try_io(|inner| {
                let fd = inner.as_raw_fd();
                let n = unsafe {
                    libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
                };
                if n < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(n as usize)
                }
            }) {
                Ok(Ok(0)) => return Ok(()),
                Ok(Ok(n)) => {
                    let chunk = &buf[..n];

                    // Scan for alternate screen buffer escape sequences
                    Self::scan_alternate_screen(chunk, &self.alternate_screen, &self.scan_carry)
                        .await;

                    let mut scrollback = self.scrollback.lock().await;
                    scrollback.append(chunk);
                    // Broadcast to data plane clients (if any)
                    if let Some(ref tx) = broadcast_tx {
                        let _ = tx.send(chunk.to_vec());
                    }
                }
                Ok(Err(e)) => {
                    // EIO is expected when child exits
                    if e.raw_os_error() == Some(libc::EIO) {
                        return Ok(());
                    }
                    return Err(PtyError::Io(e));
                }
                Err(_would_block) => continue,
            }
        }
    }

    /// Write bytes to the PTY master (input injection).
    pub async fn write(&self, data: &[u8]) -> Result<(), PtyError> {
        let fd_guard = self.master_write_fd.lock().await;
        let fd = fd_guard.as_raw_fd();
        let mut offset = 0;

        while offset < data.len() {
            let n = unsafe {
                libc::write(
                    fd,
                    data[offset..].as_ptr() as *const libc::c_void,
                    data.len() - offset,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    // Brief yield, then retry
                    tokio::task::yield_now().await;
                    continue;
                }
                return Err(PtyError::Io(err));
            }
            offset += n as usize;
        }
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let ws = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let fd = self.master_read.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &ws) };
        if ret != 0 {
            return Err(PtyError::Io(std::io::Error::last_os_error()));
        }
        Ok(())
    }

    /// Get the child process PID.
    pub fn child_pid(&self) -> u32 {
        self.child_pid
    }

    /// Get a clone of the scrollback buffer handle.
    pub fn scrollback(&self) -> Arc<Mutex<ScrollbackBuffer>> {
        self.scrollback.clone()
    }

    /// Wait for the child process to exit and return its status.
    pub async fn wait(&self) -> Result<i32, PtyError> {
        let pid = self.child_pid as libc::pid_t;

        tokio::task::spawn_blocking(move || {
            let mut status: libc::c_int = 0;
            let ret = unsafe { libc::waitpid(pid, &mut status, 0) };
            if ret < 0 {
                return Err(PtyError::Io(std::io::Error::last_os_error()));
            }
            if libc::WIFEXITED(status) {
                Ok(libc::WEXITSTATUS(status))
            } else if libc::WIFSIGNALED(status) {
                Ok(128 + libc::WTERMSIG(status))
            } else {
                Ok(-1)
            }
        })
        .await
        .map_err(|e| PtyError::Io(std::io::Error::other(e)))?
    }

    /// Query the current terminal mode via tcgetattr on the PTY master fd.
    ///
    /// Returns the current canonical/echo/raw state and alternate screen status.
    pub async fn terminal_mode(&self) -> Result<TerminalMode, PtyError> {
        let fd = self.master_read.as_raw_fd();
        let mut termios: libc::termios = unsafe { std::mem::zeroed() };
        let ret = unsafe { libc::tcgetattr(fd, &mut termios) };
        if ret != 0 {
            return Err(PtyError::Io(std::io::Error::last_os_error()));
        }

        let canonical = (termios.c_lflag & libc::ICANON as libc::tcflag_t) != 0;
        let echo = (termios.c_lflag & libc::ECHO as libc::tcflag_t) != 0;
        let raw = !canonical && !echo;
        let alternate_screen = *self.alternate_screen.lock().await;

        Ok(TerminalMode {
            canonical,
            echo,
            raw,
            alternate_screen,
        })
    }

    /// Check for terminal mode changes. Returns the new mode and the previous mode
    /// if a change was detected, or None if the mode hasn't changed.
    ///
    /// Also returns a `password_prompt_hint` flag when the ECHO flag drops.
    pub async fn poll_mode_change(
        &self,
    ) -> Result<Option<(TerminalMode, Option<TerminalMode>, bool)>, PtyError> {
        let current = self.terminal_mode().await?;
        let mut last = self.last_mode.lock().await;

        let result = match last.as_ref() {
            Some(prev) if *prev != current => {
                // Detect password prompt hint: ECHO was on, now off
                let password_hint = prev.echo && !current.echo;
                let previous = prev.clone();
                *last = Some(current.clone());
                Some((current, Some(previous), password_hint))
            }
            None => {
                // First poll — store initial state, no change event
                *last = Some(current);
                None
            }
            _ => None, // No change
        };

        Ok(result)
    }

    /// Scan output bytes for alternate screen buffer escape sequences.
    ///
    /// Three DECSET pairs put a terminal in and out of the alternate screen, and
    /// a session can be driven by any of them (T-2738):
    ///
    /// - `\x1b[?1049h/l` — the modern combined save-cursor-and-switch
    /// - `\x1b[?1047h/l` — switch only
    /// - `\x1b[?47h/l`   — the original, still emitted by older/simpler TUIs
    ///
    /// For the question this flag answers — "is the session on the alternate
    /// screen?" — all three are equivalent, so they share one table. `?1048` is
    /// deliberately absent: it saves and restores the cursor without switching
    /// screens.
    ///
    /// A single PTY read may deliver a sequence in a chunk shorter than the
    /// sequence, or split across a read boundary. Scanning each chunk in
    /// isolation would silently miss both cases, so we prepend a rolling carry
    /// of the trailing `max_seq_len - 1` bytes from prior reads and scan
    /// `[carry || chunk]`.
    ///
    /// **No-double-count invariant:** a match is applied only if it extends past
    /// the carry into the new chunk. This used to be a consequence of every
    /// sequence being exactly 8 bytes while the carry held 7 — no window could
    /// fit inside the carry. That reasoning died with the 6-byte `?47h`, which
    /// fits in a 7-byte carry comfortably, so the rule is now enforced directly
    /// instead of inherited from a length coincidence.
    async fn scan_alternate_screen(
        chunk: &[u8],
        alt_screen: &Arc<Mutex<bool>>,
        carry: &Arc<Mutex<Vec<u8>>>,
    ) {
        let mut carry_guard = carry.lock().await;
        let carry_len = carry_guard.len();

        // Combined = trailing bytes carried from prior reads, then this read.
        let mut combined = Vec::with_capacity(carry_len + chunk.len());
        combined.extend_from_slice(&carry_guard);
        combined.extend_from_slice(chunk);

        // Positional scan: later matches win, so the last state change in the
        // byte stream is the one that sticks.
        let mut changed = None;
        for start in 0..combined.len() {
            for (seq, enters) in ALT_SCREEN_SEQUENCES {
                let end = start + seq.len();
                // Past the end, or already fully seen on a previous call.
                if end > combined.len() || end <= carry_len {
                    continue;
                }
                if &combined[start..end] == *seq {
                    changed = Some(*enters);
                }
            }
        }

        // Retain the last max_seq_len-1 bytes so a sequence straddling the next
        // read boundary is still completed on the following call.
        let keep = ALT_SCREEN_MAX_SEQ_LEN - 1;
        if combined.len() > keep {
            let start = combined.len() - keep;
            *carry_guard = combined[start..].to_vec();
        } else {
            *carry_guard = combined;
        }
        drop(carry_guard);

        if let Some(new_state) = changed {
            let mut state = alt_screen.lock().await;
            *state = new_state;
        }
    }

    /// Get a clone of the alternate screen state handle.
    pub fn alternate_screen(&self) -> Arc<Mutex<bool>> {
        self.alternate_screen.clone()
    }

    /// Get a clone of the last mode handle (for external change detection).
    pub fn last_mode(&self) -> Arc<Mutex<Option<TerminalMode>>> {
        self.last_mode.clone()
    }

    /// Send a signal to the child process.
    pub fn signal(&self, sig: i32) -> Result<(), PtyError> {
        let ret = unsafe { libc::kill(self.child_pid as libc::pid_t, sig) };
        if ret != 0 {
            return Err(PtyError::Io(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

/// Outcome of attempting to reap a child process (T-2737).
///
/// Deliberately three-valued rather than a bool: "reaped", "somebody else
/// already reaped it", and "the budget ran out and a zombie survives" are
/// three different facts, and only the third is worth warning about.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ReapOutcome {
    /// `waitpid` returned the child — no zombie remains.
    Reaped,
    /// `waitpid` reported no such child: already reaped elsewhere (e.g. by
    /// `PtySession::wait`), or never ours to begin with.
    NoChild,
    /// The budget elapsed with the child still unreaped — a zombie survives.
    TimedOut { waited_ms: u64 },
}

/// How long `drop` will wait for a SIGKILLed child to become reapable.
///
/// SIGKILL is not instantaneous: the kernel must schedule the target, tear its
/// address space down, and re-parent/notify before the child is in a state
/// `waitpid` can collect. That is normally sub-millisecond, but it is never
/// zero — which is exactly why the pre-T-2737 single `WNOHANG` call reaped
/// nothing. 100ms is far above the observed cost of a normal teardown while
/// still bounding `drop`, which may run on a runtime worker thread.
const REAP_BUDGET: std::time::Duration = std::time::Duration::from_millis(100);

/// Reap `pid`, retrying `WNOHANG` until the child is collectable or `budget`
/// elapses (T-2737).
///
/// `WNOHANG` asks "is the child reapable *right now*", so a single call issued
/// immediately after `SIGKILL` reliably answers "no" and collects nothing. The
/// retry is what makes the reap actually happen. Poll interval starts fine and
/// backs off, so the common case (child already gone) costs one syscall and the
/// slow case does not spin.
pub(crate) fn reap_child_bounded(
    pid: libc::pid_t,
    budget: std::time::Duration,
) -> ReapOutcome {
    let start = std::time::Instant::now();
    let mut interval = std::time::Duration::from_micros(200);
    loop {
        let mut status: libc::c_int = 0;
        let ret = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if ret > 0 {
            return ReapOutcome::Reaped;
        }
        if ret < 0 {
            // ECHILD (already reaped / not ours) or EINVAL — either way there
            // is nothing here for us to collect, so do not burn the budget.
            return ReapOutcome::NoChild;
        }
        // ret == 0: alive but not yet reapable. Keep trying until the budget.
        let elapsed = start.elapsed();
        if elapsed >= budget {
            return ReapOutcome::TimedOut {
                waited_ms: elapsed.as_millis() as u64,
            };
        }
        std::thread::sleep(interval.min(budget - elapsed));
        interval = (interval * 2).min(std::time::Duration::from_millis(5));
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        let pid = self.child_pid as libc::pid_t;
        // Kill child to ensure PTY device is released promptly.
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
        // Then actually reap it. A single WNOHANG here would return before the
        // kernel had finished killing the child, leaving a zombie per session
        // for the life of a long-running host (T-2737).
        if let ReapOutcome::TimedOut { waited_ms } = reap_child_bounded(pid, REAP_BUDGET) {
            // Directive #2 — a leaked zombie is a failure, not a no-op.
            tracing::warn!(
                child_pid = pid,
                waited_ms,
                "PTY child not reapable within budget — zombie survives until this process exits"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::PTY_LOCK;

    /// Spawn a real child that lives for `secs` and return its pid.
    ///
    /// `std::process::Child::drop` does not wait, so nothing reaps this behind
    /// the tests' back — the raw `waitpid` calls below are the only reaper.
    fn spawn_sleeper(secs: &str) -> (std::process::Child, libc::pid_t) {
        let child = std::process::Command::new("sleep")
            .arg(secs)
            .spawn()
            .expect("sleep is available");
        let pid = child.id() as libc::pid_t;
        (child, pid)
    }

    fn wnohang_once(pid: libc::pid_t) -> libc::c_int {
        let mut status: libc::c_int = 0;
        unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) }
    }

    /// T-2737 (load-bearing, deterministic): the reap must actually WAIT.
    ///
    /// The child is alive for ~50ms and is never signalled, so at call time it
    /// is definitively not reapable. A single `WNOHANG` — the pre-T-2737 shape
    /// — returns 0 and collects nothing; only the retry loop reaps it. This
    /// test does not depend on kill-delivery timing, so reverting
    /// `reap_child_bounded` to one `WNOHANG` fails it every run, not sometimes.
    #[test]
    fn bounded_reap_waits_for_a_child_that_exits_later() {
        let (_child, pid) = spawn_sleeper("0.05");

        assert_eq!(
            wnohang_once(pid),
            0,
            "precondition: a live child is not reapable, so WNOHANG yields 0"
        );

        let outcome = reap_child_bounded(pid, std::time::Duration::from_secs(2));
        assert_eq!(
            outcome,
            ReapOutcome::Reaped,
            "a 2s budget must outlast a 50ms sleeper"
        );
    }

    /// T-2737: the case `drop` actually hits — reap a child killed a moment ago.
    #[test]
    fn bounded_reap_collects_a_killed_child() {
        let (_child, pid) = spawn_sleeper("30");
        unsafe { libc::kill(pid, libc::SIGKILL) };

        assert_eq!(
            reap_child_bounded(pid, std::time::Duration::from_secs(2)),
            ReapOutcome::Reaped
        );
        assert!(
            wnohang_once(pid) < 0,
            "after a successful reap the child is gone: waitpid must report ECHILD"
        );
    }

    /// T-2737: the timeout arm is reachable, so it is not dead code.
    #[test]
    fn bounded_reap_times_out_against_a_live_child() {
        let (mut child, pid) = spawn_sleeper("30");

        match reap_child_bounded(pid, std::time::Duration::ZERO) {
            ReapOutcome::TimedOut { .. } => {}
            other => panic!("a live child with no budget must time out, got {other:?}"),
        }

        let _ = child.kill();
        let _ = child.wait();
    }

    /// T-2737: a pid that is not our child is `NoChild`, never a timeout —
    /// otherwise `drop` would burn its whole budget on an already-reaped child.
    #[test]
    fn bounded_reap_reports_no_child_for_a_foreign_pid() {
        // pid 1 is never a child of the test process. No signal is sent.
        assert_eq!(
            reap_child_bounded(1, std::time::Duration::from_secs(2)),
            ReapOutcome::NoChild
        );
    }

    /// T-2727: a freshly spawned PTY must report a usable window size.
    ///
    /// Load-bearing: revert the `&initial_ws` argument in `spawn_with_env`
    /// back to `std::ptr::null_mut()` and this test fails with 0x0. Without
    /// it, a full-screen child (vim, less, top, an agent TUI) queries
    /// TIOCGWINSZ at startup and is told the terminal has no size.
    #[tokio::test]
    async fn spawn_has_nonzero_winsize() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let fd = session.master_read.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
        assert_eq!(ret, 0, "TIOCGWINSZ should succeed on the pty master");

        assert!(
            ws.ws_row > 0 && ws.ws_col > 0,
            "fresh PTY reported a degenerate {}x{} window size — a child \
             querying TIOCGWINSZ would be told the terminal has no size",
            ws.ws_col,
            ws.ws_row
        );
        assert_eq!(ws.ws_row, DEFAULT_PTY_ROWS);
        assert_eq!(ws.ws_col, DEFAULT_PTY_COLS);
    }

    /// T-2727: the seeded size is a FLOOR, not a pin — `resize()` still wins.
    #[tokio::test]
    async fn resize_overrides_default_winsize() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        session.resize(120, 40).unwrap();

        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let fd = session.master_read.as_raw_fd();
        let ret = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) };
        assert_eq!(ret, 0, "TIOCGWINSZ should succeed on the pty master");
        assert_eq!(ws.ws_col, 120, "caller-driven resize must override default");
        assert_eq!(ws.ws_row, 40, "caller-driven resize must override default");
    }

    #[tokio::test]
    async fn spawn_and_exit() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        session.write(b"exit 0\n").await.unwrap();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            session.read_loop(),
        )
        .await;

        assert!(result.is_ok(), "read_loop should terminate");
    }

    #[tokio::test]
    async fn spawn_echo_and_capture() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 4096).unwrap();

        session
            .write(b"echo TERMLINK_TEST_MARKER\nexit\n")
            .await
            .unwrap();

        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            session.read_loop(),
        )
        .await;

        let scrollback = session.scrollback();
        let sb = scrollback.lock().await;
        let bytes = sb.last_n_bytes(sb.len());
        let output_str = String::from_utf8_lossy(&bytes);

        assert!(
            output_str.contains("TERMLINK_TEST_MARKER"),
            "Scrollback should contain marker, got: {:?}",
            output_str
        );
    }

    /// T-1302: env vars passed to spawn_with_env are visible to the spawned shell.
    #[tokio::test]
    async fn spawn_passes_env_to_child() {
        let _guard = PTY_LOCK.lock().await;
        let env = vec![("TL_TEST_VAR".to_string(), "hello-1302".to_string())];
        let session = PtySession::spawn_with_env(Some("/bin/sh"), 4096, &env).unwrap();

        session
            .write(b"echo VAR_IS=$TL_TEST_VAR\nexit\n")
            .await
            .unwrap();

        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            session.read_loop(),
        )
        .await;

        let scrollback = session.scrollback();
        let sb = scrollback.lock().await;
        let bytes = sb.last_n_bytes(sb.len());
        let output_str = String::from_utf8_lossy(&bytes);

        assert!(
            output_str.contains("VAR_IS=hello-1302"),
            "Child shell should see injected env, got: {:?}",
            output_str
        );
    }

    #[tokio::test]
    async fn child_pid_is_valid() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        assert!(session.child_pid() > 0);

        session.signal(libc::SIGTERM).unwrap();
        let status = session.wait().await.unwrap();
        assert!(status > 0);
    }

    #[tokio::test]
    async fn terminal_mode_returns_valid_flags() {
        let _guard = PTY_LOCK.lock().await;
        // Verify tcgetattr succeeds and returns a valid TerminalMode struct.
        // Note: the exact flags depend on the shell and OS configuration.
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        // Give the shell a moment to initialize
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mode = session.terminal_mode().await.unwrap();

        // raw should be consistent with canonical/echo flags
        assert_eq!(mode.raw, !mode.canonical && !mode.echo,
            "raw should be !canonical && !echo, got canonical={} echo={} raw={}",
            mode.canonical, mode.echo, mode.raw);
        assert!(!mode.alternate_screen, "Should not be in alternate screen initially");

        session.write(b"exit 0\n").await.unwrap();
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            session.wait(),
        ).await;
    }

    #[tokio::test]
    async fn terminal_mode_struct_serialization() {
        let mode = TerminalMode {
            canonical: true,
            echo: true,
            raw: false,
            alternate_screen: false,
        };
        let json = serde_json::to_value(&mode).unwrap();
        assert_eq!(json["canonical"], true);
        assert_eq!(json["echo"], true);
        assert_eq!(json["raw"], false);
        assert_eq!(json["alternate_screen"], false);

        let deserialized: TerminalMode = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, mode);
    }

    #[tokio::test]
    async fn alternate_screen_detection() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        // Simulate entering alternate screen
        PtySession::scan_alternate_screen(b"\x1b[?1049h", &alt_screen, &carry).await;
        assert!(*alt_screen.lock().await, "Should detect alternate screen enter");

        // Simulate leaving alternate screen
        PtySession::scan_alternate_screen(b"\x1b[?1049l", &alt_screen, &carry).await;
        assert!(!*alt_screen.lock().await, "Should detect alternate screen leave");
    }

    /// T-2738: the carry is sized from the table, so a longer sequence added
    /// later cannot silently under-size it and start dropping split reads.
    #[test]
    fn alt_screen_max_seq_len_matches_table() {
        let longest = ALT_SCREEN_SEQUENCES
            .iter()
            .map(|(seq, _)| seq.len())
            .max()
            .expect("table is not empty");
        assert_eq!(
            ALT_SCREEN_MAX_SEQ_LEN, longest,
            "ALT_SCREEN_MAX_SEQ_LEN must track the longest sequence in the table"
        );
    }

    /// T-2738: the original 6-byte DECSET pair, still emitted by older TUIs.
    #[tokio::test]
    async fn alternate_screen_detects_legacy_47_variant() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        PtySession::scan_alternate_screen(b"\x1b[?47h", &alt_screen, &carry).await;
        assert!(*alt_screen.lock().await, "?47h enters the alternate screen");

        PtySession::scan_alternate_screen(b"\x1b[?47l", &alt_screen, &carry).await;
        assert!(!*alt_screen.lock().await, "?47l leaves the alternate screen");
    }

    /// T-2738: the switch-only variant.
    #[tokio::test]
    async fn alternate_screen_detects_1047_variant() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        PtySession::scan_alternate_screen(b"\x1b[?1047h", &alt_screen, &carry).await;
        assert!(*alt_screen.lock().await, "?1047h enters the alternate screen");

        PtySession::scan_alternate_screen(b"\x1b[?1047l", &alt_screen, &carry).await;
        assert!(!*alt_screen.lock().await, "?1047l leaves the alternate screen");
    }

    /// T-2738: the 6-byte variant split across a read boundary — the length the
    /// fixed-8 window logic had no way to complete.
    #[tokio::test]
    async fn alternate_screen_detection_split_across_reads_six_byte_variant() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        // "\x1b[?47h" split as "\x1b[?4" (4 bytes) then "7h" (2 bytes).
        PtySession::scan_alternate_screen(b"\x1b[?4", &alt_screen, &carry).await;
        assert!(
            !*alt_screen.lock().await,
            "partial sequence must not trigger detection yet"
        );
        PtySession::scan_alternate_screen(b"7h", &alt_screen, &carry).await;
        assert!(
            *alt_screen.lock().await,
            "?47h completed across the read boundary should flip state to true"
        );
    }

    /// T-2738 (negative, PL-219): `?1048` saves and restores the cursor — it does
    /// NOT switch screens. A scanner that matched loosely on `?104` would flip
    /// the flag here and be wrong in the quiet direction.
    #[tokio::test]
    async fn alternate_screen_ignores_1048_cursor_save() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        PtySession::scan_alternate_screen(b"\x1b[?1048h", &alt_screen, &carry).await;
        assert!(
            !*alt_screen.lock().await,
            "?1048h is a cursor save, not an alternate-screen switch"
        );

        // And it must not suppress a real switch arriving right after it.
        PtySession::scan_alternate_screen(b"\x1b[?1049h", &alt_screen, &carry).await;
        assert!(*alt_screen.lock().await, "?1049h still enters after a ?1048h");
    }

    /// T-2738: the no-double-count invariant, made observable.
    ///
    /// `?47h` is 6 bytes and the carry keeps 7, so after this chunk the whole
    /// sequence sits inside the carry. Scanning a following chunk that contains
    /// no sequence at all must not re-apply it. The externally-set `false` is
    /// what makes the re-application visible — without it the stale match would
    /// rewrite the same value and hide.
    #[tokio::test]
    async fn alternate_screen_does_not_reapply_a_sequence_held_in_carry() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        PtySession::scan_alternate_screen(b"\x1b[?47h", &alt_screen, &carry).await;
        assert!(*alt_screen.lock().await);
        assert!(
            carry.lock().await.len() >= b"\x1b[?47h".len(),
            "precondition: the carry is long enough to hold the whole sequence"
        );

        *alt_screen.lock().await = false;

        PtySession::scan_alternate_screen(b"x", &alt_screen, &carry).await;
        assert!(
            !*alt_screen.lock().await,
            "a sequence already applied and now living in the carry must not fire again"
        );
    }

    /// T-2513: the enter sequence split across two reads must still be detected.
    /// FAILS against the old per-chunk `chunk.windows(8)` logic (neither chunk
    /// contains the whole 8-byte sequence).
    #[tokio::test]
    async fn alternate_screen_detection_split_across_reads() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        // "\x1b[?1049h" split as "\x1b[?104" (6 bytes) then "9h" (2 bytes).
        PtySession::scan_alternate_screen(b"\x1b[?104", &alt_screen, &carry).await;
        assert!(
            !*alt_screen.lock().await,
            "Partial sequence must not trigger detection yet"
        );
        PtySession::scan_alternate_screen(b"9h", &alt_screen, &carry).await;
        assert!(
            *alt_screen.lock().await,
            "Sequence completed across the read boundary should flip state to true"
        );

        // Same for the leave sequence, split differently.
        PtySession::scan_alternate_screen(b"\x1b[?10", &alt_screen, &carry).await;
        PtySession::scan_alternate_screen(b"49l", &alt_screen, &carry).await;
        assert!(
            !*alt_screen.lock().await,
            "Leave sequence completed across the read boundary should flip state to false"
        );
    }

    /// T-2513: the sequence delivered one byte per read (each chunk < seq_len, so
    /// `windows(8)` yields an empty iterator) must still be detected.
    #[tokio::test]
    async fn alternate_screen_detection_byte_at_a_time() {
        let alt_screen = Arc::new(Mutex::new(false));
        let carry = Arc::new(Mutex::new(Vec::new()));

        for b in b"\x1b[?1049h" {
            PtySession::scan_alternate_screen(&[*b], &alt_screen, &carry).await;
        }
        assert!(
            *alt_screen.lock().await,
            "Byte-at-a-time enter sequence should flip state to true"
        );

        for b in b"\x1b[?1049l" {
            PtySession::scan_alternate_screen(&[*b], &alt_screen, &carry).await;
        }
        assert!(
            !*alt_screen.lock().await,
            "Byte-at-a-time leave sequence should flip state to false"
        );
    }

    #[tokio::test]
    async fn poll_mode_change_initial_stores_mode() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // First poll should initialize and return None (no change)
        let result = session.poll_mode_change().await.unwrap();
        assert!(result.is_none(), "First poll should return None (initialization)");

        // Second poll with no changes should also return None
        let result = session.poll_mode_change().await.unwrap();
        assert!(result.is_none(), "Second poll with no change should return None");

        session.write(b"exit 0\n").await.unwrap();
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            session.wait(),
        ).await;
    }

    #[tokio::test]
    async fn write_and_read_roundtrip() {
        let _guard = PTY_LOCK.lock().await;
        let session = PtySession::spawn(Some("/bin/sh"), 8192).unwrap();

        session
            .write(b"printf 'HELLO_PTY_WORLD'\nexit\n")
            .await
            .unwrap();

        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            session.read_loop(),
        )
        .await;

        let scrollback = session.scrollback();
        let sb = scrollback.lock().await;
        let bytes = sb.last_n_bytes(sb.len());
        let output = String::from_utf8_lossy(&bytes);

        assert!(
            output.contains("HELLO_PTY_WORLD"),
            "Expected HELLO_PTY_WORLD in output, got: {:?}",
            output
        );
    }
}
