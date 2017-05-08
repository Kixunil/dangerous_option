Dangerous Option
================

About
-----

This crate provides DangerousOption - a type similar to `!` in Swift language. It's basically
an `Option` which panics if dereferenced while containing `None`. This is useful in cases one
needs to initialize things a little bit later or when accesses are made via functions called
from trusted library.

While such thing might look like a step back (there's a reason we don't have NULL pointers in
Rust), there is still one advantage over classic approach of NULL-pointer exceptions (including
manually unwrapping): the cause of the bug is usually not in the place where dereferencing
happened but in the place where assignment happened. Since this type has only three, very
unique methods for creating invalid value, those can be easily searched for and tracked.

This crate is `no_std`.

License
-------

MITNFA
