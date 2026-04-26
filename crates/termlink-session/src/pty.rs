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
    /// Last known terminal mode (for change detection).
    last_mode: Arc<Mutex<Option<TerminalMode>>>,
}

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

        let ret = unsafe {
            libc::openpty(
                &mut master_fd,
                &mut slave_fd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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
                    Self::scan_alternate_screen(chunk, &self.alternate_screen).await;

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
    /// `\x1b[?1049h` enters alternate screen, `\x1b[?1049l` leaves it.
    async fn scan_alternate_screen(chunk: &[u8], alt_screen: &Arc<Mutex<bool>>) {
        // Look for the escape sequences in the chunk
        let enter_seq = b"\x1b[?1049h";
        let leave_seq = b"\x1b[?1049l";

        let mut changed = None;
        for window in chunk.windows(enter_seq.len()) {
            if window == enter_seq {
                changed = Some(true);
            } else if window == leave_seq {
                changed = Some(false);
            }
        }

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

impl Drop for PtySession {
    fn drop(&mut self) {
        let pid = self.child_pid as libc::pid_t;
        unsafe {
            // Kill child to ensure PTY device is released promptly
            libc::kill(pid, libc::SIGKILL);
            // Reap to avoid zombie processes
            let mut status: libc::c_int = 0;
            libc::waitpid(pid, &mut status, libc::WNOHANG);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_util::PTY_LOCK;

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

        // Simulate entering alternate screen
        PtySession::scan_alternate_screen(b"\x1b[?1049h", &alt_screen).await;
        assert!(*alt_screen.lock().await, "Should detect alternate screen enter");

        // Simulate leaving alternate screen
        PtySession::scan_alternate_screen(b"\x1b[?1049l", &alt_screen).await;
        assert!(!*alt_screen.lock().await, "Should detect alternate screen leave");
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
