use super::environments;
use super::options;
use super::process;
use std::error::Error;
use std::io::Write;

pub const DEBUG_ENV_VAR: &str = "SNAP_OUT_DEBUG";
pub const DEBUG_DUMP_PATH: &str = "/tmp/snap-out-debug.log";

fn get_debugging_info() -> Result<String, Box<Error>> {
    let process = process::ProcfsProcess::myself()?;
    let environments = environments::All::detect(Box::new(process))?;
    Ok(format!("{:#?}", environments))
}

fn dump_debugging_info(options: &options::Parsed) -> Result<(), Box<Error>> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(DEBUG_DUMP_PATH)?;
    let info = match get_debugging_info() {
        Ok(s) => s,
        Err(e) => format!("Error: {}", e),
    };
    let vars = super::get_variables()?;
    let script = super::varibale_list_to_setup_script(&vars);
    let time = match std::process::Command::new("date").output() {
        Ok(o) => String::from(String::from_utf8_lossy(&o.stdout)),
        Err(e) => String::from(e.description()),
    };
    let buffer = format!(
        "
{}
Task: {:?}
Time: {}
Detected Environments: {}

Needed variable modifications:
{}
_________________________________________
",
        env!("CARGO_PKG_NAME"),
        options,
        time,
        info,
        script,
    );
    file.write_all(buffer.as_bytes())?;
    Ok(())
}

pub fn dump_info_if_needed(options: &options::Parsed) {
    if let Ok(_) = std::env::var(DEBUG_ENV_VAR) {
        if let Err(e) = dump_debugging_info(options) {
            eprintln!(
                "{}: Failed to dump debugging info: {}",
                env!("CARGO_PKG_NAME"),
                e
            );
        }
    }
}
