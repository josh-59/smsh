use reedline::{Validator, ValidationResult};

pub struct SmshLineValidator;

impl Validator for SmshLineValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if contains_unmatched_quote(line) {
            ValidationResult::Incomplete
        } else if is_unfinished_shell_construct(line) {
            ValidationResult::Incomplete
        } else if is_unfinished_block(line) {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

// TODO: This should verify that the indentation is correct
// (wrt leading indentation, if any)
fn is_unfinished_block(line: &str) -> bool {
    line.contains(":") && !line.ends_with("\n")
}

fn is_unfinished_shell_construct(line: &str) -> bool {
    line.ends_with(":")
}

fn contains_unmatched_quote(line: &str) -> bool {
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

    if single_quoted || double_quoted || escaped {
        true
    } else {
        false
    }
}
