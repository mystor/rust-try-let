#![feature(plugin_registrar, rustc_private, vec_push_all)]
// XXX: vec_push_all renamed to extend_from_slice soon

#[macro_use]
extern crate syntax;

#[macro_use]
extern crate rustc;

#[macro_use]
extern crate rustc_front;

#[macro_use]
extern crate rustc_plugin;

use rustc_plugin::Registry;
use syntax::parse::token::intern;
use syntax::codemap::{Span, spanned};
use syntax::ast::*;
use syntax::ext::base::{MacResult, ExtCtxt, DummyResult, MacEager};
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::parse::parser::{Parser, Restrictions};
use syntax::parse::PResult;
use syntax::ptr::*;
use syntax::util::small_vector::SmallVector;

use syntax::ext::base::SyntaxExtension;

use std::ascii::AsciiExt;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(intern("try_let"),
                                  SyntaxExtension::NormalTT(Box::new(expand_try_let),
                                                            None,
                                                            false));
}

fn get_bind_names(pat: &Pat) -> Vec<(SpannedIdent, Mutability)> {
    match pat.node {
        PatIdent(BindByRef(mutability), ref si, ref op) |
        PatIdent(BindByValue(mutability), ref si, ref op) => {
            let mut res = if let Some(ref p) = *op {
                get_bind_names(p)
            } else {
                vec![]
            };

            let name = si.node.name.as_str();
            if let Some(c) = name.chars().next() {
                // Check if our first character is ascii lowercase,
                // or a non-ascii character.
                if c.to_ascii_lowercase() == c {
                    res.push((*si, mutability));
                }
            }

            res
        }
        PatEnum(_, Some(ref v)) | PatTup(ref v) => {
            let mut res = Vec::new();
            for it in v {
                res.push_all(&get_bind_names(it));
            }
            res
        }
        PatStruct(_, ref v, _) => {
            let mut res = Vec::new();
            for it in v {
                res.push_all(&get_bind_names(&*it.node.pat));
            }
            res
        }
        PatBox(ref p) | PatRegion(ref p, _) => get_bind_names(p),
        PatVec(ref v1, ref op, ref v2) => {
            let mut res = Vec::new();
            for it in v1 {
                res.push_all(&get_bind_names(it));
            }
            if let Some(ref p) = *op {
                res.push_all(&get_bind_names(p));
            }
            for it in v2 {
                res.push_all(&get_bind_names(it));
            }
            res
        }
        _ => vec![],
    }
}

fn parse_try_let(mac_span: Span,
                 parser: &mut Parser) -> PResult<SmallVector<P<Stmt>>> {
    let pat = try!(parser.parse_pat());
    let pat_span = pat.span;
    try!(parser.expect(&token::Eq));
    let expr = try!(parser.parse_expr_res(
        Restrictions::RESTRICTION_NO_STRUCT_LITERAL, None));

    let names = get_bind_names(&*pat);

    // Create a list of path expressions, and form it into a tuple for
    // the body of the first branch.
    let names_exprs = names.iter().map(|name| {
        parser.mk_expr(name.0.span.lo, name.0.span.hi,
                       ExprPath(None, Path {
                           span: name.0.span,
                           global: false,
                           segments: vec![PathSegment {
                               identifier: name.0.node,
                               parameters: PathParameters::none()
                           }]
                       }), None)
    }).collect();
    let default_arm = parser.mk_expr(pat.span.lo, pat.span.hi,
                                     ExprTup(names_exprs), None);

    // Create the first arm of the match statement
    let mut arms = vec![Arm {
        attrs: Vec::new(),
        pats: vec![pat],
        guard: None,
        body: default_arm
    }];

    // Parse the rest of the body, and use it to create the remaining arms
    // of the match statement
    if parser.check(&token::OpenDelim(token::Brace)) {
        // { MATCH STATEMENTS }
        try!(parser.expect(&token::OpenDelim(token::Brace)));
        while !parser.check(&token::CloseDelim(token::Brace)) {
            arms.push(try!(parser.parse_arm()));
        }
        try!(parser.expect(&token::CloseDelim(token::Brace)));
    } else {
        // else EXPR
        try!(parser.expect_keyword(token::keywords::Else));
        let e = try!(parser.parse_expr());
        let pat = PatWild;
        arms.push(Arm {
            attrs: Vec::new(),
            pats: vec![P(Pat {
                id: DUMMY_NODE_ID,
                node: pat,
                span: e.span,
            })],
            guard: None,
            body: e,
        });
    }
    let match_expr = parser.mk_expr(mac_span.lo, mac_span.hi,
                                    ExprMatch(expr, arms), None);

    // Create the resulting pattern to bind against
    // let let_stmt = parser.mk_stmt(mac_span.lo, mac_span)

    let names_pats = names.iter().map(|name| {
        P(Pat{
            id: DUMMY_NODE_ID,
            node: PatIdent(BindByValue(name.1), name.0, None),
            span: name.0.span,
        })
    }).collect();
    let names_pat = P(Pat {
        id: DUMMY_NODE_ID,
        node: PatTup(names_pats),
        span: pat_span
    });
    let stmt = P(spanned(mac_span.lo, mac_span.hi, StmtDecl(P(spanned(
        mac_span.lo, mac_span.hi, DeclLocal(P(Local {
            ty: None,
            pat: names_pat,
            init: Some(match_expr),
            id: DUMMY_NODE_ID,
            span: mac_span,
            attrs: None,
        })))), DUMMY_NODE_ID)));

    Ok(SmallVector::one(stmt))
}

fn expand_try_let<'a>(ec: &'a mut ExtCtxt,
                      mac_span: Span,
                      tts: &[TokenTree])
                      -> Box<MacResult + 'a> {
    let mut parser = ec.new_parser_from_tts(tts);

    match parse_try_let(mac_span, &mut parser) {
        Ok(e) => MacEager::stmts(e),
        Err(_) => DummyResult::expr(mac_span),
    }
}
