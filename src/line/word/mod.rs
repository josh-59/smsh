use crate::shell::Shell;
use anyhow::{anyhow, Result};

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
pub struct Word {
    text: String,
    quote: Quote,
    expansion: Expansion,
    selection: Selection,
    separated_text: Vec<String>,
    selected_text: Vec<String>,
}

// A word is a single logical unit of input text.
impl Word {
    pub fn new(text: String) -> Result<Word> {
        let (text, quote) = get_quote(&text)?;

        let (text, selection) = match quote {
            Quote::Unquoted => get_selection(&text)?,
            Quote::SingleQuoted | Quote::DoubleQuoted => (text, Selection::All),
        };

        let (text, expansion) = match quote {
            Quote::SingleQuoted => (text, Expansion::None),
            Quote::Unquoted | Quote::DoubleQuoted => get_expansion(&text),
        };

        let word = Word {
            text,
            quote,
            expansion,
            selection,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        Ok(word)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::shell::Shell;

    #[test]
    fn create_word_1() {
        let cmd = "cat".to_string();

        let word = Word {
            text: "cat".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::None,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_2() {
        let cmd = "{cmd}".to_string();

        let word = Word {
            text: "cmd".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Variable,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_3() {
        let cmd = "!{cmd}".to_string();

        let word = Word {
            text: "cmd".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_4() {
        let cmd = "!{{cmd}}".to_string();

        let word = Word {
            text: "{cmd}".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_5() {
        let cmd = "!{{cmd}}[1]".to_string();

        let word = Word {
            text: "{cmd}".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::Index(1),
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_6() {
        let cmd = "!{{cmd}}[1..]".to_string();

        let word = Word {
            text: "{cmd}".to_string(),
            quote: Quote::Unquoted,
            expansion: Expansion::Subshell,
            selection: Selection::Slice(1, 0),
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_7() {
        let cmd = "'!{{cmd}}[1..]'".to_string();

        let word = Word {
            text: "!{{cmd}}[1..]".to_string(),
            quote: Quote::SingleQuoted,
            expansion: Expansion::None,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
            selected_text: Vec::<String>::new(),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn expand_1() {
        // Replace 'cmd' with 'cat'
        let mut smsh = Shell::new();
        smsh.insert_user_variable("cmd".to_string(), "cat".to_string());

        let mut word = Word::new("{cmd}".to_string()).unwrap();

        word.expand(&mut smsh).unwrap();

        assert_eq!(word.text, "cat".to_string())
    }
}
