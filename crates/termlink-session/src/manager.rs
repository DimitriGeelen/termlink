use std::path::{Path, PathBuf};

use tokio::net::UnixListener;

use crate::identity::SessionId;
use crate::lifecycle::SessionState;
use crate::liveness;
use crate::registration::{Registration, SessionConfig};

/// Errors from session management operations.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("registration file error: {0}")]
    Registration(String),

    #[error("session not found: {0}")]
    NotFound(String),

    #[error("display name '{0}' already in use by {1}")]
    NameConflict(String, String),
}

/// A running session with its registration and socket listener.
pub struct Session {
    pub registration: Registration,
    pub listener: UnixListener,
    sessions_dir: PathBuf,
}

impl Session {
    /// Register a new session in the default runtime directory.
    pub async fn register(config: SessionConfig) -> Result<Self, SessionError> {
        Self::register_in(config, &crate::discovery::sessions_dir()).await
    }

    /// Register a new session in a specific sessions directory.
    pub async fn register_in(
        config: SessionConfig,
        sessions_dir: &Path,
    ) -> Result<Self, SessionError> {
        let id = SessionId::generate();

        // Ensure directory exists with 0700 permissions
        std::fs::create_dir_all(sessions_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                sessions_dir,
                std::fs::Permissions::from_mode(0o700),
            )?;
        }

        let socket_path = Registration::socket_path(sessions_dir, &id);
        let json_path = Registration::json_path(sessions_dir, &id);

        // Check for display name conflicts.
        //
        // TOCTOU race (T-067): There is a time-of-check-to-time-of-use gap between
        // this uniqueness check and the JSON write below. Two concurrent register_in()
        // calls with the same display_name can both pass this check and write, resulting
        // in duplicate names. Mitigation path: use advisory file locking (flock) on a
        // per-name lockfile, or accept duplicates and resolve at query time. The current
        // single-threaded CLI usage makes this unlikely in practice.
        if let Some(ref name) = config.display_name
            && let Some(existing) = find_by_display_name(sessions_dir, name)?
        {
            if liveness::is_alive(&existing) {
                return Err(SessionError::NameConflict(
                    name.clone(),
                    existing.id.to_string(),
                ));
            }
            // Stale entry — clean it up
            liveness::cleanup_stale(&existing, sessions_dir);
        }

        // Remove leftover socket file if it exists (e.g., from a crash)
        let _ = std::fs::remove_file(&socket_path);

        // Bind Unix socket
        let listener = UnixListener::bind(&socket_path)?;

        // Write registration (initially in Initializing state).
        // If the JSON write fails, clean up the socket to avoid leaking it.
        let mut registration = Registration::new(id.clone(), config, socket_path.clone());
        if let Err(e) = registration.write_atomic(&json_path) {
            let _ = std::fs::remove_file(&socket_path);
            return Err(e.into());
        }

        // Transition to Ready.
        // If this fails, clean up both socket and JSON.
        if let Err(e) = registration.set_state(SessionState::Ready, &json_path) {
            let _ = std::fs::remove_file(&socket_path);
            let _ = std::fs::remove_file(&json_path);
            return Err(e.into());
        }

        tracing::info!(
            session_id = %id,
            display_name = %registration.display_name,
            socket = %registration.socket.display(),
            "Session registered"
        );

        Ok(Self {
            registration,
            listener,
            sessions_dir: sessions_dir.to_path_buf(),
        })
    }

    /// Deregister this session: transition to Draining and clean up.
    ///
    /// File removal happens in `Drop`, which fires when `self` is consumed
    /// at the end of this method.
    pub fn deregister(mut self) -> Result<(), SessionError> {
        let json_path =
            Registration::json_path(&self.sessions_dir, &self.registration.id);

        // Transition to draining
        let _ = self
            .registration
            .set_state(SessionState::Draining, &json_path);

        tracing::info!(
            session_id = %self.registration.id,
            "Session deregistered"
        );

        // `self` is dropped here — `Drop::drop` removes socket + JSON files
        Ok(())
    }

    /// Decompose the session into its parts, suppressing the `Drop` cleanup.
    ///
    /// Use this when you need to take ownership of individual fields (e.g.,
    /// to pass the listener to an accept loop). The caller is responsible
    /// for cleaning up files.
    pub fn into_parts(self) -> (Registration, UnixListener, PathBuf) {
        // Prevent Drop from running (which would remove files)
        let this = std::mem::ManuallyDrop::new(self);
        // Safety: we're reading the fields before ManuallyDrop prevents drop.
        // Each field is moved out exactly once.
        unsafe {
            let registration = std::ptr::read(&this.registration);
            let listener = std::ptr::read(&this.listener);
            let sessions_dir = std::ptr::read(&this.sessions_dir);
            (registration, listener, sessions_dir)
        }
    }

    /// Get the session ID.
    pub fn id(&self) -> &SessionId {
        &self.registration.id
    }

    /// Get the display name.
    pub fn display_name(&self) -> &str {
        &self.registration.display_name
    }

    /// Re-persist the registration JSON (e.g., after updating metadata).
    pub fn persist_registration(&self) -> Result<(), SessionError> {
        let json_path =
            Registration::json_path(&self.sessions_dir, &self.registration.id);
        self.registration.write_atomic(&json_path)?;
        Ok(())
    }
}

impl Drop for Session {
    /// Best-effort cleanup: remove socket and JSON registration files.
    ///
    /// This ensures files don't leak if `deregister()` is never called
    /// (e.g., due to a panic or early return).
    fn drop(&mut self) {
        let json_path =
            Registration::json_path(&self.sessions_dir, &self.registration.id);
        let _ = std::fs::remove_file(&self.registration.socket);
        let _ = std::fs::remove_file(&json_path);
        tracing::debug!(
            session_id = %self.registration.id,
            "Session dropped — best-effort cleanup"
        );
    }
}

/// List all sessions in the default runtime directory.
pub fn list_sessions(include_stale: bool) -> Result<Vec<Registration>, SessionError> {
    list_sessions_in(&crate::discovery::sessions_dir(), include_stale)
}

/// List all sessions in a specific sessions directory.
pub fn list_sessions_in(
    sessions_dir: &Path,
    include_stale: bool,
) -> Result<Vec<Registration>, SessionError> {
    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = Vec::new();

    for entry in std::fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let reg = match Registration::read_from(&path) {
            Ok(reg) => reg,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "Failed to read registration");
                continue;
            }
        };

        if !include_stale && !liveness::is_alive(&reg) {
            liveness::cleanup_stale(&reg, sessions_dir);
            continue;
        }

        sessions.push(reg);
    }

    sessions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(sessions)
}

/// Find a session by unique ID or display name in the default directory.
pub fn find_session(query: &str) -> Result<Registration, SessionError> {
    find_session_in(&crate::discovery::sessions_dir(), query)
}

/// Find a session by unique ID or display name in a specific directory.
pub fn find_session_in(
    sessions_dir: &Path,
    query: &str,
) -> Result<Registration, SessionError> {
    let sessions = list_sessions_in(sessions_dir, false)?;

    // Try unique ID match first
    if let Some(reg) = sessions.iter().find(|r| r.id.as_str() == query) {
        return Ok(reg.clone());
    }

    // Try display name match
    let matches: Vec<_> = sessions
        .iter()
        .filter(|r| r.display_name == query)
        .collect();

    match matches.len() {
        0 => Err(SessionError::NotFound(query.to_string())),
        1 => Ok(matches[0].clone()),
        _ => {
            let ids: Vec<_> = matches.iter().map(|r| r.id.to_string()).collect();
            Err(SessionError::Registration(format!(
                "Ambiguous display name '{}': matches {}",
                query,
                ids.join(", ")
            )))
        }
    }
}

/// Find sessions with a specific capability.
pub fn find_by_capability(cap: &str) -> Result<Vec<Registration>, SessionError> {
    let sessions = list_sessions(false)?;
    Ok(sessions
        .into_iter()
        .filter(|r| r.capabilities.iter().any(|c| c == cap))
        .collect())
}

/// Find sessions with a specific tag.
pub fn find_by_tag(tag: &str) -> Result<Vec<Registration>, SessionError> {
    let sessions = list_sessions(false)?;
    Ok(sessions
        .into_iter()
        .filter(|r| r.tags.iter().any(|t| t == tag))
        .collect())
}

/// Find sessions with a specific role.
pub fn find_by_role(role: &str) -> Result<Vec<Registration>, SessionError> {
    let sessions = list_sessions(false)?;
    Ok(sessions
        .into_iter()
        .filter(|r| r.roles.iter().any(|r| r == role))
        .collect())
}

/// A stale session that was found during cleanup.
#[derive(Debug, Clone)]
pub struct StaleSession {
    pub id: String,
    pub display_name: String,
    pub pid: u32,
    pub created_at: String,
}

/// Scan for stale sessions and optionally remove them.
///
/// Returns the list of stale sessions found. If `remove` is true,
/// their registration artifacts (socket + JSON files) are deleted.
pub fn clean_stale_sessions(
    sessions_dir: &Path,
    remove: bool,
) -> Result<Vec<StaleSession>, SessionError> {
    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut stale = Vec::new();

    for entry in std::fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let reg = match Registration::read_from(&path) {
            Ok(reg) => reg,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "Failed to read registration");
                continue;
            }
        };

        if !liveness::is_alive(&reg) {
            stale.push(StaleSession {
                id: reg.id.to_string(),
                display_name: reg.display_name.clone(),
                pid: reg.pid,
                created_at: reg.created_at.clone(),
            });
            if remove {
                liveness::cleanup_stale(&reg, sessions_dir);
            }
        }
    }

    Ok(stale)
}

/// Find a session by display name in a specific directory.
fn find_by_display_name(
    sessions_dir: &Path,
    name: &str,
) -> Result<Option<Registration>, SessionError> {
    if !sessions_dir.exists() {
        return Ok(None);
    }

    for entry in std::fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(reg) = Registration::read_from(&path)
            && reg.display_name == name
        {
            return Ok(Some(reg));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Unique counter to ensure each test gets its own directory
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_test_dir(name: &str) -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        // Keep path short — Unix sockets have a 104-byte path limit on macOS
        let dir = PathBuf::from(format!("/tmp/tl-t{}-{}", n, name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn register_and_deregister() {
        let sessions_dir = unique_test_dir("reg");

        let session = Session::register_in(SessionConfig::default(), &sessions_dir)
            .await
            .unwrap();
        let id = session.id().clone();

        // Verify files exist
        let json_path = Registration::json_path(&sessions_dir, &id);
        let socket_path = Registration::socket_path(&sessions_dir, &id);
        assert!(json_path.exists());
        assert!(socket_path.exists());

        // Verify registration is readable
        let reg = Registration::read_from(&json_path).unwrap();
        assert_eq!(reg.state, SessionState::Ready);

        // Deregister
        session.deregister().unwrap();
        assert!(!json_path.exists());
        assert!(!socket_path.exists());
    }

    #[tokio::test]
    async fn register_with_display_name() {
        let sessions_dir = unique_test_dir("name");

        let config = SessionConfig {
            display_name: Some("test-builder".into()),
            ..Default::default()
        };

        let session = Session::register_in(config, &sessions_dir).await.unwrap();
        assert_eq!(session.display_name(), "test-builder");

        session.deregister().unwrap();
    }

    #[tokio::test]
    async fn list_sessions_empty() {
        let sessions_dir = unique_test_dir("empty");
        let sessions = list_sessions_in(&sessions_dir, false).unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn list_sessions_finds_registered() {
        let sessions_dir = unique_test_dir("listfind");

        let s1 = Session::register_in(SessionConfig::default(), &sessions_dir)
            .await
            .unwrap();
        let s2 = Session::register_in(SessionConfig::default(), &sessions_dir)
            .await
            .unwrap();

        let sessions = list_sessions_in(&sessions_dir, false).unwrap();
        assert_eq!(sessions.len(), 2);

        s1.deregister().unwrap();
        s2.deregister().unwrap();
    }

    #[tokio::test]
    async fn find_session_by_id_and_name() {
        let sessions_dir = unique_test_dir("find");

        let config = SessionConfig {
            display_name: Some("finder-test".into()),
            ..Default::default()
        };
        let session = Session::register_in(config, &sessions_dir).await.unwrap();
        let id_str = session.id().as_str().to_string();

        // Find by ID
        let found = find_session_in(&sessions_dir, &id_str).unwrap();
        assert_eq!(found.display_name, "finder-test");

        // Find by display name
        let found = find_session_in(&sessions_dir, "finder-test").unwrap();
        assert_eq!(found.id.as_str(), id_str);

        // Not found
        assert!(find_session_in(&sessions_dir, "nonexistent").is_err());

        session.deregister().unwrap();
    }

    #[tokio::test]
    async fn name_conflict_detected() {
        let sessions_dir = unique_test_dir("conflict");

        let config1 = SessionConfig {
            display_name: Some("conflicted".into()),
            ..Default::default()
        };
        let s1 = Session::register_in(config1, &sessions_dir).await.unwrap();

        let config2 = SessionConfig {
            display_name: Some("conflicted".into()),
            ..Default::default()
        };
        let result = Session::register_in(config2, &sessions_dir).await;
        assert!(matches!(result, Err(SessionError::NameConflict(_, _))));

        s1.deregister().unwrap();
    }

    #[tokio::test]
    async fn drop_cleans_up_files() {
        let sessions_dir = unique_test_dir("drop");

        let session = Session::register_in(SessionConfig::default(), &sessions_dir)
            .await
            .unwrap();
        let id = session.id().clone();
        let json_path = Registration::json_path(&sessions_dir, &id);
        let socket_path = Registration::socket_path(&sessions_dir, &id);

        assert!(json_path.exists());
        assert!(socket_path.exists());

        // Drop without calling deregister()
        drop(session);

        assert!(!json_path.exists(), "Drop should remove JSON file");
        assert!(!socket_path.exists(), "Drop should remove socket file");
    }

    #[test]
    fn clean_stale_sessions_dry_run() {
        let dir = unique_test_dir("clean-dry");

        // Create a fake stale registration (dead PID, no socket)
        let id = crate::identity::SessionId::generate();
        let socket_path = dir.join(format!("{id}.sock"));
        let json_path = dir.join(format!("{id}.json"));
        std::fs::write(&socket_path, b"fake").unwrap();

        let config = SessionConfig::default();
        let mut reg = Registration::new(id.clone(), config, socket_path.clone());
        reg.pid = 4_000_000; // dead PID
        reg.write_atomic(&json_path).unwrap();

        // Dry run: should find but not remove
        let stale = clean_stale_sessions(&dir, false).unwrap();
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].id, id.to_string());
        assert!(json_path.exists(), "dry run should not delete files");

        // Actual run: should remove
        let stale = clean_stale_sessions(&dir, true).unwrap();
        assert_eq!(stale.len(), 1);
        assert!(!json_path.exists(), "should delete JSON file");
        assert!(!socket_path.exists(), "should delete socket file");

        // Second run: nothing left
        let stale = clean_stale_sessions(&dir, true).unwrap();
        assert!(stale.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
