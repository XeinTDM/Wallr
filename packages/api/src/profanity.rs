use rustrict::{Censor, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModerationSeverity {
    Safe,
    Mild,
    Moderate,
    Severe,
}

pub struct ModerationResult {
    pub severity: ModerationSeverity,
    pub flags: Vec<&'static str>,
    pub censored_text: String,
}

pub async fn evaluate_content(text: &str) -> ModerationResult {
    let (censored_text, analysis) = Censor::from_str(text).censor_and_analyze();

    let mut flags = Vec::new();

    let severity = if analysis.is(Type::SEVERE) {
        flags.push("SEVERE_SLUR");
        ModerationSeverity::Severe
    } else if analysis.is(Type::MODERATE) || analysis.is(Type::INAPPROPRIATE) {
        flags.push("INAPPROPRIATE");
        ModerationSeverity::Moderate
    } else if analysis.is(Type::MILD) {
        flags.push("MILD_PROFANITY");
        ModerationSeverity::Mild
    } else {
        ModerationSeverity::Safe
    };

    let lower_text = text.to_lowercase();
    let is_severe_phrase = lower_text.contains("kill yourself")
        || lower_text.contains("killyourself")
        || lower_text.contains("kys");

    if is_severe_phrase {
        flags.push("SELF_HARM");
    }

    ModerationResult {
        severity: if is_severe_phrase {
            ModerationSeverity::Severe
        } else {
            severity
        },
        flags,
        censored_text,
    }
}

pub async fn contains_forbidden_words(text: &str) -> bool {
    let result = evaluate_content(text).await;
    matches!(
        result.severity,
        ModerationSeverity::Severe | ModerationSeverity::Moderate
    )
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profanity_safe() {
        let result = evaluate_content("This is a completely safe and normal sentence.").await;
        assert_eq!(result.severity, ModerationSeverity::Safe);
        assert!(result.flags.is_empty());
        assert!(!contains_forbidden_words("This is a completely safe and normal sentence.").await);
    }

    #[tokio::test]
    async fn test_profanity_mild() {
        // 'crap' is usually considered mild
        let result = evaluate_content("This is crap.").await;
        // The exact severity depends on rustrict, but it should not be Safe and probably not Severe
        assert_ne!(result.severity, ModerationSeverity::Safe);
    }

    #[tokio::test]
    async fn test_self_harm_filter() {
        let result = evaluate_content("You should kill yourself").await;
        assert_eq!(result.severity, ModerationSeverity::Severe);
        assert!(result.flags.contains(&"SELF_HARM"));
        assert!(contains_forbidden_words("You should kill yourself").await);

        let result2 = evaluate_content("kys right now").await;
        assert_eq!(result2.severity, ModerationSeverity::Severe);
        assert!(result2.flags.contains(&"SELF_HARM"));
    }
}

