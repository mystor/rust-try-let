# Try-Let

This is an implementation of a `try-let` similar to the one proposed in
[RFC #1303](https://github.com/rust-lang/rfcs/pull/1303), as a syntax 
extension. 

## Usage

try-let is implemented using a syntax extension instead of a macro, as
parsing the pattern expression in the way which try-let needs to is no
possible with a `macro_rules!` macro.

To use the plugin, add `#[plugin(try_let)]` to the top of the project, like so:
```rust
#![feature(plugin)]
#![plugin(try_let)]
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

```rust
try_let!(Ok(x) = ... {
    Err(e) => return e
});
// ... becomes ...
let (x,) = match ... {
    Ok(x) => (x,),
    Err(e) => return e,
};
```

### A note on `None` and empty enum variants

A question which some people will be asking now is how are enum variants like
`None` handled?

```
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

