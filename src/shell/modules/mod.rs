use super::Shell;
use anyhow::{anyhow, Result};

mod core;

pub type Builtin = fn(&mut Shell, Vec<&str>) -> Result<()>;

pub enum Module {
    Core,
}

pub fn load_module(smsh: &mut Shell, module: Module) {
    match module {
        Module::Core => {
            smsh.builtins.insert("cd", core::chdir);
            smsh.builtins.insert("let", core::r#let);
            smsh.builtins.insert("fn", core::r#fn);
            smsh.builtins.insert("exit", core::exit);
            smsh.builtins.insert("self::load_module", core::lm_builtin);
            smsh.builtins
                .insert("self::unload_module", core::ulm_builtin);
        }
    }
}

pub fn unload_module(_smsh: &mut Shell, module: Module) -> Result<()> {
    match module {
        Module::Core => Err(anyhow!("Unable to unload smsh core module!")),
    }
}
