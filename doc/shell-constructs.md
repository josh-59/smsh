smsh Official Documentation
===========================

Shell Constructs
----------------

`if`, `for`, and `while` are each considered shell constructs.


###If statements

Conditional statements in `smsh` could not be simpler:

```
if [command]:
    [body]
```

Executes [command].  If [command] returns zero (true), execute [body].
Multi-branch statements follow in the spirit of Python: 

```
if [command]:
    [body]
elif [command]:
    [body]
else:
    [body]
```

The first branch whose [command] returns true is executed; all others
are ignored.

###For loops

For loops iterate over values:

```
$ for val in one two three four:
>     echo {val}    
one
two
three
four
```

It should be noted that each conditional line passes through
expansion, separation and selection before being executed.