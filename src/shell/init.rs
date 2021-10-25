
use anyhow::Result;
use xdg::BaseDirectories;

use crate::sources::{
    tty::TTY, 
    user_function::UserFunction,
    script::Script};
use super::modules::Module;
use super::{Shell, Builtin, load_module};

use std::collections::HashMap;
use std::path::PathBuf;

pub fn init() -> Result<Shell> {
    let sources = vec![TTY::new()?];
    let builtins = HashMap::<&'static str, Builtin>::new();
    let user_variables = HashMap::<String, String>::new();
    let user_functions = HashMap::<String, UserFunction>::new();

    let mut smsh = Shell { 
        sources, 
        builtins, 
        user_variables,
        user_functions,
    };

    load_module(&mut smsh, Module::Core)?;

    push_init_script(&mut smsh);

    Ok(smsh)
}

pub fn push_init_script(smsh: &mut Shell) -> Result<()> {
    let base_dirs = BaseDirectories::new()?;

    let temp = PathBuf::from("smsh/init");

    if let Some(path) = base_dirs.find_config_file(temp) {
        smsh.push_source(Script::new(path)?);
    } 

    Ok(())
}
