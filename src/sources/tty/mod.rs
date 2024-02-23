use std::borrow::Cow;
use std::boxed::Box;
use std::collections::VecDeque;

use anyhow::Result;
use nix::unistd;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal};

use super::{Source, SourceKind};
use crate::line::Line;

mod line_validator;
use line_validator::SmshLineValidator;
mod completer;

pub struct Tty {
    line_editor: Reedline,
    line_num: usize,
    last_line: Option<Line>,
    buffer: VecDeque<Line>,
}

impl Tty {
    pub fn new() -> Box<dyn Source> {
        let line_editor = Reedline::create().with_validator(Box::new(SmshLineValidator));

        Box::new(Tty {
            line_editor,
            line_num: 0,
            last_line: None,
            buffer: VecDeque::<Line>::new(),
        })
    }
}

impl Source for Tty {
    fn get_line(&mut self) -> Result<Option<Line>> {
        if let Some(line) = self.buffer.pop_front() {
            return Ok(Some(line));
        }

        match self.line_editor.read_line(&SimplePrompt)? {
            Signal::Success(buffer) => {
                self.line_num += 1;

                // Since we want blocks to be given the Reedline multiline editing treatment,
                // we must collect a block of lines in a single line, then decompose it, then
                // serve it up later using self.buffer.
                let line = if let Some((first_line, remainder)) = buffer.split_once(":\n") {
                    let mut first_line = first_line.to_string();
                    first_line.push(':'); // This is retained for processing later on
                    let first_line =
                        Line::new(first_line.to_string(), self.line_num, SourceKind::Tty)?;

                    for line in remainder.split('\n') {
                        self.line_num += 1;
                        self.buffer.push_back(Line::new(
                            line.to_string(),
                            self.line_num,
                            SourceKind::Tty,
                        )?);
                    }

                    first_line
                } else {
                    Line::new(buffer, self.line_num, SourceKind::Tty)?
                };
                self.last_line = Some(line.clone());

                Ok(Some(line))
            }
            Signal::CtrlC => Ok(Some(
                Line::new(String::new(), self.line_num, SourceKind::Tty).unwrap(),
            )),
            Signal::CtrlD => Ok(None),
        }
    }

    fn is_tty(&self) -> bool {
        true
    }

    fn print_error(&mut self) -> Result<()> {
        if let Some(line) = &self.last_line {
            eprintln!("{}", line);
        }

        Ok(())
    }
}

struct SimplePrompt;

impl Prompt for SimplePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        let prompt_string = if unistd::getuid().is_root() {
            "# ".to_string()
        } else {
            "$ ".to_string()
        };
        Cow::Owned(prompt_string)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Owned("".to_string())
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        let prompt_string = "".to_string();
        Cow::Owned(prompt_string)
    }
    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        let prompt_string = "> ".to_string();
        Cow::Owned(prompt_string)
    }
    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prompt_string = "> ".to_string();
        Cow::Owned(prompt_string)
    }
}
