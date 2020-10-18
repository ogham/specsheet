% specsheet_systemd(5) v0.1.0


NAME
====

specsheet_systemd — The ‘systemd’ check for specsheet


EXAMPLES
========

Check that a service is running:

```toml
[[systemd]]
service = 'sshd'
```

Check that a service is stopped:

```toml
[[systemd]]
service = 'httpd'
state = 'stopped'
```


PARAMETERS
==========

`systemd` (string)
: Name of the systemd service.

`state` (string)
: State of the systemd service. This can be `running`, `stopped`, or `missing`.


SEE ALSO
========

`specsheet(5)`
