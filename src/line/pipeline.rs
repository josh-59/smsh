use anyhow::Result;

use super::Line;
use crate::Shell;

pub struct Pipeline {
    elements: Vec<PipeElement>
}

impl Pipeline {
    pub fn new(line: &mut Line) -> Result<Self> {

        let mut elem = PipeElement::new(line.clone());
        let mut elements = Vec::<PipeElement>::new();

        for word in line.words() {
            if word.is_pipe_operator() {
                elements.push(elem);
                elem = PipeElement::new(line.clone());
            } else {
                for arg in word.selected_text() {
                    if !arg.is_empty() {
                        elem.push_arg(arg.to_string())
                    }
                }
            }
        }

        elements.push(elem);

        Ok( Pipeline {elements})
    }

    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {
        for elem in &mut self.elements {
            elem.execute(smsh)?;
        }

        Ok(())
    }
}

pub struct PipeElement {
    line: Line,
    args: Vec<String>,
}

impl PipeElement {
    pub fn new(line: Line) -> Self {
        PipeElement{ line, args: vec![] }
    }

    pub fn argv(&self) -> Vec<&str> {
        let mut strs = Vec::<&str>::new();

        for arg in &self.args {
            if !arg.is_empty() {
                strs.push(arg.as_str())
            }
        }

        strs
    }


    pub fn execute(&mut self, smsh: &mut Shell) -> Result<()> {
        let strs: Vec<&str> = self.args.iter().filter_map(|x| 
                                if x.is_empty() {
                                    None
                                } else {
                                    Some(x.as_str())
                                }).collect();
    
        if strs.is_empty() {
            return Ok(());
        }

        if let Some(f) = smsh.get_user_function(strs[0]) {
            smsh.push_source(f.build_source());
            Ok(())
        } else if let Some(f) = smsh.get_builtin(strs[0]) {
            f(smsh, &mut self.line)?;
            Ok(())
        } else {
            smsh.execute_external_command(strs)?;
            Ok(())
        }
    }

    pub fn push_arg(&mut self, arg: String) {
        self.args.push(arg);
    }
}
