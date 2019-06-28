/// In special cases (such as --help) it handles them and exits
/// In the normal case it returns the command + list-of-arguments that were sent in
pub fn handle() -> (String, Vec<String>) {
    match parse(std::env::args()) {
        Parsed::RunCommand { command, args } => (command, args),
        Parsed::ShowHelp => {
            println!("{}", get_help_text());
            std::process::exit(0);
        }
        Parsed::ShowVersion => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        Parsed::Error(e) => {
            eprintln!("Error parsing arguments: {}", e);
            std::process::exit(1);
        }
        Parsed::None => {
            eprintln!("No command to run, use --help for help");
            std::process::exit(0);
        }
    }
}

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

enum Parsed {
    RunCommand { command: String, args: Vec<String> },
    ShowHelp,
    ShowVersion,
    Error(String),
    None,
}

fn parse(args: impl std::iter::Iterator<Item = String>) -> Parsed {
    // skip the first arg, as it is just the current program
    let mut args = args.skip(1);
    let command: Option<String> = args.next();
    if let Some(command) = command {
        if command.starts_with("-") {
            if command == "--help" || command == "-h" {
                Parsed::ShowHelp
            } else if command == "--version" || command == "-v" {
                Parsed::ShowVersion
            } else {
                Parsed::Error(format!("Unknown argument {}", command))
            }
        } else {
            Parsed::RunCommand {
                command,
                args: args.collect(),
            }
        }
    } else {
        Parsed::None
    }
}
