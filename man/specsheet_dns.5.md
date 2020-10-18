% specsheet_dns(5) v0.1.0


NAME
====

specsheet_dns — The ‘dns’ check for specsheet


EXAMPLES
========

Check for an `A` record value:

```toml
[[dns]]
domain = 'millimeter.io'
class = 'A'
value = '159.65.215.200'
```

Check for the _lack_ of a value:

```toml
[[dns]]
domain = 'secret.millimeter.io'
class = 'SRV'
state = 'missing'
```

Check using a specific DNS nameserver:

```toml
[[dns]]
domain = 'secret.millimeter.io'
class = 'SRV'
value = '192.168.34.43:8125'
nameserver = '127.0.0.53'
```


PARAMETERS
==========

`domain` (string)
: The domain to send a query about.

`nameserver` (string)
: Address of the DNS server to send requests to.

`state` (string)
: The state of the record. This can be `present` or `missing`.

`type` (string)
: The DNS record type (rtype) to query for.

`value` (string)
: The response IP address or value.


SEE ALSO
========

`specsheet(5)`
