use std::borrow::Cow;
use std::boxed::Box;
use std::collections::VecDeque;

use anyhow::Result;
use nix::unistd;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal};

use super::{Source, SourceKind};
use crate::line::Line;
use crate::sources::is_complete;

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
            line_num: 1,  // TODO: line_num should probably reflect physical line, not logical...
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
                // Since we want blocks to be given the Reedline multiline editing treatment,
                // we must collect a block of lines in a single line (buffer), then decompose it, then
                // serve it up later (using self.buffer).
                //
                // NOTE: read_line() (above) returns Signal::Success(buffer) only if buffer has passed
                // completeness tests (notably, is_complete(), found in sources/mod.rs).  Hence,
                // we can assume that line is complete.  It may, however, contain multiple physical
                // lines (or even a block).
                //
                // So, first we collect the logical lines in a vector...
                let mut logical_lines = Vec::<String>::new();
                let mut line = String::new();
                for physical_line in buffer.split("\n") {  // Implicitly removes newline characters
                    line.push_str(physical_line); // Implicitly ignores physical lines of length 0

                    if line.len() > 0 && is_complete(line.as_str()) {
                        logical_lines.push(line);
                        line = String::new();
                    }
                }

                for line in logical_lines {
                    self.buffer
                        .push_back(Line::new(line, self.line_num, SourceKind::Tty)?);
                    self.line_num += 1;
                }

                if let Some(line) = self.buffer.pop_front() {
                    Ok(Some(line))
                } else {
                    Ok(Some(Line::new("".to_string(), self.line_num, SourceKind::Tty)?))
                }
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
