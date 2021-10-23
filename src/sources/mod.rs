use anyhow::Result;

use super::line::Line;

pub mod tty;

pub enum SourceKind {
    TTY,
}

pub trait Source {
    fn get_line(&mut self, prompt: &str) -> Result<Option<Line>>;
    fn is_tty(&self) -> bool;
}

