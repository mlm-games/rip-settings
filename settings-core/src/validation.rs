//! Runtime validation of setting values.

use crate::field::ValidationRules;

/// Result of validating a setting value.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Validate a `serde_json::Value` against validation rules.
pub fn validate_value(value: &serde_json::Value, rules: &ValidationRules) -> ValidationResult {
    // Required check
    if rules.required {
        let is_empty = match value {
            serde_json::Value::Null => true,
            serde_json::Value::String(s) => s.trim().is_empty(),
            serde_json::Value::Array(a) => a.is_empty(),
            _ => false,
        };
        if is_empty {
            return ValidationResult::Invalid(if rules.error_message.is_empty() {
                "This field is required".into()
            } else {
                rules.error_message.clone()
            });
        }
    }

    // Range check
    if let Some((min, max)) = rules.range {
        if let Some(num) = value.as_f64() {
            if num < min || num > max {
                return ValidationResult::Invalid(if rules.error_message.is_empty() {
                    format!("Value must be between {} and {}", min, max)
                } else {
                    rules.error_message.clone()
                });
            }
        }
    }

    // Length check
    if let Some((min_len, max_len)) = rules.length {
        if let Some(s) = value.as_str() {
            let len = s.len();
            if len < min_len || len > max_len {
                return ValidationResult::Invalid(if rules.error_message.is_empty() {
                    format!("Length must be between {} and {}", min_len, max_len)
                } else {
                    rules.error_message.clone()
                });
            }
        }
    }

    // Pattern check
    if let Some(ref pattern) = rules.pattern {
        if let Some(s) = value.as_str() {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(s) {
                        return ValidationResult::Invalid(if rules.error_message.is_empty() {
                            "Value does not match required pattern".into()
                        } else {
                            rules.error_message.clone()
                        });
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
    fn test_length_validation() {
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
            error_message: "Custom error".into(),
            ..Default::default()
        };

        assert_eq!(
            validate_value(&serde_json::json!(20), &rules),
            ValidationResult::Invalid("Custom error".into())
        );
    }
}
