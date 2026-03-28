use crate::verification_history::VerificationSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryLevel {
    Clear,
    Retrying,
    Stuck,
    Escalating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryAssessment {
    pub passed_attempts: usize,
    pub failed_attempts: usize,
    pub failure_streak: usize,
    pub level: RetryLevel,
}

pub fn assess(summary: &VerificationSummary) -> RetryAssessment {
    let level = match summary.failure_streak {
        0 => RetryLevel::Clear,
        1 => RetryLevel::Retrying,
        2 => RetryLevel::Stuck,
        _ => RetryLevel::Escalating,
    };

    RetryAssessment {
        passed_attempts: summary.passed_attempts,
        failed_attempts: summary.failed_attempts,
        failure_streak: summary.failure_streak,
        level,
    }
}

impl RetryAssessment {
    pub fn label(self) -> String {
        match self.level {
            RetryLevel::Clear => "clear".to_string(),
            RetryLevel::Retrying => {
                format!(
                    "retrying after {} failed verification{}",
                    self.failure_streak,
                    pluralize(self.failure_streak)
                )
            }
            RetryLevel::Stuck => {
                format!(
                    "likely stuck after {} consecutive failed verification{}",
                    self.failure_streak,
                    pluralize(self.failure_streak)
                )
            }
            RetryLevel::Escalating => {
                format!(
                    "milestone may be too large or unclear after {} consecutive failed verification{}",
                    self.failure_streak,
                    pluralize(self.failure_streak)
                )
            }
        }
    }

    pub fn should_suggest_explain(self) -> bool {
        self.failure_streak >= 1
    }

    pub fn should_surface_if_stuck(self) -> bool {
        self.failure_streak >= 2
    }

    pub fn should_flag_scope_risk(self) -> bool {
        self.failure_streak >= 3
    }
}

fn pluralize(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::{RetryLevel, assess};
    use crate::verification_history::VerificationSummary;

    fn summary(
        failed_attempts: usize,
        passed_attempts: usize,
        failure_streak: usize,
    ) -> VerificationSummary {
        VerificationSummary {
            attempts: failed_attempts + passed_attempts,
            passed_attempts,
            failed_attempts,
            failure_streak,
            last: None,
        }
    }

    #[test]
    fn assess_returns_retrying_after_one_failure() {
        let assessment = assess(&summary(1, 0, 1));
        assert_eq!(assessment.level, RetryLevel::Retrying);
        assert_eq!(assessment.label(), "retrying after 1 failed verification");
        assert!(assessment.should_suggest_explain());
        assert!(!assessment.should_surface_if_stuck());
    }

    #[test]
    fn assess_returns_stuck_after_two_failures() {
        let assessment = assess(&summary(2, 0, 2));
        assert_eq!(assessment.level, RetryLevel::Stuck);
        assert_eq!(
            assessment.label(),
            "likely stuck after 2 consecutive failed verifications"
        );
        assert!(assessment.should_surface_if_stuck());
        assert!(!assessment.should_flag_scope_risk());
    }

    #[test]
    fn assess_returns_escalating_after_three_failures() {
        let assessment = assess(&summary(3, 0, 3));
        assert_eq!(assessment.level, RetryLevel::Escalating);
        assert_eq!(
            assessment.label(),
            "milestone may be too large or unclear after 3 consecutive failed verifications"
        );
        assert!(assessment.should_flag_scope_risk());
    }

    #[test]
    fn assess_returns_clear_after_success_resets_streak() {
        let assessment = assess(&summary(2, 1, 0));
        assert_eq!(assessment.level, RetryLevel::Clear);
        assert_eq!(assessment.label(), "clear");
        assert!(!assessment.should_suggest_explain());
    }
}
