use super::process;
use super::variable::Variable;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::rc::Rc;

/// The environment variable used to determine if the snap variables have been set
const SNAP_SENTINEL_VAR: &str = "SNAP";

/// The environment of all relevant processes:
/// - The process that launched the snap (no snap variables)
/// - The first process inside the snap (has snap variables)
/// - The current process (has snap variables, plus modifications not made by the snap that we want)
#[derive(Debug)]
pub struct All {
    external: Rc<HashMap<OsString, OsString>>,
    snap: Rc<HashMap<OsString, OsString>>,
    myself: Rc<HashMap<OsString, OsString>>,
}

impl All {
    /// Detects relevant environments
    pub fn detect(mut process: Box<dyn process::Process>) -> Result<Self, Box<dyn Error>> {
        let mut envs = Vec::new();
        let mut pids = Vec::new();
        loop {
            envs.push(process.get_env());
            pids.push(process.get_pid());
            if !envs
                .last()
                .unwrap()
                .contains_key(OsStr::new(SNAP_SENTINEL_VAR))
            {
                break;
            }
            process = match process.get_parent()? {
                Some(p) => p,
                None => bail!(
                    "Could not find a process outside of the snap (searched PIDs {:?})",
                    pids
                ),
            }
        }
        if envs.len() < 2 {
            bail!("Not inside a snap");
        }
        let myself = envs[0].clone();
        let external = envs.last().unwrap().clone();
        let snap = envs.iter().rev().skip(1).next().unwrap().clone();
        Ok(All {
            external,
            snap,
            myself,
        })
    }

    pub fn consolidate(&self) -> HashMap<OsString, Variable> {
        let mut result = HashMap::new();
        for key in std::iter::empty()
            .chain(self.external.keys())
            .chain(self.snap.keys())
            .chain(self.myself.keys())
        {
            if !result.contains_key(key) {
                result.insert(
                    key.clone(),
                    Variable::new(
                        self.external.get(key).map(Clone::clone),
                        self.snap.get(key).map(Clone::clone),
                        self.myself.get(key).map(Clone::clone),
                    ),
                );
            }
        }
        result
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
        let result = All::detect(Box::new(process));
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
        let result = All::detect(Box::new(process));
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
        let result = All::detect(Box::new(process));
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
        let result = All::detect(Box::new(process));
        if let Ok(result) = result {
            panic!(
                "Should have been unable to find the edge of the snap: {:#?}",
                result
            )
        }
    }

    impl All {
        fn mock(
            external: Vec<(&str, &str)>,
            snap: Vec<(&str, &str)>,
            myself: Vec<(&str, &str)>,
        ) -> Self {
            fn vec2map(vec: Vec<(&str, &str)>) -> HashMap<OsString, OsString> {
                let mut map = HashMap::new();
                for (key, val) in vec {
                    map.insert(OsString::from(key), OsString::from(val));
                }
                map
            }
            All {
                external: Rc::new(vec2map(external)),
                snap: Rc::new(vec2map(snap)),
                myself: Rc::new(vec2map(myself)),
            }
        }
    }

    struct MockVariable {
        name: &'static str,
        external: Option<&'static str>,
        snap: Option<&'static str>,
        myself: Option<&'static str>,
    }

    fn mock_var_map(vec: Vec<MockVariable>) -> HashMap<OsString, Variable> {
        let mut map = HashMap::new();
        for i in vec {
            map.insert(
                OsString::from(i.name),
                Variable::new(
                    i.external.map(OsString::from),
                    i.snap.map(OsString::from),
                    i.myself.map(OsString::from),
                ),
            );
        }
        map
    }

    #[test]
    fn consolidates_single_variable() {
        let envs = All::mock(
            vec![("FOO", "external")],
            vec![("FOO", "snap")],
            vec![("FOO", "myself")],
        );
        assert_eq!(
            envs.consolidate(),
            mock_var_map(vec![MockVariable {
                name: "FOO",
                external: Some("external"),
                snap: Some("snap"),
                myself: Some("myself"),
            },])
        );
    }

    #[test]
    fn consolidates_varibale_only_in_one_env() {
        let envs = All::mock(
            vec![("FOO", "external")],
            vec![("BAR", "snap")],
            vec![("BAZ", "myself")],
        );
        assert_eq!(
            envs.consolidate(),
            mock_var_map(vec![
                MockVariable {
                    name: "FOO",
                    external: Some("external"),
                    snap: None,
                    myself: None,
                },
                MockVariable {
                    name: "BAR",
                    external: None,
                    snap: Some("snap"),
                    myself: None,
                },
                MockVariable {
                    name: "BAZ",
                    external: None,
                    snap: None,
                    myself: Some("myself"),
                },
            ])
        );
    }

    #[test]
    fn consolidates_multiple_variables() {
        let envs = All::mock(
            vec![("FOO", "1"), ("BAR", "1")],
            vec![("FOO", "1")],
            vec![("BAR", "2")],
        );
        assert_eq!(
            envs.consolidate(),
            mock_var_map(vec![
                MockVariable {
                    name: "FOO",
                    external: Some("1"),
                    snap: Some("1"),
                    myself: None,
                },
                MockVariable {
                    name: "BAR",
                    external: Some("1"),
                    snap: None,
                    myself: Some("2"),
                },
            ])
        );
    }
}
