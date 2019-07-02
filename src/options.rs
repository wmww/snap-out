pub use Parsed::*;

#[derive(Debug, PartialEq)]
pub enum Parsed {
    RunCommand { command: String, args: Vec<String> },
    ShowScript,
    ShowHelp,
    ShowVersion,
    Error(String),
    None,
}

pub fn parse(args: impl std::iter::Iterator<Item = impl AsRef<str>>) -> Parsed {
    // skip the first arg, as it is just the current program
    let mut args = args.skip(1);
    let command: Option<String> = args.next().map(|s| String::from(s.as_ref()));
    if let Some(command) = command {
        if command.starts_with("-") {
            if command == "--help" || command == "-h" {
                Parsed::ShowHelp
            } else if command == "--version" || command == "-v" {
                Parsed::ShowVersion
            } else if command == "--script" || command == "-s" {
                Parsed::ShowScript
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
    fn parses_show_script() {
        assert_eq!(
            parse(vec!["snap-out", "--script"].iter()),
            Parsed::ShowScript,
        );
        assert_eq!(parse(vec!["snap-out", "-s"].iter()), Parsed::ShowScript,);
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
