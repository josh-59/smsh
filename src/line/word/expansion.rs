use anyhow::Result;
use crate::shell::Shell;
use crate::sources::{SourceKind, BufferSource};
use crate::line::Line;

use super::{Word, Expansion};

use std::env;
use std::os::unix::io::{RawFd, FromRawFd};
use std::fs::File;
use std::io::Read;

use nix::unistd::{fork, pipe, ForkResult, close, dup2};
use nix::sys::wait::wait;

pub fn expand(word: &mut Word, smsh: &mut Shell) -> Result<()> {
    match word.expansion {
        Expansion::Variable => {
            let mut key = word.text[1..].to_string();
            key.pop();

            if let Some(val) = smsh.get_user_variable(&key) {
                word.text = val;
            } else {
                word.text.clear();
            }

            Ok(())
        }
        Expansion::Environment => {
            let mut key = word.text[2..].to_string();
            key.pop();

            if let Some(val) = env::var_os(key) {
                word.text = val.into_string().unwrap_or("".to_string());
            } else {
                word.text.clear();
            }

            Ok(())
        }
        Expansion::Subshell => {
            let mut line = word.text[2..].to_string();
            line.pop();
            word.text = subshell_expand(smsh, line)?;

            Ok(())
        }
        _ => {
            Ok(())
        }
    }
}

pub fn get_expansion(text: &str) -> Expansion {
    if text.len() < 2 {
        Expansion::None
    } else if text.starts_with("{") && text.ends_with("}") {
            Expansion::Variable
    } else if text.starts_with("!{") && text.ends_with("}") {
            Expansion::Subshell
    } else if text.starts_with("e{") && text.ends_with("}") {
            Expansion::Environment
    } else if text[0..2].contains("{") {
        Expansion::Unknown
    } else {
        Expansion::None
    }
}

pub fn subshell_expand(smsh: &mut Shell, line: String) -> Result<String>{
    eprintln!("Subshell expanding line:\n{}", line);

    let (rd, wr) = pipe()?;

    match unsafe{fork()?} {
        ForkResult::Parent { child: _, .. } => {
            close(wr)?;

            wait()?;

            let mut buf = String::new();

            unsafe {
                let mut rd = File::from_raw_fd(rd);
                rd.read_to_string(&mut buf)?;
            }

            Ok(buf)
        }
        ForkResult::Child => {
            smsh.clear_sources(); 

            close(rd)?;
            close(1 as RawFd)?;
            dup2(wr, 1 as RawFd)?;
            close(wr)?;

            let line = Line::new(line, 0, SourceKind::Subshell);
            smsh.push_source(BufferSource::new(vec![line]));

            while let Err(e) = smsh.run() {
                eprintln!("smsh (subshell): {}", e);
            }

            std::process::exit(0);
        }
    }
}
