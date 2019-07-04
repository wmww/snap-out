use super::*;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

type GenericResult<T> = Result<Rc<T>, Rc<Error>>;
type CachedResult<T> = Option<GenericResult<T>>;

pub struct Manager {
    options: Rc<options::Parsed>,
    environments: CachedResult<environments::All>,
    variables: CachedResult<HashMap<OsString, variable::Variable>>,
    variables_to_change: CachedResult<Vec<(OsString, Option<OsString>)>>,
    setup_script: CachedResult<String>,
}

impl Manager {
    pub fn new(options: options::Parsed) -> Manager {
        Manager {
            options: Rc::new(options),
            environments: None,
            variables: None,
            variables_to_change: None,
            setup_script: None,
        }
    }

    fn init_environments(&self) -> GenericResult<environments::All> {
        let process = process::ProcfsProcess::myself()?;
        let environments = environments::All::detect(Box::new(process))?;
        Ok(Rc::new(environments))
    }

    fn init_variables(&mut self) -> GenericResult<HashMap<OsString, variable::Variable>> {
        Ok(Rc::new(self.get_environments_lazy()?.consolidate()))
    }

    fn init_variables_to_change(&mut self) -> GenericResult<Vec<(OsString, Option<OsString>)>> {
        Ok(Rc::new(
            self.get_variables_lazy()?
                .iter()
                .filter_map(|(name, val)| {
                    val.get_required_change().map(|v| (OsString::from(name), v))
                })
                .collect(),
        ))
    }

    fn init_setup_script(&mut self) -> GenericResult<String> {
        use std::fmt::Write;
        let vars = self.get_variables_to_change_lazy()?;
        let mut setters = String::new();
        let mut unsetters = String::new();
        for (name, value) in &*vars {
            match (name.to_str(), value) {
                (Some(name), Some(value)) =>
                    match value.to_str() {
                        Some(value) => write!(&mut setters, "export {}={}\n", name, value).unwrap(),
                        None => eprintln!("Variable {:?} is not included because it's value {:?} includes invalid unicode", name, value.to_string_lossy()),
                    },
                (Some(name), None) => write!(&mut unsetters, "unset {}\n", name).unwrap(),
                (None, _) => eprintln!("Variable {:?} is not included because it's name includes invalid unicode", name.to_string_lossy()),
            }
        }
        Ok(Rc::new(format!("{}{}", setters, unsetters)))
    }

    pub fn get_options(&self) -> Rc<options::Parsed> {
        self.options.clone()
    }

    pub fn get_environments_lazy(&mut self) -> GenericResult<environments::All> {
        if let None = self.environments {
            self.environments = Some(self.init_environments());
        }
        self.environments.as_ref().unwrap().clone()
    }

    pub fn get_variables_lazy(&mut self) -> GenericResult<HashMap<OsString, variable::Variable>> {
        if let None = self.variables {
            self.variables = Some(self.init_variables());
        }
        self.variables.as_ref().unwrap().clone()
    }

    pub fn get_variables_to_change_lazy(
        &mut self,
    ) -> GenericResult<Vec<(OsString, Option<OsString>)>> {
        if let None = self.variables_to_change {
            self.variables_to_change = Some(self.init_variables_to_change());
        }
        self.variables_to_change.as_ref().unwrap().clone()
    }

    pub fn get_setup_script_lazy(&mut self) -> GenericResult<String> {
        if let None = self.setup_script {
            self.setup_script = Some(self.init_setup_script());
        }
        self.setup_script.as_ref().unwrap().clone()
    }
}
