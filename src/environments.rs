extern crate procfs;

use std::collections::HashMap;
use std::ffi::OsString;

/// The environment of all relavent processes:
/// - The process that launched the snap (no snap variables)
/// - The first process inside the snap (has snap variables)
/// - The current process (has snap variables, plus modifications not made by the snap that we want)
#[derive(Debug)]
pub struct All {
    external: HashMap<OsString, OsString>,
    snap: HashMap<OsString, OsString>,
    myself: HashMap<OsString, OsString>,
}

impl All {
    pub fn detect() -> Result<Self, Box<std::error::Error>> {
        let process = procfs::Process::myself()?;
        let my_env = process.environ()?;
        let snap_env = my_env.clone();
        let external_env = my_env.clone();
        Ok(Self {
            external: external_env,
            snap: snap_env,
            myself: my_env,
        })
    }
}
