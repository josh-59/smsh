use anyhow::{anyhow, Result};
use crate::shell::Shell;

mod selection;
use selection::get_selection;
mod expansion;
use expansion::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quote {
    SingleQuoted,
    DoubleQuoted,
    Unquoted,
    Expansion(usize),
}

#[derive(Clone, Copy)]
pub enum Expansion {
    None,
    Variable,
    Environment,
    Subshell,
    Unknown,
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Word {
    text: String,
    expansion: Expansion,
    separator: Separator,
    selection: Selection,
}

// A word is a single logical unit of raw text.
// Words expand themselves (wrt a given shell).
impl Word {
    fn new(text: String, quote: Quote) -> Result<Word> {
        let (text, selection) = get_selection(&text)?;

        // Debug printing
        match selection {
            Selection::Index(n) => {
                eprintln!("Selected INDEX {}; got word {}", n, text);
            } 
            Selection::Slice(n, m) => {
                eprintln!("Selected SLICE {} to {}; got word {}", n, m, text);
            }
            _ => {}
        }

        let expansion = match quote {
            Quote::SingleQuoted => {
                Expansion::None
            }
            _ => {
                get_expansion(&text)
            }
        };

        Ok(Word { text, expansion, separator: Separator::Whitespace, selection })
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

pub fn get_words_from_str(line: &str) -> Result<Vec<Word>> {
    let mut words = Vec::<Word>::new();
    let mut word_text = String::new();

    let mut quoted = Quote::Unquoted;

    for ch in line.chars() {
        match quoted {
            Quote::Unquoted => {
                match ch {
                    ' ' | '\n' => {
                        if word_text.len() > 0 {
                            words.push(Word::new(word_text, quoted)?);
                            word_text = String::new();
                        }
                    }
                    '\'' => {
                        quoted = Quote::SingleQuoted;
                    }
                    '\"' => {
                        quoted = Quote::DoubleQuoted;
                    }
                    '{' => {
                        word_text.push(ch);
                        quoted = Quote::Expansion(1);
                    }
                    _ => {
                        word_text.push(ch);
                    }
                }
            }
            Quote::SingleQuoted => {
                if ch == '\'' {
                    words.push(Word::new(word_text, quoted)?);
                    word_text = String::new();
                    quoted = Quote::Unquoted;
                } else {
                    word_text.push(ch);
                }
            }
            Quote::DoubleQuoted => {
                if ch == '\"' {
                    words.push(Word::new(word_text, quoted)?);
                    word_text = String::new();
                    quoted = Quote::Unquoted;
                } else {
                    word_text.push(ch);
                }
            }
            Quote::Expansion(n) => {
                if ch == '{' {
                    quoted = Quote::Expansion(n+1);
                } else if ch == '}' {
                    quoted = Quote::Expansion(n-1);
                } 

                word_text.push(ch);

                if quoted == Quote::Expansion(0) {
                    quoted = Quote::Unquoted;
                }
            }
        }
    }

    if word_text.len() > 0 {
        words.push(Word::new(word_text, quoted)?);
    }

    match quoted {
        Quote::SingleQuoted => {
            Err(anyhow!("Unmatched single quote (This should not happen)."))
        } 
        Quote::DoubleQuoted => {
            Err(anyhow!("Unmatched double quote (This should not happen)."))
        } 
        Quote::Expansion(_) => {
            Err(anyhow!("Improperly formed text replacement"))
        } 
        Quote::Unquoted => {
            Ok(words)
        }
    }

}
