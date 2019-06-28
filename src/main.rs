mod arguments;
mod command;
mod environments;

fn main() {
    let (cmd, args) = arguments::parse();
    let all_environments = environments::All::detect().expect("Failed to detect snap environment");
    println!("environments: {:#?}", all_environments);
    let exit_code = command::run(cmd, &args);
    std::process::exit(exit_code);
}
