use anyhow::Result;

#[derive(Clone, Copy)]
pub enum Quote {
    SingleQuoted,
    DoubleQuoted,
    Unquoted,
}

#[derive(Clone, Copy)]
pub enum Expansion {
    None,           // If single-quoted
    Variable,
    Environment,
    Subshell
}

pub enum WordSeparator {
    None,           // If single- or double-quoted
    Whitespace,     // Default
    Line,           
    Arbitrary(String),
}

pub struct Word {
    text: String,
    expansion: Expansion,
    separator: WordSeparator,
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
        Ok(Word { text, expansion, separator: WordSeparator::Whitespace })
    }

    pub fn text<'a>(&'a self) -> &'a str {
        &self.text
    }
}

fn get_expansion(text: &str) -> Expansion {
    Expansion::None
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
