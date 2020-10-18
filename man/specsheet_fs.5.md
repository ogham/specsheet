% specsheet_fs(5) v0.1.0


NAME
====

specsheet_fs — The ‘fs’ check for specsheet


EXAMPLES
========

Check that a file exists:

```toml
[[fs]]
path = '/etc/nginx/nginx.conf'
kind = 'file'
```

Check that a file contains the right content:

```toml
[[fs]]
path = '/etc/ssh/sshd_config'
kind = 'file'
contents = { regex = '^PermitRootLogin no' }
```

Check that a file does _not_ exist:

```toml
[[fs]]
path = '/home/balrog'
kind = 'absent'
```

Check that a file is a symlink that links to a certain path:

```toml
[[fs]]
path = '~/.psqlrc'
link_target = '~/Configs/psqlrc.sql'
```

Check that a file has the right permissions:

```toml
[[fs]]
path = '/usr/local/bin/script.sh'
permissions = 'u+x'
```

Check that a file has the right owner or group:

```toml
[[fs]]
path = '/var/log/syslog'
owner = 'syslog'
group = 'adm'
```

Check multiple things at once:

```toml
[[fs]]
path = '/etc/dh/dhparam_2048.pem'
kind = 'file'
owner = 'root'
group = 'root'
contents = { empty = false }
```


PARAMETERS
==========

`contents` (content)
: The content that the file should have.

`follow` (boolean)
: Whether to follow symlinks (default: `false`)

`group` (number or string)
: ID or name of the group of the file.

`kind` (string)
: The kind of file that exists at the paint. This can be `file` or `directory` or `symlink`.

`link_target` (string)
: The target of this file as a symlink.

`owner` (number or string)
: ID or name of the user that owns this file.

`path` (string)
: The path to the local file on disk that is being checked.

`permissions` (string)
: The permissions of the file. (alias: `mode`)

`state` (string)
: The state of the file at this path. This can be `present` or `missing`.


SEE ALSO
========

`specsheet(5)`
