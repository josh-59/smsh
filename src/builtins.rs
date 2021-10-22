use anyhow::Result;
use super::shell::Shell;
use std::env;

pub type Builtin = fn(&mut Shell, Vec::<String>) -> Result<()>;

pub fn chdir(smsh: &mut Shell, args: Vec::<String>) -> Result<()> {

    if args.len() == 1 { 
        if let Some(dir) = env::var_os("HOME") {
            env::set_current_dir(dir);
        }
    } else {
        env::set_current_dir(&args[1]);
    }

    Ok(())

}
