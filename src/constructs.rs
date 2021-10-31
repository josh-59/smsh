use crate::shell::Shell;
use crate::sources::Prompt;
use crate::line::Line;

use anyhow::{anyhow, Result};

pub fn r#if(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    if !line.is_if() {
        smsh.set_rv(1);
        return Err(anyhow!("if: Improperly formed conditional"));
    }

    let conditional = line.get_conditional()?;

    let body = smsh.get_block(line.source(), line.indentation() + 1)?;

    let mut bodies = vec![body];  // Vector of conditional bodies,
    // We must collect these first, then determine which to execute
    // by executing their conditionals in a subshell environment.
    
    let mut conditionals = vec![conditional];

    while let Some(line) = smsh.get_line(Prompt::Block)? {
        if line.is_elif() {
            let conditional = line.get_conditional()?;
            conditionals.push(conditional);

            let body = smsh.get_block(line.source(), line.indentation() + 1)?;
            bodies.push(body);
        } else {
            smsh.push_back(line);
            break;
        }
    }

    let else_body = if let Some(line) = smsh.get_line(Prompt::Block)? {
        if line.is_else() {
            Some(smsh.get_block(line.source(), line.indentation() + 1)?)
        }
        else {
            None
        }
    } else {
        None
    };

    let mut found = false;

    for (conditional, body) in conditionals.iter().zip(bodies) {
        if smsh.execute_subshell(conditional)? {
            found = true;
            smsh.push_block(body);
            break;
        }
    }

    if !found {
        if let Some(body) = else_body {
            smsh.push_block(body);
        }
    }

    smsh.set_rv(0);
    Ok(())
}
