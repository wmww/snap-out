use std::ffi::OsStr;

/// Runs a command and returns it's exit code
pub fn run(
    cmd: &str,
    args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
    envs: impl IntoIterator<Item = (impl AsRef<OsStr>, Option<impl AsRef<OsStr>>)>,
) -> i32 {
    let mut command = std::process::Command::new(cmd);
    command.args(args);
    for (key, val) in envs {
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
                Vec::new() as Vec<&str>,
                Vec::new() as Vec<(&str, Option<&str>)>
            ),
            0
        );
    }

    #[test]
    fn running_false_returns_1() {
        assert_eq!(
            run(
                "false",
                Vec::new() as Vec<&str>,
                Vec::new() as Vec<(&str, Option<&str>)>
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
                Vec::new() as Vec<(&str, Option<&str>)>
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
                Vec::new() as Vec<(&str, Option<&str>)>
            ),
            1
        );
    }

    #[test]
    fn can_remove_variable() {
        let cmd = "bash";
        let args = vec!["-c", "[ -z $HOME ]"];
        assert_eq!(run(&cmd, &args, Vec::new() as Vec<(&str, Option<&str>)>), 1);
        assert_eq!(run(&cmd, &args, vec![("HOME", None as Option<&str>)]), 0);
    }

    #[test]
    fn can_add_variable() {
        let cmd = "bash";
        let args = vec!["-c", "[ -z $FOO ]"];
        assert_eq!(run(&cmd, &args, Vec::new() as Vec<(&str, Option<&str>)>), 0);
        assert_eq!(run(&cmd, &args, vec![("FOO", Some("BAR"))]), 1);
    }
}
