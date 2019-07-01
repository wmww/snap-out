#[macro_use]
extern crate simple_error;

mod command;
mod environments;
mod options;
mod process;
mod variable;

use std::error::Error;
use std::ffi::OsString;

fn get_variables() -> Result<Vec<(OsString, Option<OsString>)>, Box<Error>> {
    let process = process::ProcfsProcess::myself()?;
    let environments = environments::All::detect(&process)?;
    let vars = environments
        .consolidate()
        .iter()
        .filter_map(|(name, val)| val.get_required_change().map(|v| (OsString::from(name), v)))
        .collect();
    Ok(vars)
}

fn main() {
    let (cmd, args) = options::handle();
    let exit_code = match get_variables() {
        Ok(vars) => command::run(&cmd, args, vars),
        Err(e) => {
            eprintln!("snap-out: {}, running in unmodified environment", e);
            let no_vars: Vec<(&str, Option<&str>)> = Vec::new();
            command::run(&cmd, args, no_vars)
        }
    };
    std::process::exit(exit_code);
}
