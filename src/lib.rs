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
        let (#(#bindings,)*) = if let #pat = (#expr) {
            (#(#bindings,)*)
        } else {
            #fallback
        };
    );
    println!("output = {}", output);
    return output.into();
}
