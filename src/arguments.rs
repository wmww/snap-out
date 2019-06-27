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

/// In special cases (such as --help) it handles them and exits
/// In the normal case it returns the command + list-of-arguments that were sent in
pub fn parse() -> (String, Vec<String>) {
    // skip the first arg, as it is just the current program
    let mut args = std::env::args().skip(1);
    let command: Option<String> = args.next();
    if let Some(command) = command {
        if command.starts_with("-") {
            if command == "--help" || command == "-h" {
                println!("{}", get_help_text());
                std::process::exit(0);
            } else if command == "--version" || command == "-v" {
                println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            } else {
                eprintln!("Unknown argument {}", command);
                std::process::exit(1);
            }
        } else {
            return (command, args.collect());
        }
    } else {
        eprintln!("No command to run, use --help for help");
        std::process::exit(0);
    }
}
