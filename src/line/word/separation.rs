use super::Separator;

// text: A single word
pub fn get_separator(text: &str) -> (String, Separator) {
    enum State {
        FoundTrailingDoubleQuote,
        CollectingSeparator,
        FoundLeadingDoubleQuote,
        FoundEqualsSign,
        Complete,
        Invalid
    }

    if text.contains("S=") && text.ends_with('\"') {
        let mut state = State::FoundTrailingDoubleQuote;
        let mut separator_text = Vec::<char>::new();

        for (idx, ch) in text.char_indices().rev() {
            match state {
                State::FoundTrailingDoubleQuote => {
                    state = State::CollectingSeparator;
                    continue;
                }
                State::CollectingSeparator => {
                    if ch == '\"' && !separator_text.is_empty() {
                        state = State::FoundLeadingDoubleQuote
                    } else if ch == '\"' {
                        state = State::Invalid;
                    } else {
                        separator_text.push(ch);
                    }
                }
                State::FoundLeadingDoubleQuote => {
                    if ch == '=' {
                        state = State::FoundEqualsSign;
                    } else {
                        state = State::Invalid;
                    }
                } 
                State::FoundEqualsSign => {
                    if ch == 'S' {
                        state = State::Complete;
                    }  else {
                        state = State::Invalid;
                    }
                }
                State::Complete => {
                    let mut temp = String::new();

                    for ch in separator_text.iter().rev() {
                        temp.push(*ch);
                    }

                    return (text[0..idx+1].to_string(), Separator::Arbitrary(temp));
                }
                State::Invalid => {
                    break;
                }
            }
        }

        (text.to_string(), Separator::Whitespace)
    } else {
        (text.to_string(), Separator::Whitespace)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_separator_test_1() {
        let text = "echo";

        assert_eq!(
            ("echo".to_string(), Separator::Whitespace), 
            get_separator(text)
        );
    }

    #[test]
    fn get_separator_test_2() {
        let text = "e{PATH}S=\":\"";

        assert_eq!(
            ("e{PATH}".to_string(), Separator::Arbitrary(":".to_string())), 
            get_separator(text)
        );
    }
}
