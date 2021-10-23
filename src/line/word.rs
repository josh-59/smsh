// This section is very dirty; needs to be rewritten.
// Thinking, Word enum s.t.
// pub enum Word {
//      VariableExpansion(String)
//      EnvironmentExpansion(String)
//      SubshellExpansion(String)
//      PlainString(String)
//      }

use anyhow::Result;
use crate::shell::Shell;

#[derive(Clone, Copy)]
pub enum Quote {
    SingleQuoted,
    DoubleQuoted,
    Unquoted,
}

#[derive(Clone, Copy)]
pub enum Expansion {
    None,
    Variable,
    Environment,
    Subshell,
    Unknown,
}

pub enum Separator {
    None,           // If single- or double-quoted
    Whitespace,     // Default
    Line,           
    Arbitrary(String),
}

pub struct Word {
    pub text: String,
    expansion: Expansion,
    separator: Separator,
}

impl Word {
    fn new(text: String, quote: Quote) -> Result<Word> {
        let expansion = match quote {
            Quote::Unquoted | Quote::DoubleQuoted => {
                get_expansion(&text)
            }
            Quote::SingleQuoted=> {
                Expansion::None
            }
        };

        Ok(Word { text, expansion, separator: Separator::Whitespace })
    }

    pub fn text<'a>(&'a self) -> &'a str {
        &self.text
    }

    pub fn expand(&mut self, smsh: &mut Shell) {
        match self.expansion {
            Expansion::Variable => {
                let mut key = self.text[1..].to_string();
                key.pop();

                if let Some(val) = smsh.get_user_variable(&key) {
                    self.text = val;
                } else {
                    self.text.clear();
                }
            }
            _ => {
            }
        }
    }
}

fn get_expansion(text: &str) -> Expansion {
    if text.len() < 3 {
        Expansion::None
    } else if text.starts_with("{") {
        if text.ends_with("}") {
            Expansion::Variable
        } else {
            Expansion::None
        }
    } else if text.starts_with("!{") {
        if text.ends_with("}") {
            Expansion::Subshell
        } else {
            Expansion::None
        }
    } else if text.starts_with("e{") {
        if text.ends_with("}") {
            Expansion::Environment
        } else {
            Expansion::None
        } 
    } else if text[0..2].contains("{") {
        Expansion::Unknown
    } else {
        Expansion::None
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
                            word_text= String::new();
                        }
                    }
                    '\'' => {
                        quoted = Quote::SingleQuoted;
                    }
                    '\"' => {
                        quoted = Quote::DoubleQuoted;
                    }
                    _ => {
                        word_text.push(ch);
                    }
                }
            }
            Quote::SingleQuoted => {
                if ch == '\'' {
                    words.push(Word::new(word_text, quoted)?);
                    word_text= String::new();
                    quoted = Quote::Unquoted;
                } else {
                    word_text.push(ch);
                }
            }
            Quote::DoubleQuoted => {
                if ch == '\"' {
                    words.push(Word::new(word_text, quoted)?);
                    word_text= String::new();
                    quoted = Quote::Unquoted;
                } else {
                    word_text.push(ch);
                }
            }
        }
    }

    if word_text.len() > 0 {
        words.push(Word::new(word_text, quoted)?);
    }

    Ok(words)
}
