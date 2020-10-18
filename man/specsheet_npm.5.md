% specsheet_npm(5) v0.1.0


NAME
====

specsheet_npm — The ‘npm’ check for specsheet


EXAMPLES
========

Check that a package is installed:

```toml
[[npm]]
package = 'yarn'
```

Check that a particular version of a package is installed:

```toml
[[npm]]
package = 'typescript'
version = '3.6.4'
```

Check that a package is _not_ installed:

```toml
[[npm]]
package = 'express'
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
