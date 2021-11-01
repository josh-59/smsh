use crate::shell::Shell;
use crate::shell::modules::unload_module;

use anyhow::{anyhow, Result};
use std::env;

use super::{load_module, Module};

pub fn chdir(smsh: &mut Shell, argv: Vec<&str>) -> Result<()> {
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

pub fn exit(smsh: &mut Shell, _argv: Vec<&str>) -> Result<()> {
    smsh.set_rv(0);
    std::process::exit(smsh.state().rv);
}

pub fn lm_builtin(smsh: &mut Shell, argv: Vec<&str>) -> Result<()> {

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

pub fn ulm_builtin(smsh: &mut Shell, argv: Vec<&str>) -> Result<()> {
    if argv.len() == 2 {
        match argv[1] {
            "core" => {
                unload_module(smsh, Module::Core)
            }
            _ => {
                smsh.set_rv(2);
                Err(anyhow!("unload_module: Unrecognized module {}", argv[1]))
            }
        }
    } else {
        smsh.set_rv(1);
        Err(anyhow!("unload_module: Improper invocation of self::unload_module"))
    }
}

pub fn r#let(smsh: &mut Shell, argv: Vec<&str>) -> Result<()> {

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