extern crate procfs;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};

/// The environment variable used to determine if the snap variables have been set
const SNAP_SENTINEL_VAR: &str = "SNAP";

/// The environment of all relevant processes:
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
    /// Detects relevant environments
    pub fn detect() -> Result<Self, Box<std::error::Error>> {
        enum EnvResult {
            OutsideSnap(HashMap<OsString, OsString>),
            EdgeOfSnap {
                outside: HashMap<OsString, OsString>,
                inside: HashMap<OsString, OsString>,
            },
        }
        // Recursively traverses up the process tree until it finds one clean of snap variables
        // Returns the environment directly outside of the snap, or None if the current environment
        // is alerady outside the snap
        fn traverse_up_to_snap_boundry(
            process: &procfs::Process,
            env: HashMap<OsString, OsString>,
        ) -> Result<EnvResult, Box<std::error::Error>> {
            if !env.contains_key(OsStr::new(SNAP_SENTINEL_VAR)) {
                Ok(EnvResult::OutsideSnap(env))
            } else {
                let parent_pid = process.stat.ppid;
                if parent_pid == 0 {
                    bail!("Could not find a process outside of the snap");
                }
                let parent_process = procfs::Process::new(parent_pid)?;
                let parent_env = parent_process.environ()?;
                match traverse_up_to_snap_boundry(&parent_process, parent_env)? {
                    result @ EnvResult::EdgeOfSnap {
                        outside: _,
                        inside: _,
                    } => Ok(result),
                    EnvResult::OutsideSnap(parent_env) => Ok(EnvResult::EdgeOfSnap {
                        outside: parent_env,
                        inside: env,
                    }),
                }
            }
        }
        let process = procfs::Process::myself()?;
        let my_env = process.environ()?;
        let result = traverse_up_to_snap_boundry(&process, my_env.clone())?;
        match result {
            EnvResult::EdgeOfSnap { outside, inside } => Ok(All {
                external: outside,
                snap: inside,
                myself: my_env,
            }),
            EnvResult::OutsideSnap(_) => bail!("Not inside a snap"),
        }
    }
}
