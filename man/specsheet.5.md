% specsheet(5) v0.1.0


NAME
====

specsheet — The check document file format for specsheet


FULL-FEATURED EXAMPLE
=====================

Here is an example check document:

```toml
[[apt]]
package = 'cmatrix'
state = 'installed'

[[fs]]
path = '/etc/ssh/sshd_config'
contents = { string = 'PermitRootLogin no' }
```

The above configuration file specifies, using TOML syntax, that the `cmatrix` apt package should be installed, and the `/etc/ssh/sshd_config` file should exit with the given string somewhere in its contents.


INTRODUCTION
============

specsheet's check documents are written in TOML.


THE TOML SCHEMA
===============

The configuration file must have one top-level property, `program`, which must be an array of maps. These maps can have the following keys:

`command`
: The shell command to run.

`port`
: Delay execution until this TCP port is open.

`line`
: Delay execution until the process has emitted this line on standard output.

`duration`
: Delay execution until this much time has passed.

`file`
: Delay execution until this file exists on disk.


SEE ALSO
========

`specsheet(1)`
