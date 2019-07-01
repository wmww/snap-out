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

    /// Returns the value the child process should have
    /// Attempts to remove the modifications to the environment made by the snap without effecting
    ///   the changes that were made after
    fn get_child_value(&self) -> Option<OsString> {
        if self.myself != self.snap {
            self.myself.clone()
        } else {
            self.external.clone()
        }
    }

    fn get_change_to(&self, value: Option<OsString>) -> Option<Option<OsString>> {
        if value == self.myself {
            None
        } else {
            Some(value)
        }
    }

    /// Gets the change that needs to be made to the current environment to escapt the snap
    /// Will return None if the variable can be left as-is
    /// Will return Some(None) if the variable needs to be cleared
    /// Will return Some(Some(...)) if the variable needs to be set
    pub fn get_required_change(&self) -> Option<Option<OsString>> {
        self.get_change_to(self.get_child_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_variable_set_in_snap() {
        assert_eq!(
            Variable::new(
                None,
                Some(OsString::from("foo")),
                Some(OsString::from("foo")),
            )
            .get_child_value(),
            None
        );
    }

    #[test]
    fn leaves_variable_set_in_myself() {
        assert_eq!(
            Variable::new(None, None, Some(OsString::from("foo")),).get_child_value(),
            Some(OsString::from("foo"))
        );
    }

    #[test]
    fn leaves_variable_always_set() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("foo")),
                Some(OsString::from("foo")),
                Some(OsString::from("foo")),
            )
            .get_child_value(),
            Some(OsString::from("foo"))
        );
    }

    #[test]
    fn restores_variable_cleared_in_snap() {
        assert_eq!(
            Variable::new(Some(OsString::from("foo")), None, None,).get_child_value(),
            Some(OsString::from("foo"))
        );
    }

    #[test]
    fn restores_variable_changed_in_snap() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("foo")),
                Some(OsString::from("bar")),
                Some(OsString::from("bar")),
            )
            .get_child_value(),
            Some(OsString::from("foo"))
        );
    }

    #[test]
    fn leaves_variable_changed_in_snap_then_again_in_myself() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("foo")),
                Some(OsString::from("bar")),
                Some(OsString::from("baz")),
            )
            .get_child_value(),
            Some(OsString::from("baz"))
        );
    }

    #[test]
    fn leaves_variable_set_in_snap_then_changed_in_myself() {
        assert_eq!(
            Variable::new(
                None,
                Some(OsString::from("bar")),
                Some(OsString::from("baz")),
            )
            .get_child_value(),
            Some(OsString::from("baz"))
        );
    }

    #[test]
    fn leaves_variable_changed_in_snap_then_cleared_in_myself() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("foo")),
                Some(OsString::from("bar")),
                None,
            )
            .get_child_value(),
            None
        );
    }

    #[test]
    fn leaves_variable_set_in_snap_then_cleared_in_myself() {
        assert_eq!(
            Variable::new(None, Some(OsString::from("bar")), None,).get_child_value(),
            None
        );
    }

    #[test]
    fn detects_no_required_change_to_value() {
        assert_eq!(
            Variable::new(None, None, Some(OsString::from("foo")),)
                .get_change_to(Some(OsString::from("foo"))),
            None
        );
        assert_eq!(
            Variable::new(
                Some(OsString::from("foo")),
                Some(OsString::from("foo")),
                Some(OsString::from("foo")),
            )
            .get_change_to(Some(OsString::from("foo"))),
            None
        );
        assert_eq!(
            Variable::new(
                Some(OsString::from("bar")),
                Some(OsString::from("baz")),
                Some(OsString::from("foo")),
            )
            .get_change_to(Some(OsString::from("foo"))),
            None
        );
    }

    #[test]
    fn detects_no_required_change_to_empty() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("bar")),
                Some(OsString::from("baz")),
                None,
            )
            .get_change_to(None),
            None
        );
        assert_eq!(Variable::new(None, None, None,).get_change_to(None), None);
    }

    #[test]
    fn detects_required_clear() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("bar")),
                Some(OsString::from("baz")),
                Some(OsString::from("foo")),
            )
            .get_change_to(None),
            Some(None)
        );
        assert_eq!(
            Variable::new(None, None, Some(OsString::from("")),).get_change_to(None),
            Some(None)
        );
    }

    #[test]
    fn detects_required_change_to_value() {
        assert_eq!(
            Variable::new(
                Some(OsString::from("f0o")),
                Some(OsString::from("foo")),
                Some(OsString::from("foo")),
            )
            .get_change_to(Some(OsString::from("bar"))),
            Some(Some(OsString::from("bar")))
        );
        assert_eq!(
            Variable::new(None, None, Some(OsString::from("foo")),)
                .get_change_to(Some(OsString::from("bar"))),
            Some(Some(OsString::from("bar")))
        );
        assert_eq!(
            Variable::new(None, None, None,).get_change_to(Some(OsString::from("bar"))),
            Some(Some(OsString::from("bar")))
        );
    }
}
