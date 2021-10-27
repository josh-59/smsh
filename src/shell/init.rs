use xdg::BaseDirectories;

use crate::sources::{
    user_function::UserFunction,
    script::Script};
use super::modules::Module;
use super::{Shell, Builtin, load_module};

use std::collections::HashMap;
use std::path::PathBuf;

// We do not want this function to fail, so that
// a user of smsh always gets into its main loop.
pub fn init() -> Shell {
    let sources = vec![];
    let builtins = HashMap::<&'static str, Builtin>::new();
    let user_variables = HashMap::<String, String>::new();
    let user_functions = HashMap::<String, UserFunction>::new();

    let mut smsh = Shell { 
        sources, 
        builtins, 
        user_variables,
        user_functions,
    };

    load_module(&mut smsh, Module::Core);

    push_init_script(&mut smsh);

    smsh
}

// Again, we do not want this function to fail.
pub fn push_init_script(smsh: &mut Shell) {
    match BaseDirectories::new() {
        Ok(base_dirs) => {
            let temp = PathBuf::from("smsh/init");

            if let Some(path) = base_dirs.find_config_file(temp) {
                match Script::new(path) {
                    Ok(script) => {
                        smsh.push_source(script);
                    }
                    Err(e) => {
                        eprintln!("smsh: init: Unable to create script:\n{}", e);
                    }
                }
            } 
        }
        Err(e) => {
            eprintln!("smsh: init: Unable to obtain XDG Base Directories:\n{}", e);
        }
    }
}
