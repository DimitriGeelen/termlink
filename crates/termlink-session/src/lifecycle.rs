use serde::{Deserialize, Serialize};

/// Session lifecycle states (from T-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Initializing,
    Ready,
    Busy,
    Draining,
    Gone,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "initializing"),
            Self::Ready => write!(f, "ready"),
            Self::Busy => write!(f, "busy"),
            Self::Draining => write!(f, "draining"),
            Self::Gone => write!(f, "gone"),
        }
    }
}

impl SessionState {
    /// Whether this state accepts incoming messages.
    pub fn accepts_messages(&self) -> bool {
        matches!(self, Self::Ready | Self::Busy)
    }

    /// Whether this state accepts new commands (not just queries).
    pub fn accepts_commands(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_accepts_all() {
        assert!(SessionState::Ready.accepts_messages());
        assert!(SessionState::Ready.accepts_commands());
    }

    #[test]
    fn busy_accepts_messages_not_commands() {
        assert!(SessionState::Busy.accepts_messages());
        assert!(!SessionState::Busy.accepts_commands());
    }

    #[test]
    fn draining_rejects_all() {
        assert!(!SessionState::Draining.accepts_messages());
        assert!(!SessionState::Draining.accepts_commands());
    }

    #[test]
    fn serialization() {
        let json = serde_json::to_string(&SessionState::Ready).unwrap();
        assert_eq!(json, "\"ready\"");
        let parsed: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SessionState::Ready);
    }
}
