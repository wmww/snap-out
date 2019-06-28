use super::process;
use std::collections::HashMap;
use std::error::Error;
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
    pub fn detect(process: &process::Process) -> Result<Self, Box<Error>> {
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
            process: &process::Process,
            env: HashMap<OsString, OsString>,
        ) -> Result<EnvResult, Box<std::error::Error>> {
            if !env.contains_key(OsStr::new(SNAP_SENTINEL_VAR)) {
                Ok(EnvResult::OutsideSnap(env))
            } else {
                let parent_process = match process.get_parent()? {
                    Some(p) => p,
                    None => bail!("Could not find a process outside of the snap"),
                };
                let parent_env = parent_process.get_env()?;
                match traverse_up_to_snap_boundry(&*parent_process, parent_env)? {
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

        let my_env = process.get_env()?;
        let result = traverse_up_to_snap_boundry(process, my_env.clone())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_maps_to(map: &HashMap<OsString, OsString>, key: &str, val: Option<&str>) {
        assert_eq!(
            map.get(&OsString::from(key)),
            val.map(|v| OsString::from(v)).as_ref()
        );
    }

    #[test]
    fn detects_correctly_when_at_snap_boundry() {
        let process = process::mock::MockProcess::new(vec![
            vec![("USER", "alice"), ("OUTSIDE", "1")],
            vec![("USER", "alice"), ("SNAP", "/snap"), ("INSIDE", "2")],
        ]);
        let result = All::detect(&process);
        if let Ok(envs) = result {
            assert_maps_to(&envs.external, "OUTSIDE", Some("1"));
            assert_maps_to(&envs.external, "INSIDE", None);

            assert_maps_to(&envs.snap, "OUTSIDE", None);
            assert_maps_to(&envs.snap, "INSIDE", Some("2"));

            assert_maps_to(&envs.myself, "OUTSIDE", None);
            assert_maps_to(&envs.myself, "INSIDE", Some("2"));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn detects_correctly_when_multiple_layers_in_snap() {
        let process = process::mock::MockProcess::new(vec![
            vec![("USER", "alice")],
            vec![("USER", "alice"), ("OUTSIDE", "1")],
            vec![("USER", "alice"), ("SNAP", "/snap"), ("EDGE", "2")],
            vec![("USER", "alice"), ("SNAP", "/snap")],
            vec![("USER", "alice"), ("SNAP", "/snap"), ("MYSELF", "3")],
        ]);
        let result = All::detect(&process);
        if let Ok(envs) = result {
            assert_maps_to(&envs.external, "OUTSIDE", Some("1"));
            assert_maps_to(&envs.snap, "EDGE", Some("2"));
            assert_maps_to(&envs.myself, "MYSELF", Some("3"));
        } else {
            assert!(false);
        }
    }

    #[test]
    fn errors_when_not_in_snap() {
        let process = process::mock::MockProcess::new(vec![
            vec![("USER", "alice")],
            vec![("USER", "alice"), ("DISPLAY", ":0")],
        ]);
        let result = All::detect(&process);
        if let Ok(result) = result {
            panic!("Should have detected it was not in the snap: {:#?}", result)
        }
    }

    #[test]
    fn errors_when_cant_escape_snap() {
        let process = process::mock::MockProcess::new(vec![
            vec![("USER", "alice"), ("SNAP", "/snap")],
            vec![("USER", "alice"), ("DISPLAY", ":0"), ("SNAP", "/snap")],
        ]);
        let result = All::detect(&process);
        if let Ok(result) = result {
            panic!(
                "Should have been unable to find the edge of the snap: {:#?}",
                result
            )
        }
    }
}
