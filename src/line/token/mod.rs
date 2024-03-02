// A Token is the smallest logical unit of input to `smsh`.  It is created by
// breaking a logical line into sequences of UTF-8 graphemes according to the
// quoting rules.  A token may be a command, or a string of quoted text, or an
// expansion.  Quotes are preserved, and selection is also preserved.

use crate::shell::Shell;
use anyhow::{anyhow, Result};
use unicode_segmentation::UnicodeSegmentation;

use std::cmp::min;

mod expansion;
use expansion::*;
mod selection;
use selection::{get_selection, Selection};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Quote {
    SingleQuoted,
    DoubleQuoted,
    Unquoted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Expansion {
    None,
    Variable,
    Environment,
    Subshell,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    text: String,
    quote: Quote,
    expansion: Expansion,
    selection: Selection,
    separated_text: Vec<String>,
    selected_text: Vec<String>,
}

impl Token {
    pub fn new(text: String) -> Result<Token> {
        let (text, quote) = get_quote(&text)?;

        let (text, selection) = match quote {
            Quote::Unquoted => get_selection(&text)?,
            Quote::SingleQuoted | Quote::DoubleQuoted => (text, Selection::All),
        };

        let (text, expansion) = match quote {
            Quote::SingleQuoted => (text, Expansion::None),
            Quote::Unquoted | Quote::DoubleQuoted => get_expansion(&text),
        };

        let separated_text = vec![text.clone()];

        let token = Token {
            text,
            quote,
            expansion,
            selection,
            separated_text,
            selected_text: Vec::<String>::new(),
        };

        Ok(token)
    }

    // Replaces self.text with expanded value
    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        match &self.quote {
            Quote::Unquoted | Quote::DoubleQuoted => expand(self, smsh),
            Quote::SingleQuoted => Ok(()),
        }
    }

    pub fn is_pipe_operator(&self) -> bool {
        self.selected_text.len() == 1 && self.selected_text[0] == "|"
    }

    pub fn selected_text(&self) -> &Vec<String> {
        &self.selected_text
    }

    pub fn select(&mut self) -> Result<()> {
        match &self.selection {
            Selection::Index(n) => {
                if *n < self.separated_text.len() {
                    let word = self.separated_text[*n].clone();
                    self.selected_text.push(word);
                }
            }
            Selection::Slice(n, m) => {
                if self.separated_text.len() > 0 && *n < self.separated_text.len() {
                    if *m > *n {
                        let min = min(self.separated_text.len() - 1, *m);

                        for w in &self.separated_text[*n..min] {
                            self.selected_text.push(w.to_string());
                        }
                    } else if *m == 0 {
                        for w in &self.separated_text[*n..] {
                            self.selected_text.push(w.to_string());
                        }
                    }
                }
            }
            Selection::All => {
                self.selected_text = self.separated_text.clone();
            }
        }
        Ok(())
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

fn get_quote(text: &str) -> Result<(String, Quote)> {
    let leading_quote = match text.chars().next() {
        Some('\'') => Quote::SingleQuoted,
        Some('\"') => Quote::DoubleQuoted,
        _ => Quote::Unquoted,
    };

    let trailing_quote = match text.chars().last() {
        Some('\'') => Quote::SingleQuoted,
        Some('\"') => Quote::DoubleQuoted,
        _ => Quote::Unquoted,
    };

    match leading_quote {
        Quote::SingleQuoted | Quote::DoubleQuoted => {
            if leading_quote == trailing_quote {
                let mut s = text[1..].to_string();
                s.pop();
                Ok((s, leading_quote))
            } else {
                Err(anyhow!("Unmatched quote"))
            }
        }
        Quote::Unquoted => Ok((text.to_string(), leading_quote)),
    }
}

// Breaks `rawline` into parts according to quoting rules, yielding tokens.
// Quotes and escapes are preserved; unquoted whitespace is removed
// Selection remains appended to part.
pub fn get_tokens(rawline: &str) -> Result<Vec<Token>> {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum State {
        SingleQuoted,
        DoubleQuoted,
        Escaped,
        Unquoted,
    }

    let mut part = String::new();
    let mut tokens = Vec::<Token>::new();

    let mut state = State::Unquoted;
    let mut escaped_state: Option<State> = None;

    for grapheme in rawline.graphemes(true) {
        match state {
            State::Unquoted => match grapheme {
                " " | "\t" => {
                    if !part.is_empty() {
                        tokens.push(Token::new(part)?);
                        part = String::new();
                    }
                }
                "\'" => {
                    part.push_str(grapheme);
                    state = State::SingleQuoted;
                }
                "\"" => {
                    part.push_str(grapheme);
                    state = State::DoubleQuoted;
                }
                "\\" => {
                    part.push_str(grapheme);
                    escaped_state = Some(State::Unquoted);
                    state = State::Escaped;
                }
                _ => {
                    part.push_str(grapheme);
                }
            },
            State::SingleQuoted => {
                part.push_str(grapheme);
                if grapheme == "\'" {
                    state = State::Unquoted;
                }
            }
            State::DoubleQuoted => {
                part.push_str(grapheme);
                if grapheme == "\"" {
                    state = State::Unquoted;
                } else if grapheme == "\\" {
                    escaped_state = Some(State::DoubleQuoted);
                    state = State::Escaped;
                }
            }
            State::Escaped => {
                part.push_str(grapheme);
                state = escaped_state.unwrap();
            }
        }
    }
    if !part.is_empty() {
        tokens.push(Token::new(part)?);
    }

    Ok(tokens)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::shell::Shell;

    #[test]
    fn create_word_1() {
        let cmd = "cat".to_string();

        let word = Token {
            text: "cat".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::None,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn create_word_2() {
        let cmd = "{cmd}".to_string();

        let word = Token {
            text: "cmd".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Variable,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn create_word_3() {
        let cmd = "!{cmd}".to_string();

        let word = Token {
            text: "cmd".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn create_word_4() {
        let cmd = "!{{cmd}}".to_string();

        let word = Token {
            text: "{cmd}".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn create_word_5() {
        let cmd = "!{{cmd}}[1]".to_string();

        let word = Token {
            text: "{cmd}".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::Index(1),
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn create_word_6() {
        let cmd = "!{{cmd}}[1..]".to_string();

        let word = Token {
            text: "{cmd}".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::Slice(1, 0),
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn create_word_7() {
        let cmd = "'!{{cmd}}[1..]'".to_string();

        let word = Token {
            text: "!{{cmd}}[1..]".to_string(),
            quote: Quote::SingleQuoted,
            expansion: Expansion::None,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Token::new(cmd).unwrap());
    }

    #[test]
    fn expand_1() {
        // Replace 'cmd' with 'cat'
        let mut smsh = Shell::new();
        smsh.insert_user_variable("cmd".to_string(), "cat".to_string());

        let mut word = Token::new("{cmd}".to_string()).unwrap();

        word.expand(&mut smsh).unwrap();

        assert_eq!(word.text, "cat".to_string())
    }
}
