% specsheet_tcp(5) v0.1.0


NAME
====

specsheet_tcp — The ‘tcp’ check for specsheet


EXAMPLES
========

Check that a local TCP port is open:

```toml
[[tcp]]
port = 8301
```

Check that a local TCP port is _closed_:

```toml
[[tcp]]
port = 8301
state = 'closed'
```

Check that a port is open on another machine:

```toml
[[tcp]]
port = 8301
address = '192.168.0.1'
```


PARAMETERS
==========

`address` (string)
: The address to send the request to.

`port` (number)
: The TCP port number.

`source` (string)
: The network address or interface to send from.

`state` (string)
: The state of the port. This can be `open` or `closed`.

`ufw` (table)
: UFW check options.


SEE ALSO
========

`specsheet(5)`
