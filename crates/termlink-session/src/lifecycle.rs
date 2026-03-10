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

/// Error returned when a state transition is invalid.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid state transition: {from} → {to}")]
pub struct InvalidTransition {
    pub from: SessionState,
    pub to: SessionState,
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

    /// Check whether transitioning from `self` to `target` is valid.
    ///
    /// Valid transitions:
    ///   Initializing → Ready
    ///   Ready → Busy | Draining
    ///   Busy → Ready | Draining
    ///   Draining → Gone
    ///   Gone → (none)
    pub fn valid_transition(&self, target: Self) -> Result<(), InvalidTransition> {
        let ok = match self {
            Self::Initializing => target == Self::Ready,
            Self::Ready => matches!(target, Self::Busy | Self::Draining),
            Self::Busy => matches!(target, Self::Ready | Self::Draining),
            Self::Draining => target == Self::Gone,
            Self::Gone => false,
        };
        if ok {
            Ok(())
        } else {
            Err(InvalidTransition { from: *self, to: target })
        }
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

    #[test]
    fn valid_transitions_succeed() {
        assert!(SessionState::Initializing.valid_transition(SessionState::Ready).is_ok());
        assert!(SessionState::Ready.valid_transition(SessionState::Busy).is_ok());
        assert!(SessionState::Ready.valid_transition(SessionState::Draining).is_ok());
        assert!(SessionState::Busy.valid_transition(SessionState::Ready).is_ok());
        assert!(SessionState::Busy.valid_transition(SessionState::Draining).is_ok());
        assert!(SessionState::Draining.valid_transition(SessionState::Gone).is_ok());
    }

    #[test]
    fn invalid_transitions_rejected() {
        // Can't skip states
        assert!(SessionState::Initializing.valid_transition(SessionState::Busy).is_err());
        assert!(SessionState::Initializing.valid_transition(SessionState::Gone).is_err());
        // Can't go backward
        assert!(SessionState::Ready.valid_transition(SessionState::Initializing).is_err());
        assert!(SessionState::Gone.valid_transition(SessionState::Ready).is_err());
        // Terminal state
        assert!(SessionState::Gone.valid_transition(SessionState::Initializing).is_err());
        assert!(SessionState::Gone.valid_transition(SessionState::Gone).is_err());
        // Can't drain from initializing
        assert!(SessionState::Initializing.valid_transition(SessionState::Draining).is_err());
    }

    #[test]
    fn invalid_transition_error_message() {
        let err = SessionState::Gone.valid_transition(SessionState::Ready).unwrap_err();
        assert_eq!(err.to_string(), "invalid state transition: gone → ready");
        assert_eq!(err.from, SessionState::Gone);
        assert_eq!(err.to, SessionState::Ready);
    }
}
