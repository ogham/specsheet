% specsheet_tap(5) v0.1.0


NAME
====

specsheet_tap — The ‘tap’ check for specsheet


EXAMPLES
========

Run a script that outputs TAP:

```toml
[[tap]]
shell = './my-test-script.sh'
```


PARAMETERS
==========

`shell` (string)
: Shell command to run and examine the output of.


SEE ALSO
========

`specsheet(5)`
