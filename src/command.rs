/// Runs a command and returns it's exit code
pub fn run(cmd: String, args: &[String]) -> i32 {
    let status = std::process::Command::new(cmd).args(args).status();
    match status {
        Ok(status) => {
            if let Some(exit_code) = status.code() {
                return exit_code;
            } else {
                eprintln!("{}: child process terminated without an exit code", env!("CARGO_PKG_NAME"));
                return 1;
            }
        }
        Err(error) => {
            eprintln!("{}: Failed to run child process: {}", env!("CARGO_PKG_NAME"), error);
            return 1;
        }
    }
}

