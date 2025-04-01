use std::path::PathBuf;

use crate::export::resolve_path;

pub enum Commands {
    SetMaxPoints(u32),
    Export(PathBuf),
}

impl Commands {
    // parse the user input to a command.
    pub fn parse(raw_input: &str) -> Result<Commands, String> {
        let (cmd, args) = raw_input.split_at(1);
        match cmd.trim() {
            "p" => {
                if let Ok(points) = args.trim().parse::<u32>() {
                    Ok(Self::SetMaxPoints(points))
                } else {
                    Err(format!(
                        "ERROR: could not parse points from '{}' to u32.",
                        args
                    ))
                }
            }
            "e" => {
                if let Some(path) = resolve_path(args) {
                    Ok(Self::Export(path))
                } else {
                    Err(format!("Could not resovle path from '{}'", args))
                }
            }
            _ => Err(format!("ERROR: '{}' is an unknown command", cmd)),
        }
    }
}
