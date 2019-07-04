use std::borrow::Borrow;
use std::ffi::{OsStr, OsString};

#[allow(dead_code)]
pub const NO_ARGS: std::iter::Empty<OsString> = std::iter::empty();
#[allow(dead_code)]
pub const NO_VARS: std::iter::Empty<(OsString, Option<OsString>)> = std::iter::empty();

/// Runs a command and returns it's exit code
pub fn run(
    cmd: &str,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    envs: impl IntoIterator<Item = impl Borrow<(OsString, Option<OsString>)>>,
) -> i32 {
    let mut command = std::process::Command::new(cmd);
    command.args(args);
    for var in envs {
        let (key, val) = var.borrow();
        if let Some(val) = val {
            command.env(key, val);
        } else {
            command.env_remove(key);
        }
    }
    // actually run the command
    let status = command.status();
    match status {
        Ok(status) => {
            if let Some(exit_code) = status.code() {
                return exit_code;
            } else {
                eprintln!(
                    "{}: child process terminated without an exit code",
                    env!("CARGO_PKG_NAME")
                );
                return 1;
            }
        }
        Err(error) => {
            eprintln!(
                "{}: Failed to run child process: {}",
                env!("CARGO_PKG_NAME"),
                error
            );
            return 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_true_returns_0() {
        assert_eq!(
            run(
                "true",
                NO_ARGS,
                NO_VARS,
            ),
            0
        );
    }

    #[test]
    fn running_false_returns_1() {
        assert_eq!(
            run(
                "false",
                NO_ARGS,
                NO_VARS,
            ),
            1
        );
    }

    #[test]
    fn evaluates_true_bash_exp() {
        assert_eq!(
            run(
                "bash",
                vec!["-c", "[ 3 -eq 3 ]"],
                NO_VARS,
            ),
            0
        );
    }

    #[test]
    fn evaluates_false_bash_exp() {
        assert_eq!(
            run(
                "bash",
                vec!["-c", "[ 3 -eq 5 ]"],
                NO_VARS,
            ),
            1
        );
    }

    #[test]
    fn can_remove_variable() {
        let cmd = "bash";
        let args = vec!["-c", "[ -z $HOME ]"];
        let vars = vec![(OsString::from("HOME"), None)];
        assert_eq!(run(&cmd, &args, NO_VARS), 1);
        assert_eq!(run(&cmd, &args, vars), 0);
    }

    #[test]
    fn can_add_variable() {
        let cmd = "bash";
        let args = vec!["-c", "[ -z $FOO ]"];
        let vars = vec![(OsString::from("FOO"), Some(OsString::from("BAR")))];
        assert_eq!(run(&cmd, &args, NO_VARS), 0);
        assert_eq!(run(&cmd, &args, vars), 1);
    }
}
