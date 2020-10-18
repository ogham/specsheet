% specsheet_udp(5) v0.1.0


NAME
====

specsheet_udp — The ‘udp’ check for specsheet


EXAMPLES
========

Check that a local UDP port responds when a packet is sent to it:

```toml
[[udp]]
port = 53
```

Check that a port does _not_ respond:

```toml
[[udp]]
port = 53
state = 'no-response'
```

Check that a port on another machine responds:

```toml
[[udp]]
port = 53
address = '192.168.0.1'
```


PARAMETERS
==========

`address` (string)
: The address to send the request to.

`port` (string)
: The UDP port number.

`source` (string)
: The network address or interface to send from.

`state` (string)
: The state of the port. This can be `responds` or `no-response`.

`ufw` (table)
: UFW check options.


SEE ALSO
========

`specsheet(5)`
