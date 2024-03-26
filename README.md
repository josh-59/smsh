# Simple Shell

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
within braces `{}`, with different expansion types being denoted by leading
characters.  For example, `${}` denotes subshell expansion:

__Verbose Error Reporting.__ No more "Syntax error near unexpected token"!
The Rust compiler does CLI error reporting right, and we want to be like Rust:

```
$ echo {PATH
smsh: echo {PATH
            ^ Unmatched expansion brace
```

__Modular.__ `smsh` can accomodate new and different modules.  For instance, the `file` module contains builtins for testing files, so that we write something like,

```
$ load-mod files
$ if file-exists foo:
>   # Do stuff
```

## Project Status

Mostly abandoned, for lack of skill.  Major features are present and working, though.  

## Contributing

Contributions are welcome!  Ideas and suggestions are welcome as-well.
