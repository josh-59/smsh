//        Allow negative indices (definitely)
//      This should be redone, making gratuitous use of enums and helper functions.
//

use anyhow::{anyhow, Result};

// TODO: Slice(Option<usize>, Option<usize>)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    All,
    Index(usize),
    Slice(usize, usize), // Omitted indices are represented by value zero.
}

// Returns text with selector removed.
pub fn get_selection(text: &str) -> Result<(String, Selection)> {
    if let Some((text, selection_text)) = get_selector(text) {
        let selection = determine_selection(selection_text.as_str())?;
        Ok((text, selection))
    } else {
        Ok((text.to_string(), Selection::All))
    }
}

// Returns (text, selector)
// Selector does not include braces
// Returns 'None' is selector is empty.
fn get_selector(text: &str) -> Option<(String, String)> {
    if text.ends_with(']') {
        if let Some(brace_index) = text.rfind('[') {
            let selector: String = text[brace_index + 1..text.len() - 1].to_string();
            let text: String = text[..brace_index].to_string();

            if selector.is_empty() {
                None
            } else {
                Some((text, selector))
            }
        } else {
            None
        }
    } else {
        None
    }
}

// selector is an &str of one of the following forms:
// a..b
// a..
// ..b
// a
// where a, b are integers
fn determine_selection(selection_text: &str) -> Result<Selection> {
    enum State {
        OnFirstNum,
        FoundFirstPeriod,
        FoundSecondPeriod,
        Invalid,
    }

    let mut first_num: usize = 0;
    let mut second_num: usize = 0;
    let mut state = State::OnFirstNum;

    // TODO:  Change to selector.graphemes?  
    for ch in selection_text.chars() {
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
            State::Invalid => break,
        }
    }

    match state {
        State::OnFirstNum => Ok(Selection::Index(first_num)),
        State::FoundSecondPeriod => Ok(Selection::Slice(first_num, second_num)),
        State::Invalid | State::FoundFirstPeriod => {
            Err(anyhow!("Invalid selection [{}]", selection_text))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_selector_test_some_1() {
        let (text, selector) = get_selector("{cmds}[0]").unwrap();
        assert_eq!(text, "{cmds}".to_string());
        assert_eq!(selector, "0".to_string());
    }

    #[test]
    fn get_selector_test_some_2() {
        let (text, selector) = get_selector("{cmds}[0..]").unwrap();
        assert_eq!(text, "{cmds}".to_string());
        assert_eq!(selector, "0..".to_string());
    }

    #[test]
    fn get_selector_test_some_3() {
        let (text, selector) = get_selector("{cmds}[0..2]").unwrap();
        assert_eq!(text, "{cmds}".to_string());
        assert_eq!(selector, "0..2".to_string());
    }

    #[test]
    fn get_selector_test_some_4() {
        let (text, selector) = get_selector("{cmds}[..2]").unwrap();
        assert_eq!(text, "{cmds}".to_string());
        assert_eq!(selector, "..2".to_string());
    }

    #[test]
    fn get_selector_test_none_1() {
        assert_eq!(None, get_selector("{cmds}"));
    }

    #[test]
    fn get_selector_test_none_2() {
        assert_eq!(None, get_selector("{cmds}["));
    }

    #[test]
    fn get_selector_test_none_3() {
        assert_eq!(None, get_selector("{cmds}]"));
    }

    #[test]
    fn get_selector_test_none_4() {
        assert_eq!(None, get_selector("{cmds}[]"));
    }

    #[test]
    fn determine_selection_1() {
        assert_eq!(
            Selection::Index(0),
            determine_selection("0").unwrap()
        );
    }

    #[test]
    fn determine_selection_2() {
        assert_eq!(
            Selection::Index(10),
            determine_selection("10").unwrap()
        );
    }

    #[test]
    fn determine_selection_3() {
        assert_eq!(
            Selection::Slice(0, 5),
            determine_selection("0..5").unwrap()
        );
    }

    #[test]
    fn determine_selection_4() {
        assert_eq!(
            Selection::Slice(0, 10),
            determine_selection("0..10").unwrap()
        );
    }

    #[test]
    fn determine_selection_5() {
        assert_eq!(
            Selection::Slice(10, 10),
            determine_selection("10..10").unwrap()
        );
    }

    #[test]
    fn determine_selection_6() {
        assert_eq!(
            Selection::Slice(0, 10),
            determine_selection("..10").unwrap()
        );
    }

    #[test]
    fn determine_selection_7() {
        assert_eq!(
            Selection::Slice(3, 0),
            determine_selection("3..").unwrap()
        );
    }

    #[test]
    fn determine_selection_8() {
        assert!(determine_selection("1...10").is_err());
    }

    #[test]
    fn determine_selection_9() {
        assert!(determine_selection("...10").is_err());
    }

    #[test]
    fn determine_selection_10() {
        assert!(determine_selection("10...").is_err());
    }

    #[test]
    fn determine_selection_11() {
        assert!(determine_selection("a...b").is_err());
    }
}
