use anyhow::{anyhow, Result};
use super::Shell;

mod core;

pub type Builtin = fn(&mut Shell, Vec::<String>) -> Result<()>;

pub enum Module {
    Core
}

pub fn load_module(smsh: &mut Shell, module: Module) -> Result<()> {
    match module {
        Module::Core => {
            smsh.builtins.insert("cd", core::chdir);
            smsh.builtins.insert("exit", core::exit);
            smsh.builtins.insert("self::load_module", core::lm_builtin);
            smsh.builtins.insert("self::unload_module", core::ulm_builtin);
            Ok(())
        }
    }
}

pub fn unload_module(_smsh: &mut Shell, module: Module) -> Result<()> {
    match module {
        Module::Core => {
            Err(anyhow!("Unable to unload smsh core module!"))
        }
    }
}