mod shell;
mod sources;
mod line;

use shell::Shell;

fn main() {
    let mut smsh = Shell::new();

    while let Err(e) = smsh.run() {
        eprintln!("smsh: {}", e);

        if smsh.is_interactive() {
            smsh.reset_interactive();
        }
    }
}
