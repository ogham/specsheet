//! Filtering checks by their tags and types, and reordering the list after
//! they have all been loaded.


#[derive(PartialEq, Debug, Default)]
pub struct Filter {
    pub tags: TagsFilter,
    pub types: TypesFilter,
    pub order: RunningOrder,
}

#[derive(PartialEq, Debug, Default)]
pub struct TagsFilter {
    pub tags: Vec<String>,
    pub skip_tags: Vec<String>,
}

#[derive(PartialEq, Debug, Default)]
pub struct TypesFilter {
    pub types: Vec<String>,
    pub skip_types: Vec<String>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RunningOrder {
    ByType,
    Random,
}

impl Default for RunningOrder {
    fn default() -> Self {
        Self::ByType
    }
}


impl TagsFilter {

    /// Whether this filter should load a check with the given set of tags.
    ///
    /// This takes a slice of tags, instead of just one, because we need to
    /// know all the tags at once to determine whether to load a check.
    pub fn should_include_tags(&self, tags: &[impl AsRef<str>]) -> bool {
        if self.skip_tags.iter().any(|t1| tags.iter().any(|t2| t1 == t2.as_ref())) {
            false
        }
        else if self.tags.is_empty() {
            true
        }
        else {
            self.tags.iter().any(|t1| tags.iter().any(|t2| t1 == t2.as_ref()))
        }
    }
}

impl TypesFilter {

    /// Whether this filter should load checks of the given type.
    pub fn should_include_type(&self, ct: &str) -> bool {
        if self.skip_types.iter().any(|t| t == ct) {
            false
        }
        else if self.types.is_empty() {
            true
        }
        else {
            self.types.iter().any(|t| t == ct)
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    const EMPTY: &[&str] = &[];

    #[test]
    fn allow_all() {
        let filter = Filter::default();

        assert_eq!(true, filter.tags.should_include_tags(EMPTY));
        assert_eq!(true, filter.tags.should_include_tags(&[ "wibble" ]));
        assert_eq!(true, filter.types.should_include_type("apt"));
        assert_eq!(true, filter.types.should_include_type("systemd"));
    }

    #[test]
    fn only_certain_types_1() {
        let mut filter = Filter::default();
        filter.types.types.push("apt".into());

        assert_eq!(true,  filter.types.should_include_type("apt"));
        assert_eq!(false, filter.types.should_include_type("systemd"));
    }

    #[test]
    fn only_certain_types_2() {
        let mut filter = Filter::default();
        filter.types.skip_types.push("apt".into());

        assert_eq!(false, filter.types.should_include_type("apt"));
        assert_eq!(true,  filter.types.should_include_type("systemd"));
    }

    #[test]
    fn only_certain_tags_1() {
        let mut filter = Filter::default();
        filter.tags.tags.push("blue".into());

        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue" ]));
        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue", "green" ]));
        assert_eq!(false, filter.tags.should_include_tags(&[ "green" ]));
        assert_eq!(false, filter.tags.should_include_tags(EMPTY));
    }

    #[test]
    fn only_certain_tags_2() {
        let mut filter = Filter::default();
        filter.tags.tags.push("blue".into());
        filter.tags.tags.push("green".into());

        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue" ]));
        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue", "green" ]));
        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue", "green", "red" ]));
        assert_eq!(true,  filter.tags.should_include_tags(&[ "green" ]));
        assert_eq!(false, filter.tags.should_include_tags(&[ "red" ]));
        assert_eq!(false, filter.tags.should_include_tags(EMPTY));
    }

    #[test]
    fn only_certain_tags_3() {
        let mut filter = Filter::default();
        filter.tags.skip_tags.push("red".into());

        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue" ]));
        assert_eq!(false, filter.tags.should_include_tags(&[ "blue", "red" ]));
        assert_eq!(false, filter.tags.should_include_tags(&[ "red" ]));
        assert_eq!(true,  filter.tags.should_include_tags(EMPTY));
    }

    #[test]
    fn only_certain_tags_4() {
        let mut filter = Filter::default();
        filter.tags.tags.push("green".into());
        filter.tags.skip_tags.push("red".into());

        assert_eq!(false, filter.tags.should_include_tags(&[ "blue" ]));
        assert_eq!(false, filter.tags.should_include_tags(&[ "blue", "red" ]));
        assert_eq!(false, filter.tags.should_include_tags(&[ "blue", "green", "red" ]));
        assert_eq!(true,  filter.tags.should_include_tags(&[ "blue", "green" ]));
        assert_eq!(true,  filter.tags.should_include_tags(&[ "green" ]));
        assert_eq!(false, filter.tags.should_include_tags(EMPTY));
    }
}
