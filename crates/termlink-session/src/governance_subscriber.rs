//! Governance subscriber — watches Output frames for pattern matches and emits Governance frames.
//!
//! The subscriber is opt-in, non-blocking, and processes output asynchronously via a bounded
//! broadcast channel. It strips ANSI escape sequences before matching, so patterns match
//! clean text regardless of terminal formatting.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;
use tokio::sync::{broadcast, mpsc};

use termlink_protocol::data::{Frame, FrameFlags, FrameType};
use termlink_protocol::governance::GovernanceEvent;

/// A named pattern to match against output text.
#[derive(Debug, Clone)]
pub struct PatternRule {
    /// Human-readable name for this pattern (included in governance events).
    pub name: String,
    /// Compiled regex pattern.
    pub regex: Regex,
}

/// Configuration for the governance subscriber.
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    /// Patterns to match against stripped output text.
    pub patterns: Vec<PatternRule>,
}

/// A governance subscriber that watches output frames for pattern matches.
///
/// Attach to a data plane connection's output broadcast channel. The subscriber
/// runs in its own task and emits [`Frame`]s of type [`FrameType::Governance`]
/// through an mpsc channel when patterns match.
pub struct GovernanceSubscriber {
    config: Arc<GovernanceConfig>,
}

impl GovernanceSubscriber {
    /// Create a new governance subscriber with the given configuration.
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Run the subscriber, consuming Output frames from `output_rx` and emitting
    /// Governance frames to `governance_tx`.
    ///
    /// This method runs until the broadcast channel closes. It is designed to be
    /// spawned as a tokio task.
    pub async fn run(
        &self,
        mut output_rx: broadcast::Receiver<Vec<u8>>,
        governance_tx: mpsc::Sender<Frame>,
    ) {
        let config = self.config.clone();
        let mut sequence: u64 = 0;

        loop {
            match output_rx.recv().await {
                Ok(data) => {
                    // Convert to string, strip ANSI, then match
                    let text = String::from_utf8_lossy(&data);
                    let stripped = strip_ansi_codes(&text);

                    for rule in &config.patterns {
                        if let Some(m) = rule.regex.find(&stripped) {
                            let event = GovernanceEvent {
                                pattern_name: rule.name.clone(),
                                match_text: m.as_str().to_string(),
                                timestamp: unix_timestamp(),
                                channel_id: 0,
                            };
                            let payload = event.to_payload();
                            let frame = Frame::new(
                                FrameType::Governance,
                                FrameFlags::empty(),
                                0,
                                sequence,
                                payload,
                            );
                            sequence += 1;

                            // Non-blocking send — drop if receiver is full/gone
                            if governance_tx.try_send(frame).is_err() {
                                tracing::debug!(
                                    pattern = %rule.name,
                                    "Governance subscriber: event dropped (channel full or closed)"
                                );
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(
                        skipped = n,
                        "Governance subscriber lagging, frames dropped"
                    );
                }
            }
        }
    }
}

/// Get current Unix timestamp in seconds.
fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Strip ANSI escape sequences and carriage returns from a string.
///
/// Handles CSI sequences (\x1b[...), OSC sequences (\x1b]...\x07 or \x1b]...\x1b\\),
/// and bare escape sequences.
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch.is_ascii_alphabetic() || ch == 'K' || ch == 'J' || ch == 'H' {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\x07' {
                            break;
                        }
                        if ch == '\x1b' {
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
        } else if c == '\r' {
            continue;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- strip_ansi_codes tests ---

    #[test]
    fn strip_plain_text() {
        assert_eq!(strip_ansi_codes("hello world"), "hello world");
    }

    #[test]
    fn strip_csi_color() {
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strip_osc_title() {
        assert_eq!(
            strip_ansi_codes("\x1b]0;My Title\x07text"),
            "text"
        );
    }

    #[test]
    fn strip_carriage_return() {
        assert_eq!(strip_ansi_codes("line\r\n"), "line\n");
    }

    // --- PatternRule + GovernanceSubscriber tests ---

    fn make_rule(name: &str, pattern: &str) -> PatternRule {
        PatternRule {
            name: name.to_string(),
            regex: Regex::new(pattern).unwrap(),
        }
    }

    #[tokio::test]
    async fn pattern_match_emits_governance_frame() {
        let config = GovernanceConfig {
            patterns: vec![make_rule("error_detect", r"(?i)fatal error")],
        };
        let subscriber = GovernanceSubscriber::new(config);

        let (output_tx, output_rx) = broadcast::channel::<Vec<u8>>(16);
        let (gov_tx, mut gov_rx) = mpsc::channel::<Frame>(16);

        let handle = tokio::spawn({
            let subscriber = GovernanceSubscriber::new(GovernanceConfig {
                patterns: subscriber.config.patterns.clone(),
            });
            async move {
                subscriber.run(output_rx, gov_tx).await;
            }
        });

        // Send output containing the pattern
        output_tx.send(b"some output FATAL ERROR occurred".to_vec()).unwrap();

        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            gov_rx.recv(),
        )
        .await
        .expect("timed out")
        .expect("channel closed");

        assert_eq!(frame.header.frame_type, FrameType::Governance);
        let event = GovernanceEvent::from_payload(&frame.payload).unwrap();
        assert_eq!(event.pattern_name, "error_detect");
        assert_eq!(event.match_text, "FATAL ERROR");

        drop(output_tx);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn no_match_no_frame() {
        let config = GovernanceConfig {
            patterns: vec![make_rule("secret", r"AWS_SECRET_ACCESS_KEY")],
        };

        let (output_tx, output_rx) = broadcast::channel::<Vec<u8>>(16);
        let (gov_tx, mut gov_rx) = mpsc::channel::<Frame>(16);

        let handle = tokio::spawn({
            let subscriber = GovernanceSubscriber::new(config);
            async move {
                subscriber.run(output_rx, gov_tx).await;
            }
        });

        // Send output that does NOT match
        output_tx.send(b"normal output here".to_vec()).unwrap();

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            gov_rx.recv(),
        )
        .await;

        assert!(result.is_err(), "should have timed out — no match expected");

        drop(output_tx);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn ansi_stripped_before_matching() {
        let config = GovernanceConfig {
            patterns: vec![make_rule("password", r"password=\S+")],
        };

        let (output_tx, output_rx) = broadcast::channel::<Vec<u8>>(16);
        let (gov_tx, mut gov_rx) = mpsc::channel::<Frame>(16);

        let handle = tokio::spawn({
            let subscriber = GovernanceSubscriber::new(config);
            async move {
                subscriber.run(output_rx, gov_tx).await;
            }
        });

        // Output with ANSI codes embedded in the pattern text
        output_tx
            .send(b"\x1b[31mpassword=\x1b[1msecret123\x1b[0m".to_vec())
            .unwrap();

        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            gov_rx.recv(),
        )
        .await
        .expect("timed out")
        .expect("channel closed");

        let event = GovernanceEvent::from_payload(&frame.payload).unwrap();
        assert_eq!(event.pattern_name, "password");
        assert_eq!(event.match_text, "password=secret123");

        drop(output_tx);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn multiple_patterns_multiple_matches() {
        let config = GovernanceConfig {
            patterns: vec![
                make_rule("error", r"(?i)error"),
                make_rule("warning", r"(?i)warning"),
            ],
        };

        let (output_tx, output_rx) = broadcast::channel::<Vec<u8>>(16);
        let (gov_tx, mut gov_rx) = mpsc::channel::<Frame>(16);

        let handle = tokio::spawn({
            let subscriber = GovernanceSubscriber::new(config);
            async move {
                subscriber.run(output_rx, gov_tx).await;
            }
        });

        // Output matches both patterns
        output_tx
            .send(b"ERROR: something bad; WARNING: also bad".to_vec())
            .unwrap();

        let frame1 = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            gov_rx.recv(),
        )
        .await
        .expect("timed out")
        .expect("channel closed");

        let frame2 = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            gov_rx.recv(),
        )
        .await
        .expect("timed out")
        .expect("channel closed");

        let event1 = GovernanceEvent::from_payload(&frame1.payload).unwrap();
        let event2 = GovernanceEvent::from_payload(&frame2.payload).unwrap();
        assert_eq!(event1.pattern_name, "error");
        assert_eq!(event2.pattern_name, "warning");

        drop(output_tx);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn governance_frame_sequence_increments() {
        let config = GovernanceConfig {
            patterns: vec![make_rule("any", r"match")],
        };

        let (output_tx, output_rx) = broadcast::channel::<Vec<u8>>(16);
        let (gov_tx, mut gov_rx) = mpsc::channel::<Frame>(16);

        let handle = tokio::spawn({
            let subscriber = GovernanceSubscriber::new(config);
            async move {
                subscriber.run(output_rx, gov_tx).await;
            }
        });

        output_tx.send(b"match one".to_vec()).unwrap();
        output_tx.send(b"match two".to_vec()).unwrap();

        let f1 = tokio::time::timeout(std::time::Duration::from_secs(2), gov_rx.recv())
            .await.unwrap().unwrap();
        let f2 = tokio::time::timeout(std::time::Duration::from_secs(2), gov_rx.recv())
            .await.unwrap().unwrap();

        assert_eq!(f1.header.sequence, 0);
        assert_eq!(f2.header.sequence, 1);

        drop(output_tx);
        let _ = handle.await;
    }
}
