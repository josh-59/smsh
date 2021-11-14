smsh Official Documentation
===========================

Modular
-------

`smsh` is designed to be highly modular.
Modules contain builtins and shell parameters, and
can be loaded and unloaded at runtime:

```
$ self::load_module mod
$ self::unload_module mod
```

Core Module
-----------

The essential builtins are found in the `core` module.
It is loaded at `smsh` initialization, and cannot be
unloaded.

Core Builtins:
    cd
    exit
    self::load_module
    self::unlaod_module


File Module
-----------

Contains builtins for testing the properties of files.

exists 
is_empty
is_regular
is_directory
is_block_special 
is_character_special
is_symbolic_link 
is_fifo
is_socket
is_readable
is_writeable
is_executable
has_set_group_id_set
has_sticky_bit_set
