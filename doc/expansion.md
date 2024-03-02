smsh Official Introduction
==========================

Expansion
---------

Expansion in `smsh` is is strictly explicit; it occurs only within braces, `{}`. 
We support several types of expansion, with each type explicitly denoted by a 
leading character.

Expansion Types:
    {}      Variable Expansion
    ${}     Subshell Expansion
    f{}     Filename Expansion
    t{}     Terminal Expansion

### Argument Expansion

For example:

```
$ let myvar = 1234
$ echo {myvar}
1234
```

If no replacement text is found, the empty string is substituted.
After replacement, unless the expansion is double-quoted (`"`), 
the expanded text undergoes separation.

Some variable names are special, and cannot be assigned to by `let`.

`{0}`
    Expands to the script name or function name or `smsh`, depending on
    context

`{1}`
    Expands to the first argument given.

`{2}`
    Expands to the second argument given...

`{1..}`
    Expands to *all* arguments given to the script or function.

`{rv}`
    Expands to the return value of the previously executed command.

[Possibly]
`{options}`
    Expands to all arguments beginning with `--` (e.g., all options).

`{1..-options}`
    Expands to all argument that are *not* options.


### Subshell Expansion
User-defined variable expansion is pretty self-explanatory, so we'll move right
along to subshell expansion.  Subshell expansion does exactly what you'd expect
it to do-- it launches a subshell, executes the commands and captures stdout;
said captured output then replaces the subshell expansion expression.


### Filename Expansion

We like regex!  So regex it is.  ...with one small modification.  

- Expression is split by forward slash `/`.
    - Expressions beginning with slash are treated as absolute
    - Expressions not beginning with slash are treated as relative
- Each split is then treated as a regex expression
- Expands to alphebetized list

Examples:
`f{/usr/bin/.*}`
    Expands to all files in `/usr/bin`.

`f{Downloads/.*}`
    Expands to all files in `./Downloads`.

`f{.*}`
    Expands to all files in current working directory.


### Terminal Expansion
To make prompt customization simpler, we include a special expansion type

`t{bell}`
    Expands to an ASCII bell character

`t{date}`
    Expands to the date in "Weekday Month Date" format (e.g., "Tue May 26")
    
