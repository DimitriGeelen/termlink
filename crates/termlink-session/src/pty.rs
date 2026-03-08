use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Arc;

use tokio::io::unix::AsyncFd;
use tokio::io::Interest;
use tokio::sync::Mutex;

use crate::scrollback::ScrollbackBuffer;

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
}

impl PtySession {
    /// Spawn a new PTY session running the given shell command.
    ///
    /// If `shell` is None, uses the user's default shell from $SHELL (or /bin/sh).
    pub fn spawn(shell: Option<&str>, scrollback_bytes: usize) -> Result<Self, PtyError> {
        let shell = shell.map(String::from).unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        });

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

    /// Send a signal to the child process.
    pub fn signal(&self, sig: i32) -> Result<(), PtyError> {
        let ret = unsafe { libc::kill(self.child_pid as libc::pid_t, sig) };
        if ret != 0 {
            return Err(PtyError::Io(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_and_exit() {
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        session.write(b"exit 0\n").await.unwrap();

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            session.read_loop(),
        )
        .await;

        assert!(result.is_ok(), "read_loop should terminate");
    }

    #[tokio::test]
    async fn spawn_echo_and_capture() {
        let session = PtySession::spawn(Some("/bin/sh"), 4096).unwrap();

        session
            .write(b"echo TERMLINK_TEST_MARKER\nexit\n")
            .await
            .unwrap();

        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(5),
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

    #[tokio::test]
    async fn child_pid_is_valid() {
        let session = PtySession::spawn(Some("/bin/sh"), 1024).unwrap();

        assert!(session.child_pid() > 0);

        session.signal(libc::SIGTERM).unwrap();
        let status = session.wait().await.unwrap();
        assert!(status > 0);
    }

    #[tokio::test]
    async fn write_and_read_roundtrip() {
        let session = PtySession::spawn(Some("/bin/sh"), 8192).unwrap();

        session
            .write(b"printf 'HELLO_PTY_WORLD'\nexit\n")
            .await
            .unwrap();

        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(5),
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
