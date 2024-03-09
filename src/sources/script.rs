use anyhow::Result;

use super::{Source, SourceKind};
use crate::line::Line;

use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Script {
    path: PathBuf,
    body: Vec<String>,
    line_num: usize,
    last_line: Option<Line>,
}

impl Script {
    // TODO:  This function reads the entire script into memory before
    // dolling it out; should probably read a (logical) line at a time.
    // On the other hand, though, even a large script (10000 lines) is
    // less than 10 MB, so...
    pub fn build_source(path: PathBuf) -> Result<Box<dyn Source>> {
        let body = read_to_string(&path)?
            .lines()
            .map(|x| x.to_string())
            .collect();

        let script = Script {
            path,
            body,
            line_num: 0,
            last_line: None,
        };

        Ok(Box::new(script))
    }

    pub fn file_name(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

impl Source for Script {
    fn get_line(&mut self) -> Result<Option<Line>> {
        if self.line_num == self.body.len() {
            Ok(None)
        } else {
            let text = self.body[self.line_num].clone();
            self.line_num += 1;

            let line = Line::new(text, self.line_num, SourceKind::Script(self.file_name()))?;

            self.last_line = Some(line.clone());

            Ok(Some(line))
        }
    }

    fn get_source_kind(&self) -> SourceKind {
        let p = match self.path.to_str() {
            Some(p) => { p.to_string() }
            None  => { "".to_string() }
        };

        SourceKind::Script(p)
    }

    fn print_error(&mut self) -> Result<()> {
        if let Some(line) = &self.last_line {
            eprintln!("{}", line);
        }

        Ok(())
    }
}
