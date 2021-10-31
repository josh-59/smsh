use crate::shell::Shell;
use crate::sources::user_function::UserFunction;
use crate::line::Line;

use anyhow::{anyhow, Result};
use std::env;

use super::{load_module, Module};

pub fn chdir(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    if argv.len() == 1 {
        if let Some(dir) = env::var_os("HOME") {
            env::set_current_dir(dir)?;
            smsh.set_rv(0);
        }
    } else if argv.len() == 2 {
        env::set_current_dir(&argv[1])?;
        smsh.set_rv(0);
    } else {
        smsh.set_rv(0);
        return Err(anyhow!("cd: Too many arguments"));
    }

    Ok(())
}

pub fn exit(smsh: &mut Shell, _line: &mut Line) -> Result<()> {
    smsh.set_rv(0);
    std::process::exit(smsh.state().rv);
}

pub fn lm_builtin(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    if argv.len() == 2 {
        match argv[1] {
            "core" => {
                load_module(smsh, Module::Core);
                smsh.set_rv(0);
                Ok(())
            }
            _ => {
                smsh.set_rv(1);
                Err(anyhow!("Unrecognized module {}", argv[1]))
            }
        }
    } else {
        smsh.set_rv(2);
        Err(anyhow!("Improper invocation of self::load_module"))
    }
}

pub fn ulm_builtin(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    if argv.len() == 2 {
        match argv[1] {
            "core" => {
                smsh.set_rv(3);
                Err(anyhow!("Unable to unload smsh core module!"))
            }
            _ => {
                smsh.set_rv(2);
                Err(anyhow!("Unrecognized module {}", argv[1]))
            }
        }
    } else {
        smsh.set_rv(1);
        Err(anyhow!("Improper invocation of self::unload_module"))
    }
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

    smsh.set_rv(0);
    Ok(())
}

// Collect a block of input from the shell, create a
// a new function with it, and save it into the shell
pub fn r#fn(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    if argv.len() != 2 || !argv[1].ends_with(':') {
        smsh.set_rv(1);
        return Err(anyhow!("Improper invocation of `fn`"));
    }

    let mut fn_name = argv[1].to_string();
    fn_name.pop();

    let fn_body = smsh.get_block()?.iter().map(|x| x.text()).collect();

    let func = UserFunction::new(fn_name, fn_body);

    smsh.insert_user_function(func);

    smsh.set_rv(0);
    Ok(())
}

// XXX: Kind of ugly: Since this is implemented as a builtin,
// must parse a vector of strings, then deal with 'Line' structs
// afterwards.
pub fn r#if(smsh: &mut Shell, line: &mut Line) -> Result<()> {
    let argv = line.argv();

    if argv.len() < 2 || !argv[argv.len() - 1].ends_with(':') {
        smsh.set_rv(1);
        return Err(anyhow!("if: Improperly formed conditional"));
    }

    let mut conditional = String::new();

    for arg in &argv[1..] {
        conditional.push_str(arg);
        conditional.push(' ');
    }

    conditional.pop(); // Remove trailing whitespace
    conditional.pop(); // Remove trailing semicolon

    let body = smsh.get_block()?;

    let mut bodies = vec![body];  // Vector of conditional bodies,
    // We must collect these first, then determine which to execute
    // by executing their conditionals in a subshell environment.
    
    let mut conditionals = vec![conditional];

    while let Some(line) = smsh.get_line(None)? {
        if line.is_elif() {
            let conditional = line.get_conditional();
            conditionals.push(conditional);

            let body = smsh.get_block()?;
            bodies.push(body);
        } else {
            smsh.push_back(line);
            break;
        }
    }

    let else_body = if let Some(line) = smsh.get_line(None)? {
        if line.is_else() {
            Some(smsh.get_block()?)
        }
        else {
            None
        }
    }
    else {
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
