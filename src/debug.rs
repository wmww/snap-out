use super::manager;
use std::error::Error;
use std::io::Write;
use std::rc::Rc;

pub const DEBUG_ENV_VAR: &str = "SNAP_OUT_DEBUG";
pub const DEBUG_DUMP_PATH: &str = "/tmp/snap-out-debug.log";

fn get_environment_info(manager: &mut manager::Manager) -> Result<String, Rc<dyn Error>> {
    Ok(format!("{:#?}", *manager.get_environments_lazy()?))
}

fn dump_debugging_info(manager: &mut manager::Manager) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(DEBUG_DUMP_PATH)?;
    let info = match get_environment_info(manager) {
        Ok(s) => s,
        Err(e) => format!("Error: {}", e),
    };
    let script = match manager.get_setup_script_lazy() {
        Ok(s) => s,
        Err(e) => Rc::new(format!("Error: {}", e)),
    };
    let time = match std::process::Command::new("date").output() {
        Ok(o) => String::from(String::from_utf8_lossy(&o.stdout)),
        Err(e) => e.to_string(),
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
        manager.get_options(),
        time,
        info,
        script,
    );
    file.write_all(buffer.as_bytes())?;
    Ok(())
}

pub fn dump_info_if_needed(manager: &mut manager::Manager) {
    if let Ok(_) = std::env::var(DEBUG_ENV_VAR) {
        if let Err(e) = dump_debugging_info(manager) {
            eprintln!(
                "{}: Failed to dump debugging info: {}",
                env!("CARGO_PKG_NAME"),
                e
            );
        }
    }
}
