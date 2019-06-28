extern crate procfs;

use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;

pub trait Process {
    fn get_parent(&self) -> Result<Option<Box<Process>>, Box<Error>>;
    fn get_env(&self) -> Result<HashMap<OsString, OsString>, Box<Error>>;
}

pub struct ProcfsProcess {
    process: procfs::Process,
}

impl ProcfsProcess {
    pub fn myself() -> Result<Self, Box<Error>> {
        Ok(Self {
            process: procfs::Process::myself()?,
        })
    }
}

impl Process for ProcfsProcess {
    fn get_parent(&self) -> Result<Option<Box<Process>>, Box<Error>> {
        let parent_pid = self.process.stat.ppid;
        if parent_pid == 0 {
            Ok(None)
        } else {
            Ok(Some(Box::new(ProcfsProcess {
                process: procfs::Process::new(parent_pid)?,
            })))
        }
    }

    fn get_env(&self) -> Result<HashMap<OsString, OsString>, Box<Error>> {
        Ok(self.process.environ()?)
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
        fn get_parent(&self) -> Result<Option<Box<Process>>, Box<Error>> {
            Ok(self
                .parent
                .as_ref()
                .map(|p| Box::new((**p).clone()) as Box<Process>))
        }

        fn get_env(&self) -> Result<HashMap<OsString, OsString>, Box<Error>> {
            Ok((*self.env).clone())
        }
    }
}
