smsh Official Documentation
===========================

String Manipulation
-------------------

A string is a sequence of characters treated as a whole.
Strings are denoted with double-quotes, as in
`"Hello World"`. 
Strings undergo separation, wherein the string is split into
substrings, and selection, where some substrings are discarded.
This sequence is applied to the result of text replacement, as well.

By default, strings are separated by UTF-8 Whitespace.
By default, the entire sequence of substrings is selected.

Separation Modifiers:
    {}          [None] Split by word
    {}S=":"     Arbitrary Separators
    {}L         Split by line
    {}R         Raw (Do not split)

Selection
    {}[n]       Index
    {}[n..m]    Slice 
    {}[n..]     Slice
    {}[..n]     Slice

