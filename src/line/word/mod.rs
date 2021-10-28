use crate::shell::Shell;
use anyhow::{anyhow, Result};

use std::cmp::min;

mod expansion;
use expansion::*;
mod selection;
use selection::get_selection;
mod separation;
use separation::get_separator;

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
    Whitespace,         // Default
    Arbitrary(String),  // S="sep"
    None,               // If single- or double-quoted
}

// TODO: Slice(Option<usize>, Option<usize>)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    All,
    Index(usize),
    Slice(usize, usize), // Omitted indices are represented by value zero. 
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Word {
    text: String,
    expansion: Expansion,
    separator: Separator,
    selection: Selection,
    pub separated_text: Vec<String>,
}

// A word is a single logical unit of input text.
impl Word {
    pub fn new(text: String) -> Result<Word> {

        let (text, selection) = get_selection(&text)?;

        let (text, quote) = get_quote(&text)?;

        let (text, separator) = match quote {
            Quote::SingleQuoted | Quote::DoubleQuoted => {
                (text, Separator::None)
            }
            Quote::Unquoted => {
                get_separator(&text)
            }
        };

        let (text, expansion) = match quote {
            Quote::SingleQuoted => {
                (text, Expansion::None)
            }
            Quote::Unquoted | Quote::DoubleQuoted => {
                get_expansion(&text)
            }
        };

        let word = Word {
            text,
            expansion,
            separator,
            selection,
            separated_text: Vec::<String>::new(),
        };

        Ok(word)
    }

    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        expand(self, smsh)
    }

    pub fn select(&mut self) -> Result<()> {
        match &self.selection {
            Selection::All => {
                Ok(())
            }
            Selection::Index(n) => {
                if *n < self.separated_text.len() {
                    let word = self.separated_text[*n].clone();
                    self.separated_text.clear();
                    self.separated_text.push(word);
                } else {
                    self.separated_text.clear();
                }

                Ok(())
            }
            Selection::Slice(n, m) => {
                if self.separated_text.len() == 0 {
                    Ok(())
                } else if *n < self.separated_text.len() {
                    let mut words = Vec::<String>::new();

                    if *m > *n {
                        let min = min(self.separated_text.len() - 1, *m);

                        for w in &self.separated_text[*n..min] {
                            words.push(w.to_string())
                        }
                    } else if *m == 0 {
                        for w in &self.separated_text[*n..] {
                            words.push(w.to_string())
                        }
                    }

                    self.separated_text = words;

                    Ok(())
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn separate(&mut self) -> Result<()> {
        match &self.separator {
            Separator::Whitespace => {
                let mut s = String::new();

                for ch in self.text.chars() {
                    if ch.is_whitespace() && !s.is_empty() {
                        self.separated_text.push(s);
                        s = String::new();
                    } else {
                        s.push(ch);
                    }
                }

                if !s.is_empty() {
                    self.separated_text.push(s);
                }
                
                Ok(())
            }
            Separator::Arbitrary(s) => {
                self.separated_text = self.text.split(s)
                    .map(|x| x.to_string()).collect();
                Ok(())
            }
            Separator::None => {
                Ok(())
            }
        }
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

    if leading_quote == trailing_quote {
        match leading_quote {
            Quote::SingleQuoted | Quote::DoubleQuoted => {
                let mut s = text[1..].to_string();
                s.pop();
                Ok((s, leading_quote))
            }
            Quote::Unquoted => Ok((text.to_string(), leading_quote)),
        }
    } else {
        Err(anyhow!("Unmatched quote"))
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
            expansion: Expansion::None,
            separator: Separator::Whitespace,
            selection: Selection::All,
            separated_text: Vec::<String>::new(),
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
            separated_text: Vec::<String>::new(),
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
            separated_text: Vec::<String>::new(),
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
            separated_text: Vec::<String>::new(),
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
            separated_text: Vec::<String>::new(),
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
            separated_text: Vec::<String>::new(),
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

    #[test]
    fn separate_1() {
        let cmd = "cat".to_string();
        let mut word = Word::new(cmd).unwrap();

        word.separate().unwrap();

        assert_eq!(word.separated_text, vec!["cat".to_string()])
    }

    #[test]
    fn expand_separate_and_select_1() {
        let mut smsh = Shell::new();

        smsh.insert_user_variable("vec".to_string(), "zero one two three four".to_string());

        let text = "{vec}".to_string();
        let mut word = Word::new(text).unwrap();

        word.expand(&mut smsh).unwrap();
        word.separate().unwrap();

        let res = vec!["zero".to_string(),
                        "one".to_string(),
                        "two".to_string(),
                        "three".to_string(),
                        "four".to_string()];

        assert_eq!(word.separated_text, res);
    }

    #[test]
    fn expand_separate_and_select_2() {
        let mut smsh = Shell::new();

        smsh.insert_user_variable("vec".to_string(), "zero one two three four".to_string());

        let text = "{vec}[0]".to_string();
        let mut word = Word::new(text).unwrap();

        word.expand(&mut smsh).unwrap();
        word.separate().unwrap();
        word.select().unwrap();

        let res = vec!["zero".to_string()];

        assert_eq!(word.separated_text, res);
    }

    #[test]
    fn expand_separate_and_select_3() {
        let mut smsh = Shell::new();

        smsh.insert_user_variable("vec".to_string(), "zero one two three four".to_string());

        let text = "{vec}[2]".to_string();
        let mut word = Word::new(text).unwrap();

        word.expand(&mut smsh).unwrap();
        word.separate().unwrap();
        word.select().unwrap();

        let res = vec!["two".to_string()];

        assert_eq!(word.separated_text, res);
    }

    #[test]
    fn expand_separate_and_select_4() {
        let mut smsh = Shell::new();

        smsh.insert_user_variable("vec".to_string(), "zero one two three four".to_string());

        let text = "{vec}[2..]".to_string();
        let mut word = Word::new(text).unwrap();

        word.expand(&mut smsh).unwrap();
        word.separate().unwrap();
        word.select().unwrap();

        let res = vec!["two".to_string(),
                        "three".to_string(),
                        "four".to_string()];

        assert_eq!(word.separated_text, res);
    }
}
