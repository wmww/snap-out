extern crate procfs;

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::rc::Rc;

pub trait Process {
    fn get_parent(&self) -> Result<Option<Box<dyn Process>>, Box<dyn Error>>;
    fn get_env(&self) -> Rc<HashMap<OsString, OsString>>;
    fn get_pid(&self) -> i32;
}

pub struct ProcfsProcess {
    process: procfs::Process,
    env: Rc<HashMap<OsString, OsString>>,
}

impl ProcfsProcess {
    pub fn from_procfs_process(process: procfs::Process) -> Result<Self, Box<dyn Error>> {
        let env = Rc::new(process.environ()?);
        Ok(Self { process, env })
    }

    pub fn myself() -> Result<Self, Box<dyn Error>> {
        Self::from_procfs_process(procfs::Process::myself()?)
    }
}

impl Process for ProcfsProcess {
    fn get_parent(&self) -> Result<Option<Box<dyn Process>>, Box<dyn Error>> {
        let parent_pid = self.process.stat.ppid;
        if parent_pid <= 1 {
            Ok(None)
        } else {
            let parent_procfs = procfs::Process::new(parent_pid)?;
            let parent_process = Self::from_procfs_process(parent_procfs)?;
            Ok(Some(Box::new(parent_process)))
        }
    }

    fn get_env(&self) -> Rc<HashMap<OsString, OsString>> {
        self.env.clone()
    }

    fn get_pid(&self) -> i32 {
        self.process.pid()
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::rc::Rc;

    #[derive(Clone)]
    pub struct MockProcess {
        env: Rc<HashMap<OsString, OsString>>,
        parent: Option<Rc<MockProcess>>,
    }

    impl MockProcess {
        pub fn new(envs: Vec<Vec<(&str, &str)>>) -> MockProcess {
            assert!(envs.len() > 0);
            let mut process: Option<MockProcess> = None;
            for env in envs {
                let mut map = HashMap::new();
                for (var, val) in env {
                    map.insert(OsString::from(var), OsString::from(val));
                }
                process = Some(MockProcess {
                    env: Rc::new(map),
                    parent: process.map(|p| Rc::new(p)),
                });
            }
            process.unwrap()
        }
    }

    impl Process for MockProcess {
        fn get_parent(&self) -> Result<Option<Box<dyn Process>>, Box<dyn Error>> {
            Ok(self
                .parent
                .as_ref()
                .map(|p| Box::new((**p).clone()) as Box<dyn Process>))
        }

        fn get_env(&self) -> Rc<HashMap<OsString, OsString>> {
            self.env.clone()
        }

        fn get_pid(&self) -> i32 {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn assert_maps_to(map: &HashMap<OsString, OsString>, key: &OsStr, val: Option<&OsStr>) {
        let val_in_map = map.get(key);
        let expected_val = val.map(|v| OsString::from(v));
        if val_in_map != expected_val.as_ref() {
            panic!(
                "Key {:?} expected to map to {:?} but actually maps to {:?}",
                key, expected_val, val_in_map
            )
        }
    }

    #[test]
    fn can_get_myself_process() {
        let _ = ProcfsProcess::myself().expect("Could not open myself process");
    }

    #[test]
    fn can_get_myself_parent_process() {
        let myself = ProcfsProcess::myself().expect("Could not open myself process");
        let _ = myself
            .get_parent()
            .expect("Could not get parent process")
            .expect("Had no parent_process");
    }

    #[test]
    fn can_traverse_to_top() {
        let mut process = Some(Box::new(
            ProcfsProcess::myself().expect("Could not open myself process"),
        ) as Box<dyn Process>);
        for _ in 0..200 {
            process = match process {
                Some(p) => p.get_parent().expect("Could not get parent"),
                None => return, // We have successfully reached the toplevel process
            }
        }
        panic!("Could not find the toplevel process");
    }

    #[test]
    fn correctly_detects_env_vars() {
        let myself = ProcfsProcess::myself().expect("Could not open myself process");
        let mut map = (*myself.get_env()).clone();
        for (key, val) in std::env::vars_os() {
            assert_maps_to(&map, &key, Some(&val));
            map.remove(&key);
        }
        if !map.is_empty() {
            panic!("Detected map has the following extra values: {:#?}", map);
        }
    }

    /*
    // This test not only fails for unclear reasons, but also causes other tests to fail
    #[test]
    fn detects_var_is_in_myself_but_not_parent() {
        std::env::set_var("NOT_IN_PARENT", "1");
        let myself = ProcfsProcess::myself().expect("Could not open myself process");
        let myself_map = myself.get_env().expect("Could not get myself environment");
        let parent = myself.get_parent()
            .expect("Could not get parent process")
            .expect("Has no parent process");
        let parent_map = parent.get_env().expect("Could not get parent environment");
        assert_maps_to(&myself_map, OsStr::new("NOT_IN_PARENT"), Some(OsStr::new("1")));
        assert_maps_to(&parent_map, OsStr::new("NOT_IN_PARENT"), None);
        std::env::remove_var("NOT_IN_PARENT");
    }
    */
}
