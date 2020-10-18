% specsheet_ufw(5) v0.1.0


NAME
====

specsheet_ufw — The ‘ufw’ check for specsheet


EXAMPLES
========

Check that the firewall allows outbound HTTPS:

```toml
[[ufw]]
port = 443
protocol = 'tcp'
allow = 'Anywhere'
```


PARAMETERS
==========

`allow` (string)
: Whether this rule is for allowing or denying.

`ipv6` (boolean)
: Whether to check for IPv6.

`port` (number or string)
: The port, or range of ports, to check.

`protocol` (string)
: The protocol of this rule. This can be `tcp` or `udp`.

`state` (string)
: The state of this rule. This can be `present` or `missing`.


SEE ALSO
========

`specsheet(5)`
