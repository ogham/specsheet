% specsheet_group(5) v0.1.0


NAME
====

specsheet_group — The ‘group’ check for specsheet


EXAMPLES
========

Check that a group exists:

```toml
[[group]]
group = 'vault'
```

Check that a group does _not_ exist:

```toml
[[group]]
group = 'vault'
state = 'missing'
```


PARAMETERS
==========

`group` (number or string)
: ID or name of a group on the local machine.

`state` (string)
: State of the group. This can be `present` or `missing`.


SEE ALSO
========

`specsheet(5)`
