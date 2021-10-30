smsh Official Documentation
===========================

File Module
-----------

The File module contains builtins useful for scripting purposes.

File::exists _file_
    Returns true if _file_ exists, otherwise returns false.

File::is_empty _file_
    Returns true if _file_ is empty, otherwise returns false.

File::is_regular _file_
    Returns true if _file_ is a regular file, otherwise returns false.

File::is_directory _file_
    Returns true if _file_ is a directory, otherwise returns false.

File::is_block_special _file_
    Returns true if _file_ is a block special (device) file, otherwise returns false.

File::is_character_special _file_
    Returns true if _file_ is a character special (device) file, otherwise returns false.

File::is_symbolic_link _file_
    Returns true if _file_ is a symbolic link, otherwise returns false.

File::is_fifo
    Returns true if _file_ is a FIFO (named pipe), otherwise returns false.

File::is_socket
    Returns true if _file_ is a socket, otherwise returns false.

File::is_readable
File::is_writeable
File::is_executable

File::has_set_group_id_set
File::has_sticky_bit_set
