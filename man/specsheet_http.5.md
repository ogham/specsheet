% specsheet_http(5) v0.1.0


NAME
====

specsheet_http — The ‘http’ check for specsheet


EXAMPLES
========

Check that an HTTP server returns HTML:

```toml
[[http]]
url = 'https://rfcs.io/'
status = 200
content_type = 'HTML'
```

Check that an HTTP server supports gzip:

```toml
[[http]]
url = 'https://rfcs.io/'
status = 200
encoding = 'gzip'
```

Check that HTTP redirects to HTTPS:

```toml
[[http]]
url = 'http://rfcs.io/'
redirect_to = 'https://rfcs.io/'
```

Check that the HTTP `Server` header is hidden:

```toml
[[http]]
url = 'https://rfcs.io/'
server = 'nginx (version hidden)'
```

Send extra HTTP headers:

```toml
[[http]]
url = 'https://example.com/secret'

[[http.request.headers]]
Authorization = 'Basic d2hhdCBkaWQ6eW91IGV4cGVjdD8='
```


PARAMETERS
==========

`body` (content)
: The content that the request body should have.

`content_type` (string)
: The `Content-Type` header expected in the response.

`encoding` (string)
: The `Content-Encoding` header expected in the response. This also gets sent in the `Accept-Encoding` header of the request.

`headers` (table)
: Mapping of HTTP headers that should exist in the response.

`redirect_to` (string)
: The URL to redirect to, if the response has a redirect (3xx) HTTP status.

`status` (number)
: The HTTP status of the response.

`server` (string)
: The `Server` header expected in the response.

`url` (string)
: The URL that is being tested.


CONTENT-TYPE SHORTHANDS
=======================

Specsheet accepts shorthand versions of many common `Content-Type` values.
Here’s the full list:

`ATOM`
: `application/atom+xml`

`CSS`
: `text/css`

`EOT`
: `application/vnd.ms-fontobject`

`FLIF`
: `image/flif`

`GIF`
: `image/gif`

`HTML`
: `text/html`

`ICO`
: `image/x-icon`

`JPEG`
: `image/jpeg`

`JS`
: `text/javascript`

`JSON`
: `application/json`

`JSONFEED`
: `application/feed+json`

`OTF`
: `font/opentype`

`PDF`
: `application/pdf`

`PNG`
: `image/png`

`SVG`
: `image/svg+xml`

`TTF`
: `font/ttf`

`WEBP`
: `image/webp`

`WOFF`
: `application/font-woff`

`WOFF2`
: `font/woff2`

`XML`
: `text/xml`

`ZIP`
: `application/zip`



SEE ALSO
========

`specsheet(5)`
