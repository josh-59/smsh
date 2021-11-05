use std::env;
use std::path::PathBuf;

use anyhow::Result;
use reedline::DefaultCompleter;

pub fn build_completer() -> Result<DefaultCompleter> {
    let commands = get_commands()?;

    let completer = DefaultCompleter::new(commands);

    Ok(completer)
}

fn get_commands() -> Result<Vec<String>> {
    let path_strings  = if let Some(os_string) = env::var_os("PATH") {
        let string = os_string.into_string().unwrap();
        string.split(":").map(|x| x.to_string()).collect()
    } else {
        vec![]
    };

    let mut commands = vec![];

    for path_string in path_strings {
        let path = PathBuf::from(path_string);
        for entry in path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                commands.push(entry.file_name().into_string().unwrap());
            }
        }
    }

    Ok(commands)
}

