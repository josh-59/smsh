smsh Official Documentation
===========================

Modular
-------

`smsh` is designed to be highly modular.
Modules can be loaded and unloaded dynamically, and 
contain builtins and shell parameters.

```
$ self::load_module mod
$ self::unload_module mod
```

Core Module
-----------

The essential builtins are found in the `core` module.
It is loaded at `smsh` initialization, and cannot be
unloaded.

###Core Builtins
    cd
    exit
    self::load_module
    slef::unlaod_module
