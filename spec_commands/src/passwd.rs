//! Unix passwd entries
//!
//! This does not actually run any external programs yet!
//! It is just a placeholder.

use std::collections::BTreeMap;
use std::sync::Mutex;

use log::*;
use users::{User, Group};

use spec_checks::user::LookupUser;
use spec_checks::group::LookupGroup;
use spec_exec::Command;

use super::GlobalOptions;


/// The **passwd non-command** examines the users and groups on the local
/// machine and caches the results.
#[derive(Debug)]
pub struct PasswdNonCommand {
    users:  BTreeMap<String, Mutex<Option<Option<User>>>>,
    groups: BTreeMap<String, Mutex<Option<Option<Group>>>>,
}

impl PasswdNonCommand {

    /// Creates a new non-command.
    pub fn create(_global_options: &impl GlobalOptions) -> Self {
        Self {
            users:  BTreeMap::new(),
            groups: BTreeMap::new(),
        }
    }

    /// Returns an iterator over the Commands contained within.
    pub fn commands(self) -> impl Iterator<Item=Command> {
        std::iter::empty()
    }
}

impl LookupUser for PasswdNonCommand {
    fn prime(&mut self, username: &str) {
        if ! self.users.contains_key(username) {
            debug!("Priming passwd with user {:?}", username);
            self.users.insert(username.to_owned(), Mutex::new(None));
        }
    }

    fn lookup_user(&self, username: &str) -> Option<User> {
        let mut slot = self.users.get(username).unwrap().lock().unwrap();
        let user = slot.get_or_insert_with(|| users::get_user_by_name(username));
        user.clone()
    }
}

impl LookupGroup for PasswdNonCommand {
    fn prime(&mut self, group_name: &str) {
        if ! self.groups.contains_key(group_name) {
            debug!("Priming passwd with group {:?}", group_name);
            self.groups.insert(group_name.to_owned(), Mutex::new(None));
        }
    }

    fn lookup_group(&self, group_name: &str) -> Option<Group> {
        let mut slot = self.groups.get(group_name).unwrap().lock().unwrap();
        let group = slot.get_or_insert_with(|| users::get_group_by_name(group_name));
        group.clone()
    }
}
