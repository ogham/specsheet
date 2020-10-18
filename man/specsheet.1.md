% specsheet(1) v0.1.0


NAME
====

specsheet — the testing toolkit


SYNOPSIS
========

`specsheet [options...] [<file>...]`

Specsheet is a command-line utility that abstracts over the general testing pattern: running commands, examining their output, and collecting the results.

It can be used to test software, your computing environment, your cloud servers, and other things.


EXAMPLES
========

`specsheet checks.toml`
: Runs a check document.


META OPTIONS
============

`--help`
: Displays an overview of the command-line options.

`--version`
: Displays the version of specsheet being invoked.


DESCRIPTION
===========

Specsheet runs TOML checks.


ENVIRONMENT VARIABLES
=====================

`NO_COLOR`
: Disables ANSI colour output when listing the table of codes, as a fallback to the `--color` and `--colour` command-line options.

`SPECSHEET_DEBUG`
: Enables debug logging to standard error.


EXIT STATUSES
=============

0
: If everything goes OK, and all checks pass.

1
: If at least one check fails.

2
: If there was a syntax error in one of the check documents, or there was an I/O error reading one of the input files.

3
: If there was a problem with the command-line arguments.
