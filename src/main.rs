#[macro_use]
extern crate simple_error;

mod command;
mod debug;
mod environments;
mod manager;
mod options;
mod process;
mod variable;

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

fn main() {
    let parsed = options::parse(std::env::args());
    let mut manager = manager::Manager::new(parsed);
    let mut exit_code = 0;
    match &*manager.get_options() {
        options::RunCommand { command, args } => {
            exit_code = match manager.get_variables_to_change_lazy() {
                Ok(vars) => command::run(command, args, &*vars),
                Err(e) => {
                    eprintln!(
                        "{}: {}, running in unmodified environment",
                        env!("CARGO_PKG_NAME"),
                        e
                    );
                    command::run(command, args, command::NO_VARS)
                }
            }
        }
        options::ShowHelp => {
            println!("{}", get_help_text());
        }
        options::ShowVersion => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        options::ShowScript => {
            match manager.get_setup_script_lazy() {
                Ok(script) => println!("{}", &*script),
                Err(e) => eprintln!("{}: {}", env!("CARGO_PKG_NAME"), e),
            };
        }
        options::Error(e) => {
            eprintln!("Error parsing arguments: {}", e);
            exit_code = 1;
        }
        options::None => eprintln!("No command to run, use --help for help"),
    }
    debug::dump_info_if_needed(&mut manager);
    std::process::exit(exit_code);
}
