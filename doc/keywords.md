smsh Official Documentation
===========================

Keywords
--------

###and
_command1_ and _command2_
Execute _command1_. If _command1_ returns zero (true), execute _command2_

###or
_command1_ _command2_
Execute _command1_. If _command1_ returns false (nonzero), execute _command2_


###not
not _command_
Execute _command_ and invert return value.
If _command_ returns true, then return false;
if _command_ returns false, then return true.

###let
let _var_ = _val1_ [val2 val3 ...]
Set a user-defined variable.

###fn
```
fn _my_func_:
    [body]
```
Set a user-defined function

if
elif
else
while
for

To escape a keyword, wrap it in quotes (single or double)
