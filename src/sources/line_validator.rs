use reedline::{Validator, ValidationResult};

pub struct SmshLineValidator;

impl Validator for SmshLineValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let mut single_quoted = false;
        let mut double_quoted = false;
        let mut escaped = false;

        for ch in line.chars() {
            if escaped {
                escaped = false;
            } else {
                match ch {
                    '\\' => {
                        escaped = true;
                    }
                    '\'' => {
                        single_quoted = !single_quoted;
                    }
                    '\"' => {
                        double_quoted = !double_quoted;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }

        if !(single_quoted || double_quoted || escaped) {
            ValidationResult::Complete 
        } else {
            ValidationResult::Incomplete
        }
    }
}
