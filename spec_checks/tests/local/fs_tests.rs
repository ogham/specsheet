use super::*;
use spec_checks::fs::{FilesystemCheck};
use spec_checks::read::Rewrites;
use pretty_assertions::assert_eq;


// ---- regular tests ----

#[test]
fn file_exists() {
    let check = FilesystemCheck::read(&toml! {
        path = "/etc/nginx/nginx.conf"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/etc/nginx/nginx.conf’ exists");
}

#[test]
fn file_is_missing() {
    let check = FilesystemCheck::read(&toml! {
        path = "/home/balrog"
        state = "absent"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/home/balrog’ does not exist");
}

#[test]
fn file_is_regular_file() {
    let check = FilesystemCheck::read(&toml! {
        path = "/bin/chmod"
        kind = "file"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/bin/chmod’ is a regular file");
}

#[test]
fn file_is_directory() {
    let check = FilesystemCheck::read(&toml! {
        path = "/home/balrog"
        kind = "directory"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/home/balrog’ is a directory");
}

#[test]
fn file_is_link() {
    let check = FilesystemCheck::read(&toml! {
        path = "/etc"
        kind = "symlink"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/etc’ is a symbolic link");
}

#[test]
fn file_links_to() {
    let check = FilesystemCheck::read(&toml! {
        path = "~/.psqlrc"
        link_target = "~/Configs/psqlrc.sql"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘~/.psqlrc’ is a symbolic link to ‘~/Configs/psqlrc.sql’");
}

#[test]
fn file_permissions_octal() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        permissions = "0600"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ has permissions ‘0600’");
}

#[test]
fn file_owner_id() {
    let check = FilesystemCheck::read(&toml! {
        path = "/opt/backups"
        owner = 503
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/opt/backups’ has owner ID ‘503’");
}

#[test]
fn file_owner_name() {
    let check = FilesystemCheck::read(&toml! {
        path = "/opt/backups"
        owner = "admin"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/opt/backups’ has owner ‘admin’");
}

#[test]
fn file_group_id() {
    let check = FilesystemCheck::read(&toml! {
        path = "/opt/backups"
        group = 13
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/opt/backups’ has group ID ‘13’");
}

#[test]
fn file_group_name() {
    let check = FilesystemCheck::read(&toml! {
        path = "/opt/backups"
        group = "backup"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/opt/backups’ has group ‘backup’");
}

#[test]
fn file_follow() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        kind = "file"
        follow = true
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ is a regular file (following symlinks)");
}

#[test]
fn file_contents_string() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        contents = { string = "#!/usr/bin/env ruby" }
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ contains string ‘#!/usr/bin/env ruby’");
}

#[test]
fn file_contents_file() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        contents = { file = "output.txt" }
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ has the contents of file ‘output.txt’");
}

#[test]
fn file_contents_regex() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        contents = { regex = "..." }
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ matches regex ‘/.../’");
}

#[test]
fn file_contents_empty() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        contents = { empty = true }
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ is empty");
}

#[test]
fn file_contents_non_empty() {
    let check = FilesystemCheck::read(&toml! {
        path = "/usr/local/bin/script.sh"
        contents = { empty = false }
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/usr/local/bin/script.sh’ is not empty");
}


// ---- parameter combinations ----

#[test]
fn file_exists_explicitly() {
    let check = FilesystemCheck::read(&toml! {
        path = "/etc/nginx/nginx.conf"
        state = "present"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/etc/nginx/nginx.conf’ exists");
}

#[test]
fn file_with_contents() {
    let check = FilesystemCheck::read(&toml! {
        path = "/etc/ssh/sshd_config"
        kind = "file"
        contents = { regex = "^PermitRootLogin no" }
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/etc/ssh/sshd_config’ is a regular file that matches regex ‘/^PermitRootLogin no/’");
}

#[test]
fn file_link_specified_twice() {
    let check = FilesystemCheck::read(&toml! {
        path = "~/.psqlrc"
        kind = "symlink"
        link_target = "~/Configs/psqlrc.sql"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘~/.psqlrc’ is a symbolic link to ‘~/Configs/psqlrc.sql’");
}

#[test]
fn file_user_and_group() {
    let check = FilesystemCheck::read(&toml! {
        path = "/opt/backups"
        owner = "river"
        group = "bed"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘/opt/backups’ has owner ‘river’ and group ‘bed’");
}

#[test]
fn file_directory_permissions() {
    let check = FilesystemCheck::read(&toml! {
        path = "~/Scripts/vendor"
        kind = "directory"
        permissions = "+x"
    }, &Rewrites::new()).unwrap();

    assert_eq!(check.to_string(),
               "File ‘~/Scripts/vendor’ is a directory and is executable");
}


// ---- invalid parameter combinations errors ----

#[test]
fn err_absent_but_kind() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        kind = "directory"
        state = "absent"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘kind’ is inappropriate when parameter ‘state’ is ‘\"absent\"’");
}

#[test]
fn err_both_mode_and_permissions() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        mode = "0755"
        permissions = "0644"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameters ‘permissions’ and ‘mode’ are both given (they are aliases)");
}

#[test]
fn err_symlink_but_wrong_kind() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        kind = "file"
        link_target = "/somewhere"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘link_target’ is inappropriate when parameter ‘kind’ is ‘\"file\"’");
}

#[test]
fn err_symlink_but_absent() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        state = "absent"
        link_target = "/somewhere"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘link_target’ is inappropriate when parameter ‘state’ is ‘\"absent\"’");
}

#[test]
fn err_directory_kind_but_contents() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        kind = "directory"
        contents = { empty = true }
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘contents’ is inappropriate when parameter ‘kind’ is ‘\"directory\"’");
}

#[test]
fn err_symlink_kind_but_contents() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        kind = "symlink"
        contents = { empty = true }
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘contents’ is inappropriate when parameter ‘kind’ is ‘\"symlink\"’");
}

#[test]
fn err_symlink_parameter_but_contents() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        link_target = "/somewhere"
        contents = { empty = true }
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘contents’ is inappropriate when parameter ‘link_target’ is given");
}

#[test]
fn err_absent_but_contents() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        state = "absent"
        contents = { empty = true }
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘contents’ is inappropriate when parameter ‘state’ is ‘\"absent\"’");
}

#[test]
fn err_permissions_number_check() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        permissions = 0644
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘permissions’ value ‘644’ is invalid (it must be a string)");
}

#[test]
fn err_mode_number_check() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        mode = 0644
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘mode’ value ‘644’ is invalid (it must be a string)");
}


// ---- invalid string errors ----

#[test]
fn err_invalid_permissions() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        permissions = "yes"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘permissions’ value ‘\"yes\"’ is invalid (it must be a permissions string)");
}
#[test]
fn err_invalid_kind() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        kind = "blob"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘kind’ value ‘\"blob\"’ is invalid (it must be ‘file’ or ‘directory’ or ‘symlink’)");
}

#[test]
fn err_invalid_state() {
    let check = FilesystemCheck::read(&toml! {
        path = "/something"
        state = "ethereal"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘\"ethereal\"’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- empty string errors ----

#[test]
fn err_empty_path() {
    let check = FilesystemCheck::read(&toml! {
        path = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘path’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_link_target() {
    let check = FilesystemCheck::read(&toml! {
        path = "/möbius"
        link_target = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘link_target’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_owner() {
    let check = FilesystemCheck::read(&toml! {
        path = "/möbius"
        owner = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘owner’ value ‘\"\"’ is invalid (it must not be empty)");
}

#[test]
fn err_empty_group() {
    let check = FilesystemCheck::read(&toml! {
        path = "/möbius"
        group = ""
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘group’ value ‘\"\"’ is invalid (it must not be empty)");
}


// ---- wrong type errors ----

#[test]
fn err_invalid_contents_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        contents = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘contents’ value ‘[]’ is invalid (it must be a table)");
}

#[test]
fn err_invalid_follow_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        follow = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘follow’ value ‘[]’ is invalid (it must be a boolean)");
}

#[test]
fn err_invalid_group_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        group = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘group’ value ‘[]’ is invalid (it must be a string or a number)");
}

#[test]
fn err_invalid_kind_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        kind = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘kind’ value ‘[]’ is invalid (it must be ‘file’ or ‘directory’ or ‘symlink’)");
}

#[test]
fn err_invalid_link_target_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        link_target = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘link_target’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_owner_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        owner = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘owner’ value ‘[]’ is invalid (it must be a string or a number)");
}

#[test]
fn err_invalid_path_type() {
    let check = FilesystemCheck::read(&toml! {
        path = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘path’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_mode_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        mode = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘mode’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_permissions_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        permissions = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘permissions’ value ‘[]’ is invalid (it must be a string)");
}

#[test]
fn err_invalid_state_type() {
    let check = FilesystemCheck::read(&toml! {
        path = "/esc/arcade"
        state = []
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘state’ value ‘[]’ is invalid (it must be ‘present’ or ‘missing’)");
}


// ---- general read errors ----

#[test]
fn err_empty_document() {
    let check = FilesystemCheck::read(&Map::new().into(), &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘path’ is missing");
}

#[test]
fn err_unknown_parameter() {
    let check = FilesystemCheck::read(&toml! {
        oaehusnaeothunaoehu = "ntsehousitnhoenith"
    }, &Rewrites::new()).unwrap_err();

    assert_eq!(check.to_string(),
               "Parameter ‘oaehusnaeothunaoehu’ is unknown");
}
