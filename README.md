# Try-Let

[![Build Status](https://travis-ci.org/mystor/rust-try-let.svg?branch=master)](https://travis-ci.org/mystor/rust-try-let)

This is an implementation of a `try-let` similar to the one proposed in
[RFC #1303](https://github.com/rust-lang/rfcs/pull/1303), as a proc macro.

> _NOTE:_ Proc macros in statement position are currently unstable, meaning this
> macro will not work on stable rust until
> [PR #68717](https://github.com/rust-lang/rust/pull/68717) is merged.

## Usage

try-let is implemented using a proc macro instead, as parsing the pattern
expression in the way which try-let needs to is not possible with a
`macro_rules!` macro.

This plugin currently requires enabling `#![feature(proc_macro_hygiene)]`, like
so:

```rust
#![feature(proc_macro_hygiene)]
use try_let::try_let;
```

The actual use is fairly similar to a `let` expression:

```rust
try_let!(Some(x) = ... else return Err("Shoot! There was a problem!"));
```

The expression after else must diverge (e.g. via `return`, `continue`, `break`
or `panic!`).

If you care about the values of other alternatives, you can match against them
too:

```rust
// What do you know! It's `let x = try!(...)` implemented more verbosely!
try_let!(Ok(x) = ... {
    Err(e) => return e
});
```

This also handles more complex types than `Some` and `None`:

```rust
enum E {
    A(i32, i32, i32, i32, Option<i32>, Result<(), i32>),
    B
}

// ...

try_let!(A(a, 21, c, 34, Some(e), Err(f)) = ... else return);
// a, c, e, and f are all bound here.
```

## Why

This provides a simple way to avoid the rightward-shift of logic which performs
a large number of faillible pattern matches in rust. This allows the main logic
flow to continue without increasing the indent level, while handling errors with
diverging logic.

## How

a `try_let!()` invocation expands to the following:

```rust
try_let!(Some(x) = ... else return Err("Shoot! There was a problem!"));
// ... becomes ...
let (x,) = match ... {
    Some(x) => (x,),
    _ => return Err("Shoot! There was a problem!"),
};
```

### A note on `None` and empty enum variants

A question which some people will be asking now is how are enum variants like
`None` handled?

```rust
try_let!(None = ... else return);
// ... becomes ...
let () = match ... {
    None => (),
    _ => return,
}
```

`None` isn't mistaken for a binding variable by try-let because of the dirty
little trick which try-let uses to function: which is that it is powered by
rust's style conventions. There is no way for the parser (which is all that the
syntax extension has access to) to determine whether a lone identifier in a
pattern is an empty enum variant like `None` or a variable binding like `x`.
This is determined later in the compiler, too late for this extension to use
that information.

Instead, the extension checks the first character of the identifier. If it is an
ASCII capital, we assume it is a empty enum variant, and otherwise we assume it
is a variable binding.
