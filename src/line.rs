use anyhow::Result;

use std::fmt;

pub struct Line {
    pub rawline: String,
}

impl Line {
    pub fn new(s: String) -> Result<Line> {
        let rawline = s.trim().to_string();

        Ok( Line { rawline } )
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rawline)
    }
}
