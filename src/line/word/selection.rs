use anyhow::{anyhow, Result};

use super::Selection;

enum State {
    OnFirstNum,
    FoundFirstPeriod,
    FoundSecondPeriod,
    Invalid
}

// Returns text with selector removed.
pub fn get_selection(text: &str) -> Result<(String, Selection)> {
    if let Some((text, selector)) = get_selector(text) {
        let selection = _get_selection(selector)?;
        Ok((text, selection))
    } else {
        Ok((text.to_string(), Selection::All))
    }
}

// Returns (text, selector)
// Selector does not include braces
fn get_selector(text: &str) -> Option<(String, String)> {
    if text.ends_with("]") {
        if let Some(brace_index) = text.rfind("[") {
            let selector: String = text[brace_index + 1 .. text.len() - 1].to_string();
            let text: String = text[..brace_index].to_string();

            Some((text, selector))
        } else {
            None
        }
    } else {
        None
    }
}

fn _get_selection(selector: String) -> Result<Selection> {
    let mut first_num: usize = 0;
    let mut second_num: usize = 0;
    let mut state = State::OnFirstNum;

    for ch in selector.chars() {
        match state {
            State::OnFirstNum => {
                if ch.is_ascii_digit() {
                    first_num *= 10;
                    first_num += ch.to_digit(10).unwrap() as usize;
                } else if ch == '.' {
                    state = State::FoundFirstPeriod;
                } else {
                    state = State::Invalid;
                }
            }
            State::FoundFirstPeriod => {
                if ch == '.' {
                    state = State::FoundSecondPeriod;
                } else {
                    state = State::Invalid;
                }
            }
            State::FoundSecondPeriod => {
                if ch.is_ascii_digit() {
                    second_num *= 10;
                    second_num += ch.to_digit(10).unwrap() as usize;
                } else {
                    state = State::Invalid;
                }
            }
            State::Invalid => {
                break
            }
        }
    }

    match state {
        State::OnFirstNum => {
            Ok(Selection::Index(first_num))
        }
        State::FoundSecondPeriod => {
            Ok(Selection::Slice(first_num, second_num))
        }
        State::Invalid | State::FoundFirstPeriod => {
            Err(anyhow!("Invalid selection [{}]", selector))
        }
    }
}
