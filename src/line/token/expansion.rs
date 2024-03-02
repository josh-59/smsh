use crate::line::Line;
use crate::shell::Shell;
use crate::sources::{subshell::SubshellSource, SourceKind};
use anyhow::Result;

use super::{Expansion, Token};

use std::env;
use std::fs::File;
use std::io::Read;
use std::os::unix::io::{FromRawFd};

use nix::sys::wait::wait;
use nix::unistd::{close, dup2, fork, pipe, ForkResult};

pub fn expand(word: &mut Token, smsh: &mut Shell) -> Result<()> {
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
        Expansion::None => {
            Ok(())
        }
    }
}

pub fn get_expansion(text: &str) -> (String, Expansion) {
    if text.len() < 2 {
        (text.to_string(), Expansion::None)
    } else if text.starts_with('{') && text.ends_with('}') {
        let mut s = text[1..].to_string();
        s.pop();
        (s, Expansion::Variable)
    } else if text.starts_with("!{") && text.ends_with('}') {
        let mut s = text[2..].to_string();
        s.pop();
        (s, Expansion::Subshell)
    } else if text.starts_with("e{") && text.ends_with('}') {
        let mut s = text[2..].to_string();
        s.pop();
        (s, Expansion::Environment)
    } else {
        (text.to_string(), Expansion::None)
    }
}

pub fn subshell_expand(smsh: &mut Shell, line: &str) -> Result<String> {
    let (rd, wr) = pipe()?;

    match unsafe { fork()? } {
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
            close(1_i32)?;
            dup2(wr, 1_i32)?;
            close(wr)?;

            let line = Line::new(line.to_string(), 0, SourceKind::Subshell)?;
            smsh.push_source(SubshellSource::build_source(vec![line]));

            while let Err(e) = smsh.run() {
                eprintln!("smsh (subshell): {}", e);
            }

            std::process::exit(0);
        }
    }
}
