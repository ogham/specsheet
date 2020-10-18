% specsheet_homebrew(5) v0.1.0


NAME
====

specsheet_homebrew — The ‘homebrew’ check for specsheet


EXAMPLES
========

Check that a formula is installed:

```toml
[[homebrew]]
formula = 'cmatrix'
```

Check that a formula is _not_ installed:

```toml
[[homebrew]]
formula = 'cmatrix'
state = 'missing'
```


PARAMETERS
==========

`formula` (string)
: Name of the formula.

`state` (string)
: The state of the formula on the system. This can be `present` or `missing`.


SEE ALSO
========

`specsheet(5)`
