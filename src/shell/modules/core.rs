use anyhow::{anyhow, Result};
use crate::shell::Shell;
use std::env;

use super::{Module, load_module, unload_module};

pub fn chdir(_smsh: &mut Shell, args: Vec::<String>) -> Result<()> {
    if args.len() == 1 { 
        if let Some(dir) = env::var_os("HOME") {
            env::set_current_dir(dir)?;
        }
    } else {
        env::set_current_dir(&args[1])?;
    }

    Ok(())
}

pub fn exit(_smsh: &mut Shell, _args: Vec::<String>) -> Result<()> {
    std::process::exit(0);
}

pub fn lm_builtin(smsh: &mut Shell, args: Vec::<String>) -> Result<()> {
    if args.len() == 2 {
        match args[1].as_str() {
            "core" => {
                load_module(smsh, Module::Core)?;
                Ok(())
            }
            _ => {
                Err(anyhow!("Unrecognized module {}", args[1]))
            }
        }
    } else {
        Ok(())
    }
}

pub fn ulm_builtin(smsh: &mut Shell, args: Vec::<String>) -> Result<()> {
    if args.len() == 2 {
        match args[1].as_str() {
            "core" => {
                unload_module(smsh, Module::Core)?;
                Ok(())
            }
            _ => {
                Err(anyhow!("Unrecognized module {}", args[1]))
            }
        }
    } else {
        Ok(())
    }
}
