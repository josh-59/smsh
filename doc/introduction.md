`smsh` Official Documentation
=============================

Overview
--------

A shell is a program that interprets and executes user input.
To do that, they employ a main loop consisting of just a few steps:

1. Get user input
2. Interpret user input
3. Find and execute command(s)

Where shells differ is in step two, interpretation.
This shell carries out the following steps:

1. Expansion
2. Separation
3. Selection


Expansion
---------

Expansion is the replacement of text with some other text.
All shells do expansion; in `smsh`, all expansions take place
within braces, `{}`.
We can, for instance, declare a variable with `let`, then
replace it with the text it contains:

```
$ let arg = Hello World!
$ echo {arg}
Hello World!
```

Unadorned braces, like those above, denote variable expansion.
Other types of expansion are had by prepending some character to 
the expansion.
For example, `e{ }` denotes environment variable expansion:

```
$ echo e{PATH}
/usr/bin/:/usr/local/bin:/usr/local/sbin
```

Separation
----------

After expansion, expanded text is separated.  
By default, text is separated by whitespace, so that in

```
$ echo {arg}
Hello World!
```

the `echo` command receives two arguments, `hello` and `world`.
We can separate by some arbitrary string by appending `S="[string]"`, 
like so:

```
$ echo e{PATH}S=":"
/usr/bin /usr/local/bin /usr/local/sbin
```
