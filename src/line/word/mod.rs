use anyhow::{anyhow, Result};
use crate::shell::Shell;

mod selection;
use selection::get_selection;
mod expansion;
use expansion::*;

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
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Separator {
    None,           // If single- or double-quoted
    Whitespace,     // Default
    Line,           
    Arbitrary(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    All,
    Index(usize),
    Slice(usize, usize),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Word {
    text: String,
    expansion: Expansion,
    separator: Separator,
    selection: Selection,
}

// A word is a single logical unit of raw text.
// Words expand themselves (wrt a given shell).
impl Word {
    pub fn new(text: String) -> Result<Word> {

        let (text, quote)  = get_quote(&text)?;

        let (text, selection) = get_selection(&text)?;

        let (text, expansion) = match quote {
            Quote::SingleQuoted => {
                (text, Expansion::None)
            }
            _ => {
                get_expansion(&text)
            }
        };

        let word = Word {
            text,
            expansion,
            separator: Separator::Whitespace,
            selection
        };

        Ok(word)
    }

    pub fn text<'a>(&'a self) -> &'a str {
        &self.text
    }

    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        expand(self, smsh)
    }

    pub fn select(&mut self) -> Result<()>{
        Ok(())
    }

    pub fn separate(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.text().is_empty()
    }
}


fn get_quote(text: &str) -> Result<(String, Quote)> {
    let leading_quote = match text.chars().nth(0) {
        Some('\'') => {
            Quote::SingleQuoted
        }
        Some('\"') => {
            Quote::DoubleQuoted
        }
        _ => {
            Quote::Unquoted
        }
    };


    let trailing_quote = match text.chars().last() {
        Some('\'') => {
            Quote::SingleQuoted
        }
        Some('\"') => {
            Quote::DoubleQuoted
        }
        _ => {
            Quote::Unquoted
        }
    };

    if leading_quote == trailing_quote {
        match leading_quote {
            Quote::SingleQuoted | Quote::DoubleQuoted => {
                let mut s = text[1..].to_string();
                s.pop();
                Ok((s, leading_quote))
            } 
            Quote::Unquoted => {
                Ok((text.to_string(), leading_quote))
            }
        }
    } else {
        Err(anyhow!("Unmatched quote"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn create_word_1() {
        let cmd = "cat".to_string();

        let word = Word {
            text: "cat".to_string(),
            expansion: Expansion::None,
            separator: Separator::Whitespace,
            selection: Selection::All,
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }
 
    #[test]
    fn create_word_2() {
        let cmd = "{cmd}".to_string();

        let word = Word {
            text: "cmd".to_string(),
            expansion: Expansion::Variable,
            separator: Separator::Whitespace,
            selection: Selection::All,
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }
 
    #[test]
    fn create_word_3() {
        let cmd = "!{cmd}".to_string();

        let word = Word {
            text: "cmd".to_string(),
            expansion: Expansion::Subshell,
            separator: Separator::Whitespace,
            selection: Selection::All,
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_4() {
        let cmd = "!{{cmd}}".to_string();

        let word = Word {
            text: "{cmd}".to_string(),
            expansion: Expansion::Subshell,
            separator: Separator::Whitespace,
            selection: Selection::All,
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_5() {
        let cmd = "!{{cmd}}[1]".to_string();

        let word = Word {
            text: "{cmd}".to_string(),
            expansion: Expansion::Subshell,
            separator: Separator::Whitespace,
            selection: Selection::Index(1),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }

    #[test]
    fn create_word_6() {
        let cmd = "!{{cmd}}[1..]".to_string();

        let word = Word {
            text: "{cmd}".to_string(),
            expansion: Expansion::Subshell,
            separator: Separator::Whitespace,
            selection: Selection::Slice(1, 0),
        };

        assert_eq!(word, Word::new(cmd).unwrap());
    }
}
