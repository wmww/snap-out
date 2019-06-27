mod arguments;
mod command;

fn main() {
    let (cmd, args) = arguments::parse();
    let exit_code = command::run(cmd, &args);
    std::process::exit(exit_code);
}
