smsh Official Documentation
===========================

Text Replacement
----------------

Text replacement serves the role of expansions in `smsh`. 
It is strictly explicit, and occurs only within braces, `{}`. 
Text replacements are broken down into types, with each type
denoted explicitly by a leading character.

Types:
    {}          User-Defined Variable Replacement
    e{}         Environment Variable Replacement
    !{}         Subshell Replacement

For example:

```
$ let myvar = 1234
$ echo {myvar}
1234
```

If no replacement text is found, the empty string is substituted.
After replacement, the resulting text is treated as a string,
and undergoes separation and selection.
