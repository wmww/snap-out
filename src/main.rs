#[macro_use]
extern crate simple_error;

mod command;
mod environments;
mod options;
mod process;
mod variable;

fn main() {
    let (cmd, args) = options::handle();
    let process = process::ProcfsProcess::myself().expect("Failed to inspect process");
    let all_environments =
        environments::All::detect(&process).expect("Failed to detect snap environment");
    println!("environments: {:#?}", all_environments);
    let exit_code = command::run(&cmd, args);
    std::process::exit(exit_code);
}
