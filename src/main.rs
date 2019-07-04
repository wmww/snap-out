#[macro_use]
extern crate simple_error;

mod command;
mod debug;
mod environments;
mod options;
mod process;
mod variable;

use std::error::Error;
use std::ffi::OsString;

fn get_help_text() -> String {
    return format!(
        "Usage: {pkg} [COMMAND] [ARGUMENTS]...
       {pkg} [OPTION]

{desc}

Options:
  -h, --help        Print this help message and exit
  -v, --version     Print the version and exit
  -s, --script      Generate a script that sets up the environment and write it to stdout
                    Output consists of lines in the following two formats:
                      export VARIABLE=VALUE
                      unset VARIABLE

Environment variables:
  {debug_var:<17} If set, dump debugging information to {debug_path}
",
        pkg = env!("CARGO_PKG_NAME"),
        desc = env!("CARGO_PKG_DESCRIPTION"),
        debug_var = debug::DEBUG_ENV_VAR,
        debug_path = debug::DEBUG_DUMP_PATH,
    );
}

fn get_variables() -> Result<Vec<(OsString, Option<OsString>)>, Box<Error>> {
    let process = process::ProcfsProcess::myself()?;
    let environments = environments::All::detect(Box::new(process))?;
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
            eprintln!(
                "{}: {}, running in unmodified environment",
                env!("CARGO_PKG_NAME"),
                e
            );
            let no_vars: Vec<(&str, Option<&str>)> = Vec::new();
            command::run(cmd, args, no_vars)
        }
    }
}

fn varibale_list_to_setup_script(vars: &Vec<(OsString, Option<OsString>)>) -> String {
    use std::fmt::Write;
    let mut setters = String::new();
    let mut unsetters = String::new();
    for (name, value) in vars {
        match (name.to_str(), value) {
            (Some(name), Some(value)) =>
                match value.to_str() {
                    Some(value) => write!(&mut setters, "export {}={}\n", name, value).unwrap(),
                    None => eprintln!("Variable {:?} is not included because it's value {:?} includes invalid unicode", name, value.to_string_lossy()),
                },
            (Some(name), None) => write!(&mut unsetters, "unset {}\n", name).unwrap(),
            (None, _) => eprintln!("Variable {:?} is not included because it's name includes invalid unicode", name.to_string_lossy()),
        }
    }
    format!("{}{}", setters, unsetters)
}

fn print_setup_script() {
    match get_variables() {
        Ok(vars) => {
            let script = varibale_list_to_setup_script(&vars);
            println!("{}", &script)
        }
        Err(e) => eprintln!("{}: {}", env!("CARGO_PKG_NAME"), e),
    }
}

fn main() {
    let parsed = options::parse(std::env::args());
    let exit_code = match &parsed {
        options::RunCommand { command, args } => run_command_outside_snap(command, args),
        options::ShowHelp => {
            println!("{}", get_help_text());
            0
        }
        options::ShowVersion => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            0
        }
        options::ShowScript => {
            print_setup_script();
            0
        }
        options::Error(e) => {
            eprintln!("Error parsing arguments: {}", e);
            1
        }
        options::None => {
            eprintln!("No command to run, use --help for help");
            0
        }
    };
    debug::dump_info_if_needed(&parsed);
    std::process::exit(exit_code);
}
