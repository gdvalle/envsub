# envsub
`envsub` is a program for substituting env vars on a stdin->stdout pipe.
Much like [envsubst][envsubst], but slightly different behavior.

It supports a configurable prefix and suffix for matching env vars, and
defaults to percent-enclosed keys, i.e. `%VAR%`.  When hitting unset
variables it will exit rather than expanding as empty strings.  It also
fully buffers input before writing, so in-place replacement is possible.

[![Build Status](https://github.com/gdvalle/envsub/workflows/CI/badge.svg)](https://github.com/gdvalle/envsub/actions?query=workflow%3ACI)

## Why
I found the combination of unset variable expansion and unconfigurable
variable expression in `envsubst` prohibitive.  Sometimes files already use
`$VAR` style variables (ex. nginx) and it's simply more readable to have
a different prefix/suffix.

Chaining `sed` expressions with `-e` is another, more flexible method. `envsub` just has a
little less boilerplate to operate.


## Usage
Given a file, `server.conf.tmpl`:
```
server {
    server_name %SERVER_NAME%;
    listen %SERVER_PORT%;
}
```
We can set environment variables to populate the config:
```
export SERVER_NAME=www.example.com
export SERVER_PORT=80
envsub < server.conf.tmpl > server.conf
```
And `server.conf` is updated:
```
server {
    server_name www.example.com;
    listen 80;
}
```

We can also restrict which environment vars are evaluated by supplying CLI
arguments:
```
$ envsub SERVER_NAME < server.conf
server {
    server_name www.example.com;
    listen %SERVER_PORT%;
}
```

## Configuration
To configure the match prefix or suffix, env vars are available:

* `ENVSUB_PREFIX=%`
* `ENVSUB_SUFFIX=%`

Arguments are used to restrict env var evaluation:
```
[[ "$(FOO=x BAR=y envsub BAR <<<"%FOO%%BAR%")" = "%FOO%y" ]]
```

## Behavior
Since this is not a direct copy of `envsubst` some other behavior may be
different.

`envsub` also fails hard on:
* Var matches that are unset,
* Anything not UTF-8? (should test this)


## Credits
This is built on the shoulders of [aha-corosick][aha-corosick].


[envsubst]: https://www.gnu.org/software/gettext/manual/html_node/envsubst-Invocation.html
[aha-corosick]: https://github.com/BurntSushi/aho-corasick
