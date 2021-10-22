# smsh
Simple Shell: A shell for the next generation


`smsh` will be a full-featured, resonably fast, and easy to use shell.
It will respect the Unix philosophy, and offer a readable, even Pythonic, scripting language.
What Rust is to systems programming, `smsh` is intended to be to shells.

`smsh` will work on any Unix-like operating system, but is otherwise
not cross-platform.
Its codebase strives for correctness and simplicity, at the expense of speed.
It is intended to be the single friendliest interactive shell in existence.

As a shell that is easy to understand, 
there are three major concepts to be understood:
- Pipelines
- Expansions
- Conditionals

## Pipelines
Each line given to `smsh` is interpretted as a pipeline.
The last element in a pipeline is executed in the context of the current shell.

## Expansions
Expansions are _Strictly Explicit,_ and occur within braces, `{}`.
From there, expansion within `smsh` forms a mini-language.
`smsh` does not support aliasing.


