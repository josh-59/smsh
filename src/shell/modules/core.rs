use crate::shell::Shell;
use crate::sources::user_function::UserFunction;
use anyhow::{anyhow, Result};
use std::env;

use super::{load_module, unload_module, Module};

pub fn chdir(_smsh: &mut Shell, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        if let Some(dir) = env::var_os("HOME") {
            env::set_current_dir(dir)?;
        }
    } else if args.len() == 2 {
        env::set_current_dir(&args[1])?;
    } else {
        return Err(anyhow!("cd: Too many arguments"));
    }

    Ok(())
}

pub fn exit(_smsh: &mut Shell, _args: Vec<&str>) -> Result<()> {
    std::process::exit(0);
}

pub fn lm_builtin(smsh: &mut Shell, args: Vec<&str>) -> Result<()> {
    if args.len() == 2 {
        match args[1] {
            "core" => {
                load_module(smsh, Module::Core);
                Ok(())
            }
            _ => Err(anyhow!("Unrecognized module {}", args[1])),
        }
    } else {
        Ok(())
    }
}

pub fn ulm_builtin(smsh: &mut Shell, args: Vec<&str>) -> Result<()> {
    if args.len() == 2 {
        match args[1] {
            "core" => {
                unload_module(smsh, Module::Core)?;
                Ok(())
            }
            _ => Err(anyhow!("Unrecognized module {}", args[1])),
        }
    } else {
        Ok(())
    }
}

pub fn r#let(smsh: &mut Shell, args: Vec<&str>) -> Result<()> {
    if args.len() < 4 || args[2] != "=" {
        return Err(anyhow!("Improper invocation of `let`"));
    }

    let key = args[1].to_string();
    let mut value = String::new();

    for word in &args[3..] {
        value.push_str(word);
        value.push(' ');
    }

    value.pop();

    smsh.insert_user_variable(key, value);

    Ok(())
}

// Collect a block of input from the shell, create a
// a new function with it, and save it into the shell
pub fn r#fn(smsh: &mut Shell, args: Vec<&str>) -> Result<()> {
    if args.len() != 2 || !args[1].ends_with(':') {
        return Err(anyhow!("Improper invocation of `fn`"));
    }

    let mut fn_name = args[1].to_string();
    fn_name.pop();

    let fn_body = smsh.get_block()?.iter().map(|x| x.text()).collect();

    let func = UserFunction::new(fn_name, fn_body);

    smsh.insert_user_function(func);

    Ok(())
}

// XXX: Kind of ugly: Since this is implemented as a builtin,
// must parse a vector of strings, then deal with 'Line' structs
// afterwards.
pub fn r#if(smsh: &mut Shell, args: Vec<&str>) -> Result<()> {
    if args.len() < 2 || !args[args.len() - 1].ends_with(':') {
        return Err(anyhow!("if: Improperly formed conditional"));
    }

    let mut conditional = String::new();

    for arg in &args[1..] {
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

    Ok(())
}
