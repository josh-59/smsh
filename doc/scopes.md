smsh Official Documentation
===========================

Scopes
------

A scope consists of variable- and function-definitions.
When invoked, `smsh` creates a single scope, referred to as the
root scope.
This corresponds to the terminal when `smsh` is invoked interactively,
and is a script's scope otherwise.
A new scope is created whenever a function is invoked. 

When searching for a variable or function, `smsh` searches 
available scopes in a top-down manner, so that
the root scope is always searched last.
