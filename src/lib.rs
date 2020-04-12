//! This is an implementation of a `try-let` similar to the one proposed in
//! [RFC #1303](https://github.com/rust-lang/rfcs/pull/1303), as a proc macro.
//!
//! > _NOTE:_ Proc macros in statement position are currently unstable, meaning this
//! > macro will not work on stable rust until
//! > [PR #68717](https://github.com/rust-lang/rust/pull/68717) is merged.
//!
//! # Usage
//!
//! try-let is implemented using a proc macro instead, as parsing the pattern
//! expression in the way which try-let needs to is not possible with a
//! `macro_rules!` macro.
//!
//! This plugin currently requires enabling `#![feature(proc_macro_hygiene)]`, like
//! so:
//!
//! ```rust
//! #![feature(proc_macro_hygiene)]
//! use try_let::try_let;
//! ```
//!
//! The actual use is fairly similar to a `let` expression:
//!
//! ```rust
//! # #![feature(proc_macro_hygiene)]
//! # use try_let::try_let;
//! # fn main() -> Result<(), &'static str> {
//! # let foo = Some(());
//! try_let!(Some(x) = foo else return Err("Shoot! There was a problem!"));
//! # Ok(())
//! # }
//! ```
//!
//! The expression after else must diverge (e.g. via `return`, `continue`, `break`
//! or `panic!`).
//!
//! This also handles more complex types than `Some` and `None`:
//!
//! ```rust
//! # #![feature(proc_macro_hygiene)]
//! # use try_let::try_let;
//! # fn main() {
//! enum E {
//!     A(i32, i32, i32, i32, Option<i32>, Result<(), i32>),
//!     B,
//! }
//!
//! let foo = E::A(0, 21, 10, 34, Some(5), Err(32));
//!
//! try_let!(E::A(a, 21, c, 34, Some(e), Err(f)) = foo else panic!());
//! // a, c, e, and f are all bound here.
//! assert_eq!(a, 0);
//! assert_eq!(c, 10);
//! assert_eq!(e, 5);
//! assert_eq!(f, 32);
//! # }
//! ```
//!
//! # Why
//!
//! This provides a simple way to avoid the rightward-shift of logic which performs
//! a large number of faillible pattern matches in rust. This allows the main logic
//! flow to continue without increasing the indent level, while handling errors with
//! diverging logic.
//!
//! # How
//!
//! a `try_let!()` invocation expands to the following:
//!
//! ```rust
//! # #![feature(proc_macro_hygiene)]
//! # use try_let::try_let;
//! # fn main() -> Result<(), &'static str> {
//! # let foo = Some(10);
//! try_let!(Some(x) = foo else return Err("Shoot! There was a problem!"));
//! // ... becomes ...
//! let (x,) = match foo {
//!     Some(x) => (x,),
//!     _ => {
//!         return Err("Shoot! There was a problem!");
//!     }
//! };
//! # Ok(())
//! # }
//! ```
//!
//! ## A note on `None` and empty enum variants
//!
//! A question which some people will be asking now is how are enum variants like
//! `None` handled?
//!
//! ```rust
//! # #![feature(proc_macro_hygiene)]
//! # use try_let::try_let;
//! # fn main() {
//! # let foo = Some(10);
//! try_let!(None = foo else return);
//! // ... becomes ...
//! let () = match foo {
//!     None => (),
//!     _ => {
//!         return;
//!     }
//! };
//! # }
//! ```
//!
//! `None` isn't mistaken for a binding variable by try-let because of the dirty
//! little trick which try-let uses to function: which is that it is powered by
//! rust's style conventions. There is no way for the parser (which is all that the
//! syntax extension has access to) to determine whether a lone identifier in a
//! pattern is an empty enum variant like `None` or a variable binding like `x`.
//! This is determined later in the compiler, too late for this extension to use
//! that information.
//!
//! Instead, the extension checks the first character of the identifier. If it is an
//! ASCII capital, we assume it is a empty enum variant, and otherwise we assume it
//! is a variable binding.

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::visit::{self, Visit};
use syn::{parse_macro_input, Expr, Ident, Pat, PatIdent, Result, Token};

struct TryLet {
    pat: Pat,
    _eq_token: Token![=],
    expr: Expr,
    _else_token: Token![else],
    fallback: Expr,
}

impl Parse for TryLet {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(TryLet {
            pat: input.parse()?,
            _eq_token: input.parse()?,
            expr: input.parse()?,
            _else_token: input.parse()?,
            fallback: input.parse()?,
        })
    }
}

struct Visitor<'a> {
    bindings: &'a mut Vec<Ident>,
}

impl<'a> Visit<'_> for Visitor<'a> {
    fn visit_pat_ident(&mut self, id: &PatIdent) {
        // If the identifier starts with an uppercase letter, assume that it's a
        // constant, unit struct, or a unit enum variant. This isn't a very
        // accurate system for this type of check, but is unfortunately the best
        // we can do from within a proc macro.
        if id
            .ident
            .to_string()
            .chars()
            .next()
            .unwrap()
            .is_ascii_uppercase()
        {
            return;
        }
        self.bindings.push(id.ident.clone());

        // Visit any nested expressions (e.g. using `id @ pat`)
        visit::visit_pat_ident(self, id);
    }
}

/// The whole point
///
/// See the module-level documentation for details.
#[proc_macro]
pub fn try_let(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let TryLet {
        pat,
        expr,
        fallback,
        ..
    } = parse_macro_input!(input as TryLet);

    let mut bindings = Vec::new();
    Visitor {
        bindings: &mut bindings,
    }
    .visit_pat(&pat);

    // NOTE: This doesn't use `mixed_site`, however it also doesn't introduce
    // any new identifiers not from the call-site.
    let output = quote!(
        let (#(#bindings,)*) = match (#expr) {
            #pat => (#(#bindings,)*),
            _ => {
                #fallback;
            }
        };
    );
    return output.into();
}
