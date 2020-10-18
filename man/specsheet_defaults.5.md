% specsheet_defaults(5) v0.1.0


NAME
====

specsheet_defaults — The ‘defaults’ check for specsheet


EXAMPLES
========

Check that a value is set:

```toml
[[defaults]]
domain   = 'Apple Global Domain'
key      = 'AppleAquaColorVariant'
value    = 6
```

Check that a value is _not_ set:

```toml
[[defaults]]
domain   = 'Apple Global Domain'
key      = 'TireCount'
state    = 'missing'
```

Check that a value exists in a file within a container:

```toml
[[defaults]]
file   = '~/Library/Containers/com.apple.Safari/Data/Library/Preferences/com.apple.Safari'
key    = 'ShowIconsInTabs'
value  = '1'
```


PARAMETERS
==========

`domain` (string)
: The defaults domain.

`file` (string)
: Path to the defaults file on disk.

`key` (string)
: The defaults key.

`state` (string)
: Whether the key should be present. This can be `present` or `missing`.

`value` (string, array, or boolean)
: The value to expect.


SEE ALSO
========

`specsheet(5)`
