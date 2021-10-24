smsh Official Documentation
===========================

Introduction
------------

Simple shell is a shell for the next generation.
It is intended to be full-featured, resonably fast, and most of all, easy to use.

`smsh` works on any Unix-like operating system (and it is a bug if this is not the case).
Its codebase strives for correctness and simplicity, at the expense of speed.
It is intended to be the single friendliest interactive shell in existence.


As a shell that is easy to understand, 
there are three major concepts to be understood:
- Pipelines
- Expansions
- Conditionals

## Pipelines
Each line given to `smsh` is interpretted as a pipeline.
The last element in a pipeline is executed in the context of the current shell
(if it is a builtin or user-defined function).

## Expansions
Expansions are _Strictly Explicit,_ and occur within braces, `{}`.
From there, expansion within `smsh` forms a mini-language.
`smsh` does not support aliasing.

## Conditionals
Like expansions, conditional expressions form a mini-language within `smsh`.

## Modularity
The core of `smsh` is modular: Builtins and shell variables belong to 
modules, and can be loaded and unloaded dynamically.
This will allow easy and arbitrary expansion of capabilities as the
shell matures.

## Contributing
Anyone is welcome to contribute to the project! Ideas and suggestions are welcome as well.


## Feature List
### Basic Shell Stuff
[ ] Pipelining
[ ] Redirection

### Expansion
[ ] Recursive Expansions
[ ] Subshell Expansion
[ ] Variable Expansion
[ ] Environment Variable Expansion
[ ] Expansion Modifiers

### Scripting Stuff
[ ] File Source
[ ] Blocks and indentations
[ ] User-defined Functions
[ ] If Expressions
[ ] While Expressions
[ ] Conditional Mini-language
[ ] Interpreter Scripts

### Interactive Stuff
[ ] Reedline usage
[ ] User-defined (left) prompt
[ ] User-defined (right) prompt
[ ] Completion
