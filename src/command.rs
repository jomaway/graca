use std::path::PathBuf;

use crate::export::resolve_path;

pub enum Commands {
    SetMaxPoints(u32),
    Export(PathBuf),
}

impl Commands {
    // parse the user input to a command.
    pub fn parse(raw_input: &str) -> Result<Commands, String> {
        let mut parts = raw_input.trim().split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let args: Vec<&str> = parts.collect(); // Rest are arguments

        if cmd.is_empty() {
            return Err(format!("ERROR: invalid input nothing found."));
        }

        // if args.len() < 1 {
        //     return Err(format!("ERROR: no arguments found"));
        // }

        match cmd {
            "set-points" => {
                if let Ok(points) = args[0].parse::<u32>() {
                    Ok(Self::SetMaxPoints(points))
                } else {
                    Err(format!(
                        "ERROR: could not parse points from '{}' to u32.",
                        args.join(",")
                    ))
                }
            }
            "export-to" => {
                if let Some(path) = resolve_path(args[0]) {
                    Ok(Self::Export(path))
                } else {
                    Err(format!("Could not resovle path from '{}'", args[0]))
                }
            }
            _ => Err(format!("ERROR: '{}' is an unknown command", cmd)),
        }
    }
}
