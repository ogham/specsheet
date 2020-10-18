% specsheet_gem(5) v0.1.0


NAME
====

specsheet_gem — The ‘gem’ check for specsheet


EXAMPLES
========

Check that a gem is installed:

```toml
[[gem]]
package = 'pry'
```

Check that a particular version of a gem is installed:

```toml
[[gem]]
package = 'bundler'
version = '1.17.2'
```

Check that a gem is _not_ installed:

```toml
[[gem]]
package = 'rvm'
state = 'missing'
```


PARAMETERS
==========

`package` (string)
: Name of the gem.

`state` (string)
: State of the gem. This can be `present` or `missing`.

`version` (string)
: If installed, the version that should be present.


SEE ALSO
========

`specsheet(5)`
