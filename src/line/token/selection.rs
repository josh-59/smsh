//        Allow negative indices (definitely)
//      This should be redone, making gratuitous use of enums and helper functions.
//

use anyhow::{anyhow, Result};

// TODO: Slice(Option<usize>, Option<usize>)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    All,
    None,
    Index(usize),
    Slice(usize, usize),
    GreaterThan(usize),
    LessThan(usize),
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
// Returns 'None' if selector is empty.
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

// If a and b are integers, this function maps &str `selection_text`
// as follows:
// a..b => Selection::Slice(a, b)
// a.. => Selection::GreaterThan(a)
// ..b => Selection::LessThan(b)
// a => Selection::Index(a)
//   => Selection::None
// xxxx => Selection::Invalid
// TODO: Perhaps decompose into get_first_num_of_selector(&str),
// get_dot_dot_of_selector(&str, index), get_second_num_of_selector(&str, index),
fn determine_selection(selection_text: &str) -> Result<Selection> {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum State {
        EnteringLoop,
        OnFirstNum,
        FoundFirstPeriod,
        FoundSecondPeriod,
        Invalid,
    }

    let mut state = State::EnteringLoop;
    let mut first_num: Option<usize> = None;
    let mut second_num: Option<usize> = None;

    for ch in selection_text.chars() {
        match state {
            State::EnteringLoop => {
                if ch == '.' {
                    state = State::FoundFirstPeriod;
                } else if ch.is_ascii_digit() {
                    first_num = Some(ch.to_digit(10).unwrap() as usize);
                    state = State::OnFirstNum;
                } else {
                    state = State::Invalid;
                    break;
                }
            }
            State::OnFirstNum => {
                if ch.is_ascii_digit() {
                    let mut temp = first_num.unwrap();
                    temp *= 10;
                    temp += ch.to_digit(10).unwrap() as usize;
                    first_num = Some(temp);
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
                    if let Some(mut temp) = second_num {
                        temp *= 10;
                        temp += ch.to_digit(10).unwrap() as usize;
                        second_num = Some(temp);
                    } else {
                        second_num = Some(ch.to_digit(10).unwrap() as usize);
                    }
                } else {
                    state = State::Invalid;
                }
            }
            State::Invalid => break,
        }
    }

    if state == State::Invalid {
        Err(anyhow!("Invalid selection {}", selection_text))
    } else if let Some(first_num) = first_num {
        if let Some(second_num) = second_num {
            Ok(Selection::Slice(first_num, second_num))
        } else if state == State::FoundSecondPeriod {
            Ok(Selection::GreaterThan(first_num))
        } else {
            Ok(Selection::Index(first_num))
        }
    } else if let Some(second_num) = second_num {
        Ok(Selection::LessThan(second_num))
    } else {
        Ok(Selection::None)
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
        assert_eq!(Selection::Index(0), determine_selection("0").unwrap());
    }

    #[test]
    fn determine_selection_2() {
        assert_eq!(Selection::Index(10), determine_selection("10").unwrap());
    }

    #[test]
    fn determine_selection_3() {
        assert_eq!(Selection::Slice(0, 5), determine_selection("0..5").unwrap());
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
            Selection::LessThan(10),
            determine_selection("..10").unwrap()
        );
    }

    #[test]
    fn determine_selection_7() {
        assert_eq!(
            Selection::GreaterThan(3),
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
