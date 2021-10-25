// This section is very dirty; needs to be rewritten.
// Thinking, Word enum s.t.
// pub enum Word {
//      VariableExpansion(String)
//      EnvironmentExpansion(String)
//      SubshellExpansion(String)
//      PlainString(String)
//      }

use anyhow::{anyhow, Result};
use crate::shell::Shell;
use crate::sources::{SourceKind, BufferSource};
use crate::line::Line;

use std::env;
use std::os::unix::io::{RawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

use nix::unistd::{fork, pipe, ForkResult, close, dup2};
use nix::sys::wait::wait;

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


#[derive(Clone)]
pub struct Word {
    text: String,
    expansion: Expansion,
    separator: Separator,
}

impl Word {
    fn new(text: String, quote: Quote) -> Result<Word> {
        // XXX: This is poor...
        let expansion = match quote {
            Quote::Unquoted | Quote::DoubleQuoted | Quote::Expansion(_) => {
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

    pub fn expand(&mut self, smsh: &mut Shell) -> Result<()> {
        match self.expansion {
            Expansion::Variable => {
                let mut key = self.text[1..].to_string();
                key.pop();

                if let Some(val) = smsh.get_user_variable(&key) {
                    self.text = val;
                } else {
                    self.text.clear();
                }

                Ok(())
            }
            Expansion::Environment => {
                let mut key = self.text[2..].to_string();
                key.pop();

                if let Some(val) = env::var_os(key) {
                    self.text = val.into_string().unwrap_or("".to_string());
                } else {
                    self.text.clear();
                }

                Ok(())
            }
            Expansion::Subshell => {
                let mut line = self.text[2..].to_string();
                line.pop();
                self.text = subshell_expand(smsh, line)?;

                Ok(())
            }
            _ => {
                Ok(())
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text().is_empty()
    }
}

fn get_expansion(text: &str) -> Expansion {
    if text.len() < 2 {
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

pub fn subshell_expand(smsh: &mut Shell, line: String) -> Result<String>{
    eprintln!("Subshell expanding line:\n{}", line);

    let (rd, wr) = pipe()?;

    match unsafe{fork()?} {
        ForkResult::Parent { child: _, .. } => {
            close(wr)?;

            wait()?;

            let mut buf = String::new();

            unsafe {
                let mut rd = File::from_raw_fd(rd);
                rd.read_to_string(&mut buf)?;
            }

            Ok(buf)
        }
        ForkResult::Child => {
            smsh.clear_sources(); 

            close(rd)?;
            close(1 as RawFd)?;
            dup2(wr, 1 as RawFd)?;
            close(wr)?;

            let line = Line::new(line, 0, SourceKind::Subshell);
            smsh.push_source(BufferSource::new(vec![line]));

            while let Err(e) = smsh.run() {
                eprintln!("smsh (subshell): {}", e);
            }

            std::process::exit(0);
        }
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
                    words.push(Word::new(word_text, quoted)?);
                    word_text = String::new();
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
