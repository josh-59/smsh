use anyhow::Result;

mod shell;
mod source;
mod line;

use shell::Shell;

fn main() -> Result<()> {

    let mut smsh = Shell::new()?;

    while let Err(e) = smsh.run() {
        eprintln!("smsh: {}", e);
    }

    Ok(())

}
