use anyhow::Result;
use super::shell::Shell;
use std::env;

pub type Builtin = fn(&mut Shell, Vec::<String>) -> Result<()>;

pub fn chdir(smsh: &mut Shell, args: Vec::<String>) -> Result<()> {

    if let Some(dir) = env::var_os("HOME") {
        env::set_current_dir(dir);
    }

    Ok(())

}
