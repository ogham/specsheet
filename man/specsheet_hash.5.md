% specsheet_hash(5) v0.1.0


NAME
====

specsheet_hash — The ‘hash’ check for specsheet


EXAMPLES
========

Check that a file has a specific MD5 hash:

```toml
[[hash]]
path = '/usr/bin/specsheet'
algorithm = 'md5'
hash = '3f22baaf4ba820a800dfc51af5ba1892'
```


PARAMETERS
==========

`algorithm` (string)
: The hashing algorithm to use.

`hash` (string)
: The expected hash, as a hexadecimal string.

`path` (string)
: The path to a local file on disk.


LIST OF ALGORITHMS
==================

- `md5`
- `sha1`
- `sha224`
- `sha256`
- `sha384`
- `sha512`


SEE ALSO
========

`specsheet(5)`
