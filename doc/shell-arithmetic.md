Some Documentation
==================

Shell Arithmetic
----------------

There is none; `smsh` is not a calculator.  Instead, we delegate that responsibility
to Python, and do our best to facilitate its use.  Note that you can pass *lines*
of code to Python with the `-c` option, so, for instance,

```
$ python -c "print(40 + 2)"
42
```

And,

```
$ let var = Hello World
$ python -c print(\"{var}\")
Hello World
```

Parenthesis have no special meaning for `smsh`, so no need to escape them.  
Note that variables expanded will be read by Python as a String, and so require
conversion to numerical values to do math on them:

```
$ let var = 123
$ python -c "
    var = int({var})
    print(var + 123)
"
246
```

Like I said, we do our best to facilitate its use.  One thing we do is redact the
number of leading tabs present in the first line from it and all following lines.  Hence, the above script is identical to,

```
$ let var = 123
$ python -c "
var = int({var})
print(var + 123)"
246
```

That's just to make things more readable: A block of Python code does not interupt the
indentation of the script it's within.  
