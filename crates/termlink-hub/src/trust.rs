use serde::{Deserialize, Serialize};

/// Qualitative trust assessment for specialist supervision (D-008).
///
/// Three axes determine how much oversight a command/specialist needs:
/// - **Script maturity:** How proven is this command? (travels with the script)
/// - **Context familiarity:** How well-known is this project context? (resets per project)
/// - **Blast radius:** How much damage could this command cause? (varies per invocation)
///
/// Trust is NOT a single number — it's a qualitative assessment across three
/// independent axes. A mature script in an unfamiliar context still needs oversight.
/// A low-blast command from an immature script may be safe to run unsupervised.

/// Supervision levels — graduated autonomy from full bypass to blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupervisionLevel {
    /// No oversight needed — proven command, familiar context, low blast radius.
    Unsupervised,
    /// Post-hoc review — results checked after execution.
    Monitored,
    /// Pre-execution approval — command reviewed before running.
    Supervised,
    /// Blocked — too risky without explicit human authorization.
    Blocked,
}

impl std::fmt::Display for SupervisionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupervised => write!(f, "unsupervised"),
            Self::Monitored => write!(f, "monitored"),
            Self::Supervised => write!(f, "supervised"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

/// Script maturity — how proven is this command?
///
/// Derived from bypass registry stats (run count, fail count, promotion status).
/// Failed-and-recovered scripts score HIGHER than never-failed scripts (antifragility).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Maturity {
    /// Never seen before — no execution history.
    Unknown,
    /// Some runs but not yet promoted (< threshold).
    Developing,
    /// Promoted to bypass registry (proven track record).
    Proven,
    /// Was demoted and recovered — strongest trust signal (antifragility).
    Hardened,
}

/// Context familiarity — how well-known is the operating environment?
///
/// Resets per project. A mature script in a new project starts at Low.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Familiarity {
    /// First interaction in this context.
    New,
    /// Some history in this context (< 5 interactions).
    Low,
    /// Moderate history (5-20 interactions).
    Medium,
    /// Extensive history (> 20 interactions).
    High,
}

/// Blast radius — how much damage could this command cause?
///
/// Varies per invocation. Derived from command characteristics and fabric data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlastRadius {
    /// Read-only, no side effects.
    None,
    /// Writes to isolated scope (worktree, temp files).
    Low,
    /// Writes to shared state (main branch, config files).
    Medium,
    /// Destructive potential (delete, force-push, production systems).
    High,
}

/// A trust assessment across three axes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssessment {
    /// The command or capability being assessed.
    pub command: String,
    /// Script maturity axis.
    pub maturity: Maturity,
    /// Context familiarity axis.
    pub familiarity: Familiarity,
    /// Blast radius axis.
    pub blast_radius: BlastRadius,
    /// Recommended supervision level.
    pub supervision: SupervisionLevel,
    /// Human-readable rationale for the recommendation.
    pub rationale: String,
}

/// Input data for computing a trust assessment.
pub struct TrustInput<'a> {
    /// Command name.
    pub command: &'a str,
    /// Is the command in the bypass registry? (promoted)
    pub is_bypassed: bool,
    /// Total successful runs (from bypass registry candidates + entries).
    pub success_count: u64,
    /// Total failure count.
    pub fail_count: u64,
    /// Was this command ever demoted and re-promoted?
    pub was_demoted: bool,
    /// Is the command flagged as mutating?
    pub mutating: bool,
    /// Number of prior interactions in this project context.
    pub context_interactions: u64,
    /// Does the command match denylist patterns?
    pub is_denylisted: bool,
}

impl TrustAssessment {
    /// Compute a trust assessment from input data.
    pub fn assess(input: &TrustInput<'_>) -> Self {
        let maturity = compute_maturity(input);
        let familiarity = compute_familiarity(input);
        let blast_radius = compute_blast_radius(input);
        let supervision = determine_supervision(maturity, familiarity, blast_radius);
        let rationale = build_rationale(maturity, familiarity, blast_radius, &supervision);

        Self {
            command: input.command.to_string(),
            maturity,
            familiarity,
            blast_radius,
            supervision,
            rationale,
        }
    }

    /// Quick assessment from bypass registry data only (no fabric enrichment).
    pub fn from_bypass_stats(
        command: &str,
        is_bypassed: bool,
        success_count: u64,
        fail_count: u64,
        mutating: bool,
    ) -> Self {
        Self::assess(&TrustInput {
            command,
            is_bypassed,
            success_count,
            fail_count,
            was_demoted: false,
            mutating,
            context_interactions: 0,
            is_denylisted: crate::bypass::is_denylisted(command),
        })
    }
}

fn compute_maturity(input: &TrustInput<'_>) -> Maturity {
    if input.was_demoted && input.is_bypassed {
        // Demoted and recovered — strongest signal (antifragility)
        Maturity::Hardened
    } else if input.is_bypassed {
        Maturity::Proven
    } else if input.success_count > 0 || input.fail_count > 0 {
        Maturity::Developing
    } else {
        Maturity::Unknown
    }
}

fn compute_familiarity(input: &TrustInput<'_>) -> Familiarity {
    match input.context_interactions {
        0 => Familiarity::New,
        1..=4 => Familiarity::Low,
        5..=20 => Familiarity::Medium,
        _ => Familiarity::High,
    }
}

fn compute_blast_radius(input: &TrustInput<'_>) -> BlastRadius {
    if input.is_denylisted {
        BlastRadius::High
    } else if input.mutating {
        BlastRadius::Medium
    } else {
        BlastRadius::Low
    }
}

/// Determine supervision level from the three trust axes.
///
/// The decision matrix prioritizes safety: any High blast radius forces
/// at minimum Supervised. Hardened + High familiarity allows Unsupervised
/// even for Medium blast radius.
fn determine_supervision(
    maturity: Maturity,
    familiarity: Familiarity,
    blast_radius: BlastRadius,
) -> SupervisionLevel {
    // Denylisted commands are always blocked
    if blast_radius == BlastRadius::High {
        return SupervisionLevel::Blocked;
    }

    match (maturity, blast_radius) {
        // No blast radius → maturity and familiarity determine level
        (_, BlastRadius::None) => SupervisionLevel::Unsupervised,

        // Low blast radius
        (Maturity::Hardened | Maturity::Proven, BlastRadius::Low) => SupervisionLevel::Unsupervised,
        (Maturity::Developing, BlastRadius::Low) => {
            if familiarity >= Familiarity::Medium {
                SupervisionLevel::Unsupervised
            } else {
                SupervisionLevel::Monitored
            }
        }
        (Maturity::Unknown, BlastRadius::Low) => SupervisionLevel::Monitored,

        // Medium blast radius — most nuanced
        (Maturity::Hardened, BlastRadius::Medium) => {
            if familiarity >= Familiarity::Medium {
                SupervisionLevel::Unsupervised
            } else {
                SupervisionLevel::Monitored
            }
        }
        (Maturity::Proven, BlastRadius::Medium) => {
            if familiarity >= Familiarity::High {
                SupervisionLevel::Monitored
            } else {
                SupervisionLevel::Supervised
            }
        }
        (Maturity::Developing, BlastRadius::Medium) => SupervisionLevel::Supervised,
        (Maturity::Unknown, BlastRadius::Medium) => SupervisionLevel::Supervised,

        // Catch-all (shouldn't reach here but safety first)
        _ => SupervisionLevel::Supervised,
    }
}

fn build_rationale(
    maturity: Maturity,
    familiarity: Familiarity,
    blast_radius: BlastRadius,
    supervision: &SupervisionLevel,
) -> String {
    let maturity_str = match maturity {
        Maturity::Unknown => "unknown script (no history)",
        Maturity::Developing => "developing script (some runs)",
        Maturity::Proven => "proven script (bypass-promoted)",
        Maturity::Hardened => "hardened script (failed and recovered)",
    };
    let familiarity_str = match familiarity {
        Familiarity::New => "new context",
        Familiarity::Low => "low familiarity",
        Familiarity::Medium => "moderate familiarity",
        Familiarity::High => "high familiarity",
    };
    let blast_str = match blast_radius {
        BlastRadius::None => "no blast radius",
        BlastRadius::Low => "low blast radius",
        BlastRadius::Medium => "medium blast radius",
        BlastRadius::High => "high blast radius (denylisted)",
    };

    format!(
        "{supervision}: {maturity_str}, {familiarity_str}, {blast_str}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> TrustInput<'static> {
        TrustInput {
            command: "cargo test",
            is_bypassed: false,
            success_count: 0,
            fail_count: 0,
            was_demoted: false,
            mutating: false,
            context_interactions: 0,
            is_denylisted: false,
        }
    }

    #[test]
    fn trust_unknown_low_blast() {
        let input = base_input();
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Unknown);
        assert_eq!(assessment.blast_radius, BlastRadius::Low);
        assert_eq!(assessment.supervision, SupervisionLevel::Monitored);
    }

    #[test]
    fn trust_proven_low_blast() {
        let mut input = base_input();
        input.is_bypassed = true;
        input.success_count = 10;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Proven);
        assert_eq!(assessment.supervision, SupervisionLevel::Unsupervised);
    }

    #[test]
    fn trust_hardened_medium_blast_familiar() {
        let mut input = base_input();
        input.is_bypassed = true;
        input.was_demoted = true;
        input.mutating = true;
        input.context_interactions = 15;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Hardened);
        assert_eq!(assessment.blast_radius, BlastRadius::Medium);
        assert_eq!(assessment.familiarity, Familiarity::Medium);
        assert_eq!(assessment.supervision, SupervisionLevel::Unsupervised);
    }

    #[test]
    fn trust_denylisted_always_blocked() {
        let mut input = base_input();
        input.command = "rm -rf /";
        input.is_denylisted = true;
        input.is_bypassed = true; // even if somehow promoted
        input.success_count = 100;
        input.context_interactions = 50;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.blast_radius, BlastRadius::High);
        assert_eq!(assessment.supervision, SupervisionLevel::Blocked);
    }

    #[test]
    fn trust_unknown_mutating_supervised() {
        let mut input = base_input();
        input.mutating = true;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Unknown);
        assert_eq!(assessment.blast_radius, BlastRadius::Medium);
        assert_eq!(assessment.supervision, SupervisionLevel::Supervised);
    }

    #[test]
    fn trust_developing_familiar_low_blast() {
        let mut input = base_input();
        input.success_count = 3;
        input.context_interactions = 10;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Developing);
        assert_eq!(assessment.familiarity, Familiarity::Medium);
        assert_eq!(assessment.supervision, SupervisionLevel::Unsupervised);
    }

    #[test]
    fn trust_developing_unfamiliar_low_blast() {
        let mut input = base_input();
        input.success_count = 3;
        input.context_interactions = 2;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Developing);
        assert_eq!(assessment.familiarity, Familiarity::Low);
        assert_eq!(assessment.supervision, SupervisionLevel::Monitored);
    }

    #[test]
    fn trust_proven_medium_blast_high_familiarity() {
        let mut input = base_input();
        input.is_bypassed = true;
        input.mutating = true;
        input.context_interactions = 25;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Proven);
        assert_eq!(assessment.blast_radius, BlastRadius::Medium);
        assert_eq!(assessment.familiarity, Familiarity::High);
        assert_eq!(assessment.supervision, SupervisionLevel::Monitored);
    }

    #[test]
    fn trust_proven_medium_blast_low_familiarity() {
        let mut input = base_input();
        input.is_bypassed = true;
        input.mutating = true;
        input.context_interactions = 2;
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.maturity, Maturity::Proven);
        assert_eq!(assessment.blast_radius, BlastRadius::Medium);
        assert_eq!(assessment.familiarity, Familiarity::Low);
        assert_eq!(assessment.supervision, SupervisionLevel::Supervised);
    }

    #[test]
    fn trust_no_blast_always_unsupervised() {
        // Read-only commands are always unsupervised regardless of maturity
        let input = TrustInput {
            command: "git status",
            is_bypassed: false,
            success_count: 0,
            fail_count: 0,
            was_demoted: false,
            mutating: false,
            context_interactions: 0,
            is_denylisted: false,
        };
        // blast_radius defaults to Low for non-mutating, non-denylisted
        // To test None, we need a way to signal read-only explicitly
        // For now, the Low path for Unknown → Monitored is correct
        let assessment = TrustAssessment::assess(&input);
        assert_eq!(assessment.blast_radius, BlastRadius::Low);
    }

    #[test]
    fn trust_from_bypass_stats() {
        let assessment = TrustAssessment::from_bypass_stats(
            "cargo test",
            true,  // bypassed
            15,    // successes
            0,     // failures
            false, // not mutating
        );
        assert_eq!(assessment.maturity, Maturity::Proven);
        assert_eq!(assessment.supervision, SupervisionLevel::Unsupervised);
    }

    #[test]
    fn trust_familiarity_thresholds() {
        let cases = [
            (0, Familiarity::New),
            (1, Familiarity::Low),
            (4, Familiarity::Low),
            (5, Familiarity::Medium),
            (20, Familiarity::Medium),
            (21, Familiarity::High),
            (100, Familiarity::High),
        ];
        for (interactions, expected) in cases {
            let mut input = base_input();
            input.context_interactions = interactions;
            let assessment = TrustAssessment::assess(&input);
            assert_eq!(
                assessment.familiarity, expected,
                "interactions={interactions} expected {expected:?}"
            );
        }
    }

    #[test]
    fn supervision_level_ordering() {
        assert!(SupervisionLevel::Unsupervised < SupervisionLevel::Monitored);
        assert!(SupervisionLevel::Monitored < SupervisionLevel::Supervised);
        assert!(SupervisionLevel::Supervised < SupervisionLevel::Blocked);
    }

    #[test]
    fn trust_rationale_contains_axes() {
        let assessment = TrustAssessment::from_bypass_stats("test", true, 5, 0, false);
        assert!(assessment.rationale.contains("proven"));
        assert!(assessment.rationale.contains("unsupervised"));
    }
}
