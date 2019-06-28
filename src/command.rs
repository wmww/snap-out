/// Runs a command and returns it's exit code
pub fn run<I, S>(cmd: &str, args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let status = std::process::Command::new(cmd).args(args).status();
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
        assert_eq!(run("true", vec![] as Vec<&str>), 0);
    }

    #[test]
    fn running_false_returns_1() {
        assert_eq!(run("false", vec![] as Vec<&str>), 1);
    }

    #[test]
    fn evaluates_true_bash_exp() {
        assert_eq!(run("bash", vec!["-c", "[ 3 -eq 3 ]"]), 0);
    }

    #[test]
    fn evaluates_false_bash_exp() {
        assert_eq!(run("bash", vec!["-c", "[ 3 -eq 5 ]"]), 1);
    }
}
