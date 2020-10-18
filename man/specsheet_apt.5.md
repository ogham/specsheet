% specsheet_apt(5) v0.1.0


NAME
====

specsheet_apt — The ‘apt’ check for specsheet


EXAMPLES
========

Check that a package is installed:

```toml
[[apt]]
package = 'ufw'
```

Check that a particular version is installed:

```toml
[[apt]]
package = 'tmux'
version = '2.8-3'
```

Check that a package is _not_ installed:

```toml
[[apt]]
package = 'httpd'
state = 'missing'
```


PARAMETERS
==========

`package` (string)
: Name of the package.

`state` (string)
: State of the package. This can be `present` or `missing`.

`version` (string)
: If installed, the version that should be present.


SEE ALSO
========

`specsheet(5)`
