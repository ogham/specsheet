% specsheet_cmd(5) v0.1.0


NAME
====

specsheet_cmd — The ‘cmd’ check for specsheet


EXAMPLES
========

Check that running a command produces certain output:

```toml
[[cmd]]
shell = 'git config --global core.excludesfile'
stdout = { regex = '^~/\.gitignore_global' }
```

Check that a command exits with a certain status:

```toml
[[cmd]]
shell = 'nomad status'
status = 0
```


PARAMETERS
==========

`environment` (table)
: Mapping of environment variable names to values, to be set for the process.

`shell` (string)
: The shell command to run.

`status` (number)
: The command’s expected exit status.

`stdout` (content)
: The content of the process’s standard output stream.

`stderr` (content)
: The content of the process’s standard error stream.


SEE ALSO
========

`specsheet(5)`
