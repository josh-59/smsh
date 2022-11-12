# Simple Shell

## Embrace the Stupid

`smsh` is a shell meant to be understood.

## Design Goals

__Simple__  is good. Simple means easy to understand, easy to learn and easy
to master.  Simple also means clean, and `smsh` takes after Python et al. to offer 
a clean, modern scripting language.

__Fully Functional__ means that `smsh` will do anything you need it to do.
It is intended to be capable as a shell for both interactive and 
non-interactive use-cases.

__Modern Interactive Experience__ Wrapping a simple and capable core is
an interactive layer that takes the best-of from current shells and
the Rust ecosystem.  

## Features

__Strictly Explicit Expansion.__  No aliasing here!  All expansion occurs
within braces `{}`, with expansion types being denoted by leading
characters.  For example, `e{}` denotes environment variable expansion:

```
$ echo e{PATH}
/usr/local/sbin:/usr/local/bin:/usr/bin
```

__Verbose Error Reporting.__ No more "Syntax error near unexpected token"!
The Rust compiler does CLI error reporting right, and we want to be like Rust:

```
$ echo e{PATH
smsh: echo e{PATH
            ^ Unmatched expansion brace
```

__Modular.__ `smsh` respects the Unix Philosophy by 
executing external commands wherever possible:  

```
$ if test -e direction:
    echo `smsh` has direction!
`smsh` has direction!
```


## Project Status

Entering 'Alpha' Status.  Major features are present and working, 
but we're still getting our wheels under us.


## Contributing

Contributions are always welcome!
Ideas and suggestions are welcome as-well.
