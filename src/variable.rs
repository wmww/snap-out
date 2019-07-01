use std::ffi::OsString;

/// The multiple values a single environment variable has held
/// See environments for what each means
#[derive(Debug, PartialEq)]
pub struct Variable {
    external: Option<OsString>,
    snap: Option<OsString>,
    myself: Option<OsString>,
}

impl Variable {
    pub fn new(
        external: Option<OsString>,
        snap: Option<OsString>,
        myself: Option<OsString>,
    ) -> Self {
        Variable {
            external,
            snap,
            myself,
        }
    }
}
