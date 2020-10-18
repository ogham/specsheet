% specsheet_ping(5) v0.1.0


NAME
====

specsheet_ping — The ‘ping’ check for specsheet


EXAMPLES
========

Check that a server responds to ping:

```toml
[[ping]]
target = '192.168.0.1'
```


PARAMETERS
==========

`state` (string)
: The state of the response. This can be `responds` or `no-response`.

`state` (string)
: The hostname or IP address to ping.


SEE ALSO
========

`specsheet(5)`
