use crate::FailureSignature;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReproductionStatus {
    Unconfirmed,
    PartiallyReproduced,
    StronglyReproduced,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureMatch {
    pub error_type_match: bool,
    pub message_match: bool,
    pub score: f32,
    pub status: ReproductionStatus,
}

pub fn compare_failure_signatures(
    production: &FailureSignature,
    reproduction: &FailureSignature,
) -> FailureMatch {
    let error_type_match = match (&production.error_type, &reproduction.error_type) {
        (Some(production_type), Some(reproduction_type)) => {
            normalize(production_type) == normalize(reproduction_type)
        }
        _ => false,
    };

    let message_match = match (&production.message, &reproduction.message) {
        (Some(production_message), Some(reproduction_message)) => {
            let production_message = normalize(production_message);

            let reproduction_message = normalize(reproduction_message);

            production_message == reproduction_message
                || reproduction_message.contains(&production_message)
                || production_message.contains(&reproduction_message)
        }
        _ => false,
    };

    let score = match (error_type_match, message_match) {
        (true, true) => 1.0,
        (true, false) => 0.4,
        (false, true) => 0.6,
        (false, false) => 0.0,
    };

    let status = match (error_type_match, message_match) {
        (true, true) => ReproductionStatus::StronglyReproduced,

        (true, false) | (false, true) => ReproductionStatus::PartiallyReproduced,

        (false, false) => ReproductionStatus::Unconfirmed,
    };

    FailureMatch {
        error_type_match,
        message_match,
        score,
        status,
    }
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(error_type: &str, message: &str) -> FailureSignature {
        FailureSignature {
            error_type: Some(error_type.to_owned()),

            message: Some(message.to_owned()),

            file: None,
            line: None,
            column: None,
            stack_frames: Vec::new(),
        }
    }

    #[test]
    fn strongly_matches_same_failure() {
        let production = failure("panic", "assertion `left == right` failed");

        let reproduction = failure("panic", "assertion `left == right` failed");

        let result = compare_failure_signatures(&production, &reproduction);

        assert_eq!(result.status, ReproductionStatus::StronglyReproduced);

        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn partially_matches_same_error_type() {
        let production = failure("panic", "original panic");

        let reproduction = failure("panic", "different panic");

        let result = compare_failure_signatures(&production, &reproduction);

        assert_eq!(result.status, ReproductionStatus::PartiallyReproduced);
    }

    #[test]
    fn does_not_match_unrelated_failure() {
        let production = failure("panic", "assertion failed");

        let reproduction = failure("compile_error", "cannot find function");

        let result = compare_failure_signatures(&production, &reproduction);

        assert_eq!(result.status, ReproductionStatus::Unconfirmed);
    }
}
