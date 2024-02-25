use reedline::{Validator, ValidationResult};
use crate::sources::is_complete;

pub struct SmshLineValidator;

impl Validator for SmshLineValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if ! is_complete(line) {    // Every line/line+block must be complete
            ValidationResult::Incomplete
        } else if is_shell_construct(line) {
            if contains_finished_block(line) {
                ValidationResult::Complete
            } else {
                ValidationResult::Incomplete
            }
        } else {
            ValidationResult::Complete
        }
    }
}

// A block is complete if it ends with a line
// of indentation 0
fn contains_finished_block(line: &str) -> bool {
    line.ends_with("\n\n") ||
    line.ends_with("\n \n") ||
    line.ends_with("\n  \n") ||
    line.ends_with("\n   \n")
}

// TODO: Handle leading whitespace
fn is_shell_construct(line: &str) -> bool {
    line.starts_with("if") ||
    line.starts_with("for") ||
    line.starts_with("while") ||
    line.starts_with("case") ||
    line.starts_with("fn")
}
