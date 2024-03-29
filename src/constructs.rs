use crate::line::Line;
/// This file contains the definitions for
/// if, for, while, let and fn.
// Countdown: Return all 10 lines, then 9, then 8...
use crate::shell::Shell;
use crate::sources::r#for::For;
use crate::sources::user_function::UserFunction;

use anyhow::{anyhow, Result};

pub fn r#if(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    // We expect to find 'if' before 'elif' and 'else'.
    if line.is_elif() {
        smsh.set_rv(1);
        return Err(anyhow!("if: `elif` must follow `if`"));
    } else if line.is_else() {
        smsh.set_rv(1);
        return Err(anyhow!("if: `else` must follow `if`"));
    }

    let conditional = line.get_conditional()?;

    let body = smsh.get_block(line.source(), line.indentation() + 1)?;

    let mut bodies = vec![body]; // Vector of conditional bodies,
                                 // We must collect these first, then determine which to execute
                                 // by executing their conditionals in a subshell environment.
    let mut conditionals = vec![conditional];

    while let Some(line) = smsh.get_line()? {
        if line.is_elif() {
            let conditional = line.get_conditional()?;
            conditionals.push(conditional);

            let body = smsh.get_block(line.source(), line.indentation() + 1)?;
            bodies.push(body);
        } else {
            smsh.push_line(line);
            break;
        }
    }

    let else_body = if let Some(line) = smsh.get_line()? {
        if line.is_else() {
            Some(smsh.get_block(line.source(), line.indentation() + 1)?)
        } else {
            None
        }
    } else {
        None
    };

    let mut found = false;

    for (conditional, body) in conditionals.iter().zip(bodies) {
        match smsh.evaluate_conditional(conditional)? {
            Some(b) => {
                if b {
                    found = true;
                    smsh.push_block(body);
                    break;
                }
            }
            None => return Ok(()),
        }
    }

    if !found {
        if let Some(body) = else_body {
            smsh.push_block(body);
        }
    }

    Ok(())
}

// Collect a block of input from the shell, create a
// a new function with it, and save it into the shell
pub fn r#fn(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    // If function invocation is incorrect, we collect and discard
    // the following block of input
    let fn_body = smsh
        .get_block(line.source(), line.indentation() + 1)?
        .iter()
        .map(|x| x.raw_text().to_string())
        .collect();

    let argv = line.argv();

    if argv.len() != 2 {
        smsh.set_rv(-1);
        return Err(anyhow!("fn: Improper invocation of `fn`"));
    }

    let fn_name = argv.last().unwrap().to_string();

    let func = UserFunction::new(fn_name, fn_body);

    smsh.insert_user_function(func);

    Ok(())
}

// Define and push a 'for' loop onto execution stack
// TODO: `for` loops should implicitly 'unset' the iterator key when the
// body of the for loop is complete.  It would be sufficient (but crude) to
// run the line, `let iterator_key = ` after the for loop exits
pub fn r#for(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    // We allow empty for loop: Just don't do anything.
    if argv.len() == 3 && argv[2] == "in" {
        let _discard: Vec<Line> = smsh.get_block(line.source(), line.indentation() + 1)?;
        return Ok(());
    } else if argv.len() < 4 || argv[2] != "in" {
        return Err(anyhow!("Improperly formed for loop"));
    }

    let iterator_key = argv[1].to_string();

    let iterator_values: Vec<String> = argv[3..].iter().map(|x| x.to_string()).collect();

    let body: Vec<Line> = smsh.get_block(line.source(), line.indentation() + 1)?;

    smsh.push_source(
        For::new(
            iterator_key,
            iterator_values,
            body,
            line.identifier().clone(),
        )
        .build_source(),
    );

    smsh.set_rv(0);
    Ok(())
}

pub fn r#while(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let conditional = line.get_conditional()?;

    let body = smsh.get_block(line.source(), line.indentation() + 1)?;

    if let Some(res) = smsh.evaluate_conditional(&conditional)? {
        if res {
            smsh.push_block(body.clone());
            smsh.push_line(line.clone());
            smsh.push_block(body);
        }
    }
    Ok(())
}

pub fn r#let(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    if argv.len() < 4 || argv[2] != "=" {
        smsh.set_rv(1);
        return Err(anyhow!("Improper invocation of `let`"));
    }

    let key = argv[1].to_string();
    let mut value = String::new();

    for word in &argv[3..] {
        value.push_str(word);
        value.push(' ');
    }

    value.pop();

    smsh.insert_user_variable(key, value);

    Ok(())
}
