use std::env;
use std::path::PathBuf;

use anyhow::Result;
use reedline::DefaultCompleter;

pub fn build_command_completer() -> Result<DefaultCompleter> {
    let commands = get_commands()?;

    let completer = DefaultCompleter::new(commands);

    Ok(completer)
}

// TODO:  Remove 'unwrap' call,
// Need a way to update this if a new command is installed
fn get_commands() -> Result<Vec<String>> {
    let paths  = if let Some(os_string) = env::var_os("PATH") {
        let string = os_string.into_string().unwrap();
        string.split(":").map(|x| x.to_string()).collect()
    } else {
        vec![]
    };

    let mut commands: Vec<String> = vec![];

    for p in paths {
        let path = PathBuf::from(p);
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                commands.push(entry.file_name().into_string().unwrap_or("".to_string()));
            }
        }
    }

    Ok(commands)
}

