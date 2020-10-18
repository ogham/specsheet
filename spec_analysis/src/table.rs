use std::collections::HashMap;
use std::path::PathBuf;

use crate::property::DataPoint;


pub struct AnalysisTable<'set, C> {
    paths: HashMap<PathBuf, MatchingChecks<'set, C>>,
    users: HashMap<String, MatchingChecks<'set, C>>,
    groups: HashMap<String, MatchingChecks<'set, C>>,
}

struct MatchingChecks<'set, C> {
    passes: Vec<&'set C>,
    fails: Vec<&'set C>,
}

pub struct Correlation<'tab> {
    pub property: DataPoint<'tab>,
    pub count: usize,
}

impl<'set, C> MatchingChecks<'set, C> {
    fn new() -> Self {
        Self {
            passes: Vec::new(),
            fails:  Vec::new(),
        }
    }
}

impl<'set, C> AnalysisTable<'set, C> {
    pub fn new() -> Self {
        Self {
            paths:  HashMap::new(),
            users:  HashMap::new(),
            groups: HashMap::new(),
        }
    }

    pub fn add(&mut self, check: &'set C, properties: impl Iterator<Item=DataPoint<'set>>, passed: bool) {
        for prop in properties {
            match prop {
                DataPoint::InvolvesPath(path) => {
                    if ! self.paths.contains_key(path) {
                        self.paths.insert(path.to_owned(), MatchingChecks::new());
                    }

                    let entry = self.paths.get_mut(path).unwrap();
                    if passed { entry.passes.push(check); }
                         else { entry.fails.push(check); }
                }

                DataPoint::InvolvesUser(user) => {
                    if ! self.users.contains_key(user) {
                        self.users.insert(user.to_owned(), MatchingChecks::new());
                    }

                    let entry = self.users.get_mut(user).unwrap();
                    if passed { entry.passes.push(check); }
                         else { entry.fails.push(check); }
                }

                DataPoint::InvolvesGroup(group) => {
                    if ! self.groups.contains_key(group) {
                        self.groups.insert(group.to_owned(), MatchingChecks::new());
                    }

                    let entry = self.groups.get_mut(group).unwrap();
                    if passed { entry.passes.push(check); }
                         else { entry.fails.push(check); }
                }
            }
        }
    }

    pub fn resolve_correlations<'tab>(&'tab self) -> Vec<Correlation<'tab>> {
        let mut correlations = Vec::new();

        for (path, path_checks) in &self.paths {
            if path_checks.passes.is_empty() && ! path_checks.fails.is_empty() {
                correlations.push(Correlation {
                    property: DataPoint::InvolvesPath(path),
                    count: path_checks.fails.len(),
                });
            }
        }

        for (user, user_checks) in &self.users {
            if user_checks.passes.is_empty() && ! user_checks.fails.is_empty() {
                correlations.push(Correlation {
                    property: DataPoint::InvolvesUser(user),
                    count: user_checks.fails.len(),
                });
            }
        }

        for (group, group_checks) in &self.groups {
            if group_checks.passes.is_empty() && ! group_checks.fails.is_empty() {
                correlations.push(Correlation {
                    property: DataPoint::InvolvesGroup(group),
                    count: group_checks.fails.len(),
                });
            }
        }

        correlations
    }
}
