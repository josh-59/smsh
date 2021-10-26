smsh Official Documentation
===========================

User Functions
--------------

User functions are defined through the `fn` builtin:

```
$ fn hello_world:
>     echo hello world
```

Blocks are denoted with four leading spaces.
By default, functions are visible to only the defining scope.
This can be overridden with the `--global` switch:

```
$ fn --global hello_world:
>     echo hello world
```

This function now exists at the root scope.

Functions may be dropped with the --drop switch:

``` 
$ fn --drop hello_world
```

Or

```
$ fn --drop --global hello_world
```
