use std::borrow::Cow;
use std::boxed::Box;

use anyhow::{anyhow, Result};
use reedline::{DefaultPrompt, Reedline, Signal, Prompt as ReedlinePrompt, PromptEditMode, PromptHistorySearch};

use super::{Source, SourceKind, Prompt, is_complete};
use crate::line::Line;

pub struct Tty {
    line_editor: Reedline,
    line_num: usize,
    last_line: Option<Line>
}

impl Tty {
    pub fn build_source() -> Result<Box<dyn Source>> {
        let line_editor = Reedline::create()?;

        Ok(Box::new(Tty { line_editor, line_num: 0, last_line: None}))
    }

    pub fn get_secondary_line(&mut self) -> Result<String> {
        let sig = self.line_editor.read_line(&BlockPrompt)?;

        match sig {
            Signal::Success(buffer) => {
                Ok(buffer)
            }
            Signal::CtrlD => {
                Err(anyhow!("Unexpected EOF"))
            }
            _ => {
                Err(anyhow!("reedline: Unexpected input"))
            }
        }
    }
}


impl Source for Tty {
    fn get_line(&mut self, prompt: Prompt) -> Result<Option<Line>> {

        let sig = match prompt {
            Prompt::Normal => {
                self.line_editor.read_line(&DefaultPrompt::default())?
            }
            Prompt::Block => {
                self.line_editor.read_line(&BlockPrompt)?
            }
        };

        match sig {
            // `buffer` does not contain trailing newline.
            Signal::Success(mut buffer) => {
                while !is_complete(buffer.as_str()) {
                    buffer.push('\n');
                    buffer.push_str(self.get_secondary_line()?.as_str());
                }

                self.line_num += 1;
                let line= Line::new(buffer, self.line_num, SourceKind::Tty)?;
                self.last_line = Some(line.clone());
                Ok(Some(line))
            }
            Signal::CtrlD => {
                Ok(None)
            }
            _ => {
                Err(anyhow!("reedline: Unexpected input"))
            }
        }
    }

    fn is_tty(&self) -> bool {
        true
    }

    fn is_faux_source(&self) -> bool {
        false
    }

    fn print_error(&mut self) -> Result<()> {
        if let Some(line) = &self.last_line {
            eprintln!("{}", line);
        }

        Ok(())
    }
}

struct BlockPrompt;

impl ReedlinePrompt for BlockPrompt {
    fn render_prompt(&self, _screen_width: usize) -> Cow<'_, str> {
        let prompt_string = "> ".to_string();
        Cow::Owned(prompt_string)
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        let prompt_string = "".to_string();
        Cow::Owned(prompt_string)
    }
    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        let prompt_string = "> ".to_string();
        Cow::Owned(prompt_string)
    }
    fn render_prompt_history_search_indicator(&self, _history_search: PromptHistorySearch) -> Cow<'_, str> {
        let prompt_string = "> ".to_string();
        Cow::Owned(prompt_string)
    }
}