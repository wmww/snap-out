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

#[derive(Debug, PartialEq)]
enum Parsed {
    RunCommand { command: String, args: Vec<String> },
    ShowHelp,
    ShowVersion,
    Error(String),
    None,
}

fn parse(args: impl std::iter::Iterator<Item = impl AsRef<str>>) -> Parsed {
    // skip the first arg, as it is just the current program
    let mut args = args.skip(1);
    let command: Option<String> = args.next().map(|s| String::from(s.as_ref()));
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
                args: args.map(|s| String::from(s.as_ref())).collect(),
            }
        }
    } else {
        Parsed::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_command_with_no_args() {
        assert_eq!(
            parse(vec!["snap-out", "ls"].iter()),
            Parsed::RunCommand {
                command: "ls".to_owned(),
                args: vec![],
            }
        );
    }

    #[test]
    fn parses_command_with_args() {
        assert_eq!(
            parse(vec!["snap-out", "ls", "..", "-a"].iter()),
            Parsed::RunCommand {
                command: "ls".to_owned(),
                args: vec!["..", "-a"].iter().map(|s| s.to_string()).collect(),
            }
        );
    }

    #[test]
    fn parses_show_help() {
        assert_eq!(parse(vec!["snap-out", "--help"].iter()), Parsed::ShowHelp,);
        assert_eq!(parse(vec!["snap-out", "-h"].iter()), Parsed::ShowHelp,);
    }

    #[test]
    fn parses_show_version() {
        assert_eq!(
            parse(vec!["snap-out", "--version"].iter()),
            Parsed::ShowVersion,
        );
        assert_eq!(parse(vec!["snap-out", "-v"].iter()), Parsed::ShowVersion,);
    }

    #[test]
    fn errors_on_bad_arg() {
        match parse(vec!["snap-out", "--bad"].iter()) {
            Parsed::Error(_) => (),
            result @ _ => panic!(
                "Should have been an error, but instead returned {:?}",
                result
            ),
        };
        match parse(vec!["snap-out", "-x"].iter()) {
            Parsed::Error(_) => (),
            result @ _ => panic!(
                "Should have been an error, but instead returned {:?}",
                result
            ),
        };
    }

    #[test]
    fn none_on_no_args() {
        assert_eq!(parse(vec!["snap-out"].iter()), Parsed::None,);
        assert_eq!(parse((vec![] as Vec<&str>).iter()), Parsed::None,);
    }
}
