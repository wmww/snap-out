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
        return Ok(self.process.environ()?);
    }
}
