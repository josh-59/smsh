mod line;
mod shell;
mod sources;
mod constructs;

use shell::Shell;

fn main() {
    let mut smsh = Shell::new();

    while let Err(e) = smsh.run() {
        eprintln!("smsh: {}", e);

        smsh.backtrace();
    }

    std::process::exit(smsh.state().rv);
}
