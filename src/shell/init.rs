use xdg::BaseDirectories;

use super::modules::{Module, Builtin, load_module};
use super::Shell;
use super::state::State;
use crate::sources::{
    Sources,
    script::Script, 
    user_function::UserFunction,
    tty::Tty,
};


use std::collections::HashMap;
use std::path::PathBuf;

// We do not want this function to fail, so that
// a user of smsh always gets into its main loop.
pub fn init() -> Shell {
    let state = State::new();
    let sources = Sources::new();
    let builtins = HashMap::<&'static str, Builtin>::new();
    let user_variables = HashMap::<String, String>::new();
    let user_functions = HashMap::<String, UserFunction>::new();

    let mut smsh = Shell {
        state,
        sources,
        builtins,
        user_variables,
        user_functions,
    };

    load_module(&mut smsh, Module::Core);

    if smsh.state().is_interactive() {
        smsh.push_source(Tty::build_source());
        push_interactive_init_script(&mut smsh);
    }

    smsh
}

// Again, we do not want this function to fail.
fn push_interactive_init_script(smsh: &mut Shell) {
    match BaseDirectories::new() {
        Ok(base_dirs) => {
            let temp = PathBuf::from("smsh/init");

            if let Some(path) = base_dirs.find_config_file(temp) {
                match Script::build_source(path) {
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
