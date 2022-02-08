use crate::common::TS;
use quote::quote;
use std::ops::Index;
use syn::spanned::Spanned;
use syn::{Expr, Member, Result};

pub fn connect_gen(item: &syn::ItemFn) -> Result<TS> {
    let body = connect_block(&item.block)?;
    Ok(quote! {
        fn connect(&mut self) {
            #body
        }
    })
}

pub fn connect_block(block: &syn::Block) -> Result<TS> {
    let mut stmt = vec![];
    for x in &block.stmts {
        stmt.push(connect_statement(x)?);
    }
    Ok(quote! {#(#stmt);*;})
}

fn connect_statement(statement: &syn::Stmt) -> Result<TS> {
    match statement {
        syn::Stmt::Expr(e) => connect_inner_statement(e),
        syn::Stmt::Semi(e, _) => connect_inner_statement(e),
        _ => Err(syn::Error::new(
            statement.span(),
            "Local definitions and items are not allowed in HDL kernels",
        )),
    }
}

fn connect_inner_statement(expr: &syn::Expr) -> Result<TS> {
    match expr {
        Expr::Assign(x) => connect_assignment(x),
        Expr::If(x) => connect_conditional(x),
        Expr::Match(x) => connect_match(x),
        Expr::ForLoop(x) => connect_for_loop(x),
        Expr::MethodCall(x) => connect_method_call(x),
        Expr::Call(x) => connect_call(x),
        _ => Ok(TS::new()),
    }
}

fn connect_for_loop(node: &syn::ExprForLoop) -> Result<TS> {
    let body = connect_block(&node.body)?;
    let ndx = &node.pat;
    let range = &node.expr;
    Ok(quote!(for #ndx in #range {
        #body
    }))
}

fn connect_call(node: &syn::ExprCall) -> Result<TS> {
    let source = node.args.index(0);
    let target = node.args.index(1);
    Ok(quote!(
        logic::logic_connect_join_fn(#source, #target);
    ))
}

fn connect_method_call(node: &syn::ExprMethodCall) -> Result<TS> {
    let source = &node.receiver;
    let method_name = node.method.to_string();
    if method_name == "link" {
        let target = node.args.index(0);
        if let Expr::Reference(t) = target {
            let target = &t.expr;
            return Ok(quote!(
                logic::logic_connect_link_fn(&mut #source, &mut #target);
            ));
        }
    }
    Ok(TS::new())
}

fn connect_assignment(node: &syn::ExprAssign) -> Result<TS> {
    if let Expr::Field(field) = node.left.as_ref() {
        if let Member::Named(nxt) = &field.member {
            if nxt.eq("next") {
                let lhs = &field.base;
                return Ok(quote!(
                    logic::logic_connect_fn(&mut #lhs)
                ));
            }
        }
    }
    Ok(TS::new())
}

fn connect_conditional(conditions: &syn::ExprIf) -> Result<TS> {
    let br1 = connect_block(&conditions.then_branch)?;
    let mut br2 = TS::new();
    if let Some((_, e_branch)) = &conditions.else_branch {
        match e_branch.as_ref() {
            Expr::Block(block) => {
                br2 = connect_block(&block.block)?;
            }
            Expr::If(cond) => {
                br2 = connect_conditional(cond)?;
            }
            _ => {
                return Err(syn::Error::new(
                    conditions.span(),
                    "Unsupported if/else structure",
                ))
            }
        }
    }
    Ok(quote!(#br1 #br2))
}

fn connect_match(m: &syn::ExprMatch) -> Result<TS> {
    let mut branches = vec![];
    for arm in &m.arms {
        branches.push(connect_body(&arm.body)?);
    }
    Ok(quote! {#(#branches);*;})
}

fn connect_body(body: &syn::Expr) -> Result<TS> {
    if let Expr::Block(b) = body {
        connect_block(&b.block)
    } else {
        connect_inner_statement(body)
    }
}
