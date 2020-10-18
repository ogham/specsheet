% specsheet_user(5) v0.1.0


NAME
====

specsheet_user — The ‘user’ check for specsheet


EXAMPLES
========

Check that a user exists:

```toml
[[user]]
user = 'bethany'
```

Check that a user exists, has a specific login shell, and is a member of several groups:

```toml
[[user]]
user = 'bethany'
login_shell = '/usr/local/bin/fish'
groups = [ 'adm', 'storage' ]
```

Check that a user does _not_ exist:

```toml
[[user]]
user = 'bethany'
state = 'missing'
```


PARAMETERS
==========

`groups` (array of strings)
: Groups this user should be in, as an array of names.

`login_shell` (string)
: This user's login shell, as a path.

`state` (string)
: State of this user. This can be `present` or `missing`.

`user` (number or string)
: ID or name of a user on the local machine.


SEE ALSO
========

`specsheet(5)`
