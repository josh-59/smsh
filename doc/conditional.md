smsh Official Documentation
===========================

If statements
-------------

Conditional statements in `smsh` could not be simpler:

if [command]:
    [body]

Executes [command].  If [command] returns zero (true), execute [body].
The `smsh` language relies upon a plethora of builtins (compartmentalized
into modules) to offer users full-featured functionality.

Multi-branch statements are modeled after Python:

if [command]:
    [body]
elif [command]:
    [body]
else:
    [body]

The branch whose [command] returns true is executed
