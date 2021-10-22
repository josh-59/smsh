use anyhow::Result;
use std::io;

fn main() -> Result<()> {

    let stdin = io::stdin();
    let mut buffer = String::new();

    stdin.read_line(&mut buffer)?;

    println!("{}", buffer.trim());

    Ok(())
}
