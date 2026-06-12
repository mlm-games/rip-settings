use crate::field::ValidationRules;

#[derive(Clone, Debug, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

pub fn validate_value(value: &serde_json::Value, rules: &ValidationRules) -> ValidationResult {
    if rules.required {
        let is_empty = match value {
            serde_json::Value::Null => true,
            serde_json::Value::String(s) => s.trim().is_empty(),
            serde_json::Value::Array(a) => a.is_empty(),
            _ => false,
        };
        if is_empty {
            return ValidationResult::Invalid(
                rules.error_message.clone().unwrap_or_else(|| "This field is required".into()),
            );
        }
    }

    if let Some((min, max)) = rules.range {
        if let Some(num) = value.as_f64() {
            if num < min || num > max {
                return ValidationResult::Invalid(
                    rules.error_message.clone().unwrap_or_else(|| {
                        format!("Value must be between {} and {}", min, max)
                    }),
                );
            }
        }
    }

    if let Some((min_len, max_len)) = rules.length {
        if let Some(s) = value.as_str() {
            let len = s.chars().count();
            if len < min_len || len > max_len {
                return ValidationResult::Invalid(
                    rules.error_message.clone().unwrap_or_else(|| {
                        format!("Length must be between {} and {}", min_len, max_len)
                    }),
                );
            }
        }
    }

    if let Some(ref pattern) = rules.pattern {
        if let Some(s) = value.as_str() {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(s) {
                        return ValidationResult::Invalid(
                            rules.error_message.clone().unwrap_or_else(|| {
                                "Value does not match required pattern".into()
                            }),
                        );
                    }
                }
                Err(e) => {
                    return ValidationResult::Invalid(format!("Invalid validation pattern: {}", e));
                }
            }
        }
    }

    ValidationResult::Valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validation() {
        let rules = ValidationRules {
            required: true,
            ..Default::default()
        };

        assert_eq!(
            validate_value(&serde_json::Value::Null, &rules),
            ValidationResult::Invalid("This field is required".into())
        );

        assert_eq!(
            validate_value(&serde_json::json!("hello"), &rules),
            ValidationResult::Valid
        );

        assert_eq!(
            validate_value(&serde_json::json!(""), &rules),
            ValidationResult::Invalid("This field is required".into())
        );
    }

    #[test]
    fn test_range_validation() {
        let rules = ValidationRules {
            range: Some((0.0, 100.0)),
            ..Default::default()
        };

        assert_eq!(
            validate_value(&serde_json::json!(50), &rules),
            ValidationResult::Valid
        );
        assert_eq!(
            validate_value(&serde_json::json!(150), &rules),
            ValidationResult::Invalid("Value must be between 0 and 100".into())
        );
        assert_eq!(
            validate_value(&serde_json::json!(-1), &rules),
            ValidationResult::Invalid("Value must be between 0 and 100".into())
        );
    }

    #[test]
    fn test_length_validation_char_count() {
        let rules = ValidationRules {
            length: Some((3, 10)),
            ..Default::default()
        };

        assert_eq!(
            validate_value(&serde_json::json!("hello"), &rules),
            ValidationResult::Valid
        );
        assert_eq!(
            validate_value(&serde_json::json!("hi"), &rules),
            ValidationResult::Invalid("Length must be between 3 and 10".into())
        );
        assert_eq!(
            validate_value(&serde_json::json!("🎉"), &rules),
            ValidationResult::Invalid("Length must be between 3 and 10".into())
        );
        assert_eq!(
            validate_value(&serde_json::json!("café"), &rules),
            ValidationResult::Valid
        );
    }

    #[test]
    fn test_pattern_validation() {
        let rules = ValidationRules {
            pattern: Some(r"^\d{3}-\d{4}$".into()),
            ..Default::default()
        };

        assert_eq!(
            validate_value(&serde_json::json!("123-4567"), &rules),
            ValidationResult::Valid
        );
        assert_eq!(
            validate_value(&serde_json::json!("abc"), &rules),
            ValidationResult::Invalid("Value does not match required pattern".into())
        );
    }

    #[test]
    fn test_custom_error_message() {
        let rules = ValidationRules {
            range: Some((0.0, 10.0)),
            error_message: Some("Custom error".into()),
            ..Default::default()
        };

        assert_eq!(
            validate_value(&serde_json::json!(20), &rules),
            ValidationResult::Invalid("Custom error".into())
        );
    }
}
