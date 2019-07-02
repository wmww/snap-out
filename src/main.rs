#[macro_use]
extern crate simple_error;

mod command;
mod environments;
mod options;
mod process;
mod variable;

use std::error::Error;
use std::ffi::OsString;

fn get_help_text() -> String {
    return format!(
        "Usage: {0} [COMMAND] [ARGUMENTS]...
       {0} [OPTION]

{1}

Options:
  -h, --help        Print this help message and exit
  -v, --version     Print the version and exit",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_DESCRIPTION")
    );
}

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

fn run_command_outside_snap(
    cmd: &str,
    args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
) -> i32 {
    match get_variables() {
        Ok(vars) => command::run(cmd, args, vars),
        Err(e) => {
            eprintln!("snap-out: {}, running in unmodified environment", e);
            let no_vars: Vec<(&str, Option<&str>)> = Vec::new();
            command::run(cmd, args, no_vars)
        }
    }
}

fn main() {
    let parsed = options::parse(std::env::args());
    match parsed {
        options::RunCommand { command, args } => {
            let exit_code = run_command_outside_snap(&command, args);
            std::process::exit(exit_code);
        }
        options::ShowHelp => {
            println!("{}", get_help_text());
            std::process::exit(0);
        }
        options::ShowVersion => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        options::Error(e) => {
            eprintln!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
        options::None => {
            eprintln!("No command to run, use --help for help");
            std::process::exit(0);
        }
    }
}
