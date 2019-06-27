/// In special cases (such as --help) it handles them and exits
/// In the normal case it returns the command + list-of-arguments that were sent in
fn parse_args() -> (String, Vec<String>) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        eprintln!("No command to run, use --help for help");
        std::process::exit(0);
    } else if args[1].starts_with("-") {
        let arg = &args[1];
        if arg == "--help" || arg == "-h" {
            println!(
                "Usage: {0} [COMMAND] [ARGUMENTS]...
       {0} --version
       {0} --help
{1}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_DESCRIPTION")
            );
            std::process::exit(0);
        } else if arg == "--version" || arg == "-v" {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        } else {
            eprintln!("Unknown argument {}", arg);
            std::process::exit(1);
        }
    } else {
        return (args[1].clone(), args[2..].into());
    }
}

fn main() {
    let (command, args) = parse_args();
    println!("{}, {:?}", command, args);
}
