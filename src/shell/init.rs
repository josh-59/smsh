use xdg::BaseDirectories;

use super::Shell;
use crate::sources::script::Script;

use std::path::PathBuf;

// Again, we do not want this function to fail.
pub fn push_interactive_init_script(smsh: &mut Shell) {
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
