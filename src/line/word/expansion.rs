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
            if let Some(val) = smsh.get_user_variable(&word.text) {
                word.text = val;
            } else {
                word.text.clear();
            }

            Ok(())
        }
        Expansion::Environment => {
            if let Some(val) = env::var_os(&word.text) {
                word.text = val.into_string().unwrap_or("".to_string());
            } else {
                word.text.clear();
            }

            Ok(())
        }
        Expansion::Subshell => {
            word.text = subshell_expand(smsh, &word.text)?;

            Ok(())
        }
        _ => {
            Ok(())
        }
    }
}

pub fn get_expansion(text: &str) -> (String, Expansion) {
    if text.len() < 2 {
        (text.to_string(), Expansion::None)
    } else if text.starts_with("{") && text.ends_with("}") {
        let mut s = text[1..].to_string();
        s.pop();
        (s, Expansion::Variable)
    } else if text.starts_with("!{") && text.ends_with("}") {
        let mut s = text[2..].to_string();
        s.pop();
        (s, Expansion::Subshell)
    } else if text.starts_with("e{") && text.ends_with("}") {
        let mut s = text[2..].to_string();
        s.pop();
        (s, Expansion::Environment)
    } else if text[0..2].contains("{") {
        let mut s = text[2..].to_string();
        s.pop();
        (s, Expansion::Unknown)
    } else {
        (text.to_string(), Expansion::None)
    }
}

pub fn subshell_expand(smsh: &mut Shell, line: &str) -> Result<String>{
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

            let line = Line::new(line.to_string(), 0, SourceKind::Subshell);
            smsh.push_source(BufferSource::new(vec![line]));

            while let Err(e) = smsh.run() {
                eprintln!("smsh (subshell): {}", e);
            }

            std::process::exit(0);
        }
    }
}
