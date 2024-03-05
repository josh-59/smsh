Some Documentation
==================

Expansion
---------

Multiplication
--------------

Addition
--------

The first conjoins two tokens together and removes whitespace,
so instead of writing, `abc{one}` one would write `abc + {one}`, and

Selection
---------

as for multiplication, `file/ * "one two three"` would result in 
`file/one file/two file/three`

Strings undergo separation, wherein the string is split into
substrings, and selection, where some substrings are discarded.
This sequence is applied to the result of text replacement, as well.

Selection
    {}[n]       Index
    {}[n..m]    Slice 
    {}[n..]     Slice 
    {}[..n]     Slice
