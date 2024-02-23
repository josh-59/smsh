smsh Official Documentation
===========================

Expansion
---------

Expansion in `smsh` is is strictly explicit, occuring only within braces, `{}`. 
There are several types of expansion, with each type
explicitly denoted by a leading character.

Expansion Types:
    {}          User-Defined Variable Expansion
    !{}         Subshell Expansion
    f{}         Filename Expansion

For example:

```
$ let myvar = 1234
$ echo {myvar}
1234
```

If no replacement text is found, the empty string is substituted.
After replacement, unless the expansion is double-quoted (`"`), 
the resulting text undergoes separation, then selection.


### Filename Expansion

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

`f{}`
    Expands to all files in current working directory.
    Same as `f{.*}`.
