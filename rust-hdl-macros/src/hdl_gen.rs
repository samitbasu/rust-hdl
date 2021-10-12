use std::ops::Index;

use quote::format_ident;
use quote::quote;
use syn::spanned::Spanned;
use syn::{BinOp, Expr, Pat, Result, Stmt, UnOp};

use crate::common;
use crate::common::TS;

pub(crate) fn hdl_gen_process(item: syn::ItemFn) -> Result<TS> {
    let signature = &item.sig;
    if signature.inputs.len() != 1 {
        return Err(syn::Error::new(
            signature.span(),
            "HDL functions must contain a single argument (&mut self)",
        ));
    }
    let body = hdl_block(&item.block)?;
    Ok(quote! {
    fn hdl(&self) -> rust_hdl_core::ast::Verilog {
       rust_hdl_core::ast::Verilog::Combinatorial(#body)
    }
    })
}

fn hdl_block(block: &syn::Block) -> Result<TS> {
    let mut stmt = vec![];
    for statement in &block.stmts {
        stmt.push(hdl_statement(statement)?);
    }
    Ok(quote! {
    {
        let mut ret = vec![];
        #(ret.push(#stmt));*;
        ret
    }
    })
}

fn hdl_statement(statement: &syn::Stmt) -> Result<TS> {
    match statement {
        Stmt::Expr(e) => hdl_inner_statement(e),
        Stmt::Semi(e, _) => hdl_inner_statement(e),
        _ => Err(syn::Error::new(
            statement.span(),
            "Local definitions and items are not allowed in HDL kernels",
        )),
    }
}

fn hdl_for_loop(expr: &syn::ExprForLoop) -> Result<TS> {
    if let Pat::Ident(loop_index) = &expr.pat {
        if let Expr::Range(range) = &expr.expr.as_ref() {
            if let Some(from) = range.from.as_ref() {
                if let Some(to) = range.to.as_ref() {
                    let block = hdl_block(&expr.body)?;
                    let loop_index = quote!(#loop_index).to_string();
                    return Ok(quote!(
                    rust_hdl_core::ast::VerilogStatement::Loop(
                        rust_hdl_core::ast::VerilogLoop {
                            index: #loop_index.into(),
                            from: #from.into(),
                            to: #to.into(),
                            block: #block,
                        }
                    )));
                }
            }
        }
    }
    Err(syn::Error::new(
        expr.span(),
        "For loops must be simple (e.g. for <ident> in <const>..<const>",
    ))
}

fn hdl_inner_statement(expr: &syn::Expr) -> Result<TS> {
    match expr {
        Expr::Assign(x) => hdl_assignment(x),
        Expr::If(x) => hdl_conditional(x),
        Expr::Match(x) => hdl_match(x),
        Expr::MethodCall(x) => hdl_method_set(x),
        Expr::Macro(x) => hdl_macro(x),
        Expr::ForLoop(x) => hdl_for_loop(x),
        _ => Err(syn::Error::new(
            expr.span(),
            format!("Expression does not translate {:?}", expr),
        )),
    }
}

fn hdl_assignment(expr: &syn::ExprAssign) -> Result<TS> {
    if let syn::Expr::Index(_) = *expr.left {
        Err(syn::Error::new(
            expr.span(),
            "Indexed assignments do not translate",
        ))
    } else {
        hdl_non_indexed_assignment(expr)
    }
}

fn hdl_non_indexed_assignment(expr: &syn::ExprAssign) -> Result<TS> {
    let target;
    if let Expr::Field(p) = &*expr.left {
        target = hdl_map_field_assign(p)?;
    } else {
        return Err(syn::Error::new(
            expr.span(),
            "unsupported assignment type for HDL",
        ));
    }
    let value = hdl_compute(expr.right.as_ref())?;
    Ok(quote!({
       rust_hdl_core::ast::VerilogStatement::Assignment(#target, #value)
    }))
}

fn hdl_map_field_assign(expr: &syn::ExprField) -> Result<TS> {
    let expr_expanded = common::fixup_ident(quote!(#expr).to_string());
    if expr_expanded.ends_with("_val") {
        return Err(syn::Error::new(
            expr.span(),
            "Do not assign to .val in HDL.  Use .next instead.",
        ));
    }
    Ok(quote!(rust_hdl_core::ast::VerilogExpression::Signal(#expr_expanded.to_string())))
}

fn hdl_map_field(expr: &syn::ExprField) -> Result<TS> {
    let expr_expanded = common::fixup_ident(quote!(#expr).to_string());
    if expr_expanded.ends_with("_next") {
        return Err(syn::Error::new(
            expr.span(),
            "Do not read from .next in HDL.  Use .val instead.",
        ));
    }
    Ok(quote!(rust_hdl_core::ast::VerilogExpression::Signal(#expr_expanded.to_string())))
}

fn hdl_map_path(expr: &syn::ExprPath) -> Result<TS> {
    let expr_expanded = common::fixup_ident(quote!(#expr).to_string());
    Ok(quote!(rust_hdl_core::ast::VerilogExpression::Signal(#expr_expanded.to_string())))
}

fn hdl_conditional(conditions: &syn::ExprIf) -> Result<TS> {
    let test_condition = hdl_compute(&conditions.cond)?;
    let then_branch = hdl_block(&conditions.then_branch)?;
    let mut else_branch = quote!({ rust_hdl_core::ast::VerilogBlockOrConditional::None });
    if let Some((_, e_branch)) = &conditions.else_branch {
        match e_branch.as_ref() {
            Expr::Block(block) => {
                let else_branch_block = hdl_block(&block.block)?;
                else_branch = quote!({rust_hdl_core::ast::VerilogBlockOrConditional::Block(#else_branch_block)});
            }
            Expr::If(cond) => {
                let else_branch_block = hdl_conditional(cond)?;
                else_branch = quote!({rust_hdl_core::ast::VerilogBlockOrConditional::Conditional(Box::new(#else_branch_block))});
            }
            _ => {
                return Err(syn::Error::new(
                    conditions.span(),
                    "Unsupported if/else structure",
                ));
            }
        }
    }
    Ok(quote!({
       rust_hdl_core::ast::VerilogStatement::If(rust_hdl_core::ast::VerilogConditional{test: #test_condition, then: #then_branch, otherwise: #else_branch})
    }))
}

fn hdl_match(m: &syn::ExprMatch) -> Result<TS> {
    let test = hdl_compute(m.expr.as_ref())?;
    let mut condition = vec![];
    let mut blocks = vec![];
    for arm in &m.arms {
        condition.push(hdl_pattern(&arm.pat)?);
        blocks.push(hdl_body(&arm.body)?);
    }
    Ok(quote!({
       {
          let mut cases = vec![];
          #(cases.push(rust_hdl_core::ast::VerilogCase{condition: #condition.to_string(), block: #blocks}));*;
          rust_hdl_core::ast::VerilogStatement::Match(rust_hdl_core::ast::VerilogMatch{test: #test, cases: cases})
       }
    }))
}

fn hdl_compute(m: &syn::Expr) -> Result<TS> {
    match m {
        Expr::Path(path) => hdl_map_path(path),
        Expr::Field(field) => hdl_map_field(field),
        Expr::Paren(paren) => {
            let inner = hdl_compute(&paren.expr)?;
            Ok(quote!(rust_hdl_core::ast::VerilogExpression::Paren(Box::new(#inner))))
        }
        Expr::Binary(binop) => hdl_binop(binop),
        Expr::Unary(unop) => hdl_unop(unop),
        Expr::Call(call) => hdl_call(call),
        Expr::MethodCall(method) => hdl_method(method),
        Expr::Lit(lit) => hdl_literal(lit),
        Expr::Cast(cast) => hdl_cast(&cast),
        _ => Err(syn::Error::new(
            m.span(),
            format!("Unsupported expression type: {:?}", m),
        )),
    }
}

fn hdl_unop(unop: &syn::ExprUnary) -> Result<TS> {
    let arg = hdl_compute(&unop.expr)?;
    let op = match &unop.op {
        UnOp::Not(_) => quote!(rust_hdl_core::ast::VerilogOpUnary::Not),
        UnOp::Neg(_) => quote!(rust_hdl_core::ast::VerilogOpUnary::Neg),
        _ => {
            return Err(syn::Error::new(
                unop.span(),
                "Unsupported operator in HDL code",
            ));
        }
    };
    Ok(quote!({
      rust_hdl_core::ast::VerilogExpression::Unary(#op, Box::new(#arg))
    }))
}

fn hdl_binop(binop: &syn::ExprBinary) -> Result<TS> {
    let left = hdl_compute(&binop.left)?;
    let right = hdl_compute(&binop.right)?;
    let op = match &binop.op {
        BinOp::Add(_) => quote!(rust_hdl_core::ast::VerilogOp::Add),
        BinOp::Sub(_) => quote!(rust_hdl_core::ast::VerilogOp::Sub),
        BinOp::Mul(_) => quote!(rust_hdl_core::ast::VerilogOp::Mul),
        BinOp::And(_) => quote!(rust_hdl_core::ast::VerilogOp::LogicalAnd),
        BinOp::Or(_) => quote!(rust_hdl_core::ast::VerilogOp::LogicalOr),
        BinOp::BitXor(_) => quote!(rust_hdl_core::ast::VerilogOp::BitXor),
        BinOp::BitAnd(_) => quote!(rust_hdl_core::ast::VerilogOp::BitAnd),
        BinOp::BitOr(_) => quote!(rust_hdl_core::ast::VerilogOp::BitOr),
        BinOp::Shl(_) => quote!(rust_hdl_core::ast::VerilogOp::Shl),
        BinOp::Shr(_) => quote!(rust_hdl_core::ast::VerilogOp::Shr),
        BinOp::Eq(_) => quote!(rust_hdl_core::ast::VerilogOp::Eq),
        BinOp::Lt(_) => quote!(rust_hdl_core::ast::VerilogOp::Lt),
        BinOp::Le(_) => quote!(rust_hdl_core::ast::VerilogOp::Le),
        BinOp::Ne(_) => quote!(rust_hdl_core::ast::VerilogOp::Ne),
        BinOp::Ge(_) => quote!(rust_hdl_core::ast::VerilogOp::Ge),
        BinOp::Gt(_) => quote!(rust_hdl_core::ast::VerilogOp::Gt),
        _ => {
            return Err(syn::Error::new(
                binop.span(),
                "Unsupported operator in HDL code",
            ));
        }
    };
    Ok(quote!({
      rust_hdl_core::ast::VerilogExpression::Binary(Box::new(#left), #op, Box::new(#right))
    }))
}

fn hdl_literal(lit: &syn::ExprLit) -> Result<TS> {
    Ok(quote!({
       rust_hdl_core::ast::VerilogExpression::Literal(#lit.into())
    }))
}

fn hdl_cast(cast: &syn::ExprCast) -> Result<TS> {
    let expr = hdl_compute(cast.expr.as_ref())?;
    let dtype = cast.ty.as_ref();
    Ok(quote!({
       rust_hdl_core::ast::VerilogExpression::Cast(Box::new(#expr), #dtype::bits())
    }))
}

fn hdl_call(call: &syn::ExprCall) -> Result<TS> {
    let funcname = quote!(#call).to_string();
    if funcname.starts_with("bit_cast") || funcname.starts_with("tagged_bit_cast") {
        hdl_compute(&call.args[0])
    } else if funcname.starts_with("all_true") {
        let arg = hdl_compute(&call.args[0])?;
        Ok(quote!({
        rust_hdl_core::ast::VerilogExpression::Unary(rust_hdl_core::ast::VerilogOpUnary::All, Box::new(#arg))
        }))
    } else {
        Ok(quote!({
        rust_hdl_core::ast::VerilogExpression::Literal(#call.into())
        }))
    }
}

fn hdl_method_set(method: &syn::ExprMethodCall) -> Result<TS> {
    let method_name = method.method.to_string();
    let field_set_match = regex::Regex::new(r"set_value_([a-zA-Z][a-zA-Z0-9_]*)").unwrap();
    if field_set_match.is_match(method_name.as_ref()) {
        let expr = method.receiver.as_ref();
        let signal = common::fixup_ident(quote!(#expr).to_string());
        let field = field_set_match
            .captures(method_name.as_ref())
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        let get_width_name = format_ident!("get_width_{}", field);
        let get_offset_name = format_ident!("get_my_offset_{}", field);
        let width = quote!(#expr.#get_width_name());
        let offset = quote!(#expr.#get_offset_name());
        let value = hdl_compute(method.args.index(0))?;
        return Ok(quote!({
           rust_hdl_core::ast::VerilogStatement::SliceAssignment{
               base: #signal.to_string(),
               width: #width,
               offset: rust_hdl_core::ast::VerilogExpression::Literal(#offset.into()),
               replacement: #value,
           }
        }));
    } else if method_name == "set_bit" {
        let expr = method.receiver.as_ref();
        let signal = common::fixup_ident(quote!(#expr).to_string());
        let index = hdl_compute(method.args.index(0))?;
        let value = hdl_compute(method.args.index(1))?;
        return Ok(quote!({
           rust_hdl_core::ast::VerilogStatement::SliceAssignment{
               base: #signal.to_string(),
               width: 1,
               offset: #index,
               replacement: #value,
           }
        }));
    } else if method_name == "link" {
        let expr = method.receiver.as_ref();
        let signal = common::fixup_ident(quote!(#expr).to_string());
        let target = method.args.index(0);
        let target = common::fixup_ident(quote!(#target).to_string());
        return Ok(quote!(
            rust_hdl_core::ast::VerilogStatement::Link(#expr.link_hdl(#signal, #signal, #target))
        ));
    }
    Err(syn::Error::new(
        method.span(),
        format!(
            "Unsupported set method {} called for HDL conversion",
            method_name
        ),
    ))
}

fn hdl_method(method: &syn::ExprMethodCall) -> Result<TS> {
    let method_name = method.method.to_string();
    let field_get_match = regex::Regex::new(r"get_value_([a-zA-Z][a-zA-Z0-9_]*)").unwrap();
    if field_get_match.is_match(method_name.as_ref()) {
        let expr = method.receiver.as_ref();
        let signal = common::fixup_ident(quote!(#expr).to_string());
        let field = field_get_match
            .captures(method_name.as_ref())
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        let get_width_name = format_ident!("get_width_{}", field);
        let get_offset_name = format_ident!("get_my_offset_{}", field);
        let width = quote!(#expr.#get_width_name());
        let offset = quote!(#expr.#get_offset_name());
        return Ok(quote!({
           rust_hdl_core::ast::VerilogExpression::Slice(#signal.to_string(), #width, Box::new(rust_hdl_core::ast::VerilogExpression::Literal(#offset.into())))
        }));
    }
    match method_name.as_ref() {
        "get_bits" => {
            let expr = method.receiver.as_ref();
            let target = hdl_compute(expr)?;
            if method.turbofish.is_none() {
                return Err(syn::Error::new(method.span(), "get_bits needs a type argument to indicate the width of the slice (e.g., x.get_bits::<Bits4>(ndx))"));
            }
            if method.turbofish.as_ref().unwrap().args.len() != 1 {
                return Err(syn::Error::new(method.span(), "get_bits needs a type argument to indicate the width of the slice (e.g., x.get_bits::<Bits4>(ndx))"));
            }
            let width_type = method.turbofish.as_ref().unwrap().args.first().unwrap();
            let width = quote!(#width_type);
            if method.args.len() != 1 {
                return Err(syn::Error::new(
                    method.span(),
                    "get_bits needs one argument (offset)",
                ));
            }
            let offset = hdl_compute(&method.args[0])?;
            Ok(quote!({
               rust_hdl_core::ast::VerilogExpression::Slice(Box::new(#target), #width, Box::new(#offset))
            }))
        }
        "get_bit" => {
            let signal = hdl_compute(method.receiver.as_ref())?;
            if method.args.is_empty() {
                return Err(syn::Error::new(
                    method.span(),
                    "get_bit must be supplied with an argument",
                ));
            }
            let index = hdl_compute(method.args.first().unwrap())?;
            Ok(quote!({
               rust_hdl_core::ast::VerilogExpression::Index(Box::new(#signal), Box::new(#index))
            }))
        }
        "replace_bit" => {
            let receiver = hdl_compute(method.receiver.as_ref())?;
            if method.args.len() != 2 {
                return Err(syn::Error::new(
                    method.span(),
                    "set_bit needs two arguments",
                ));
            }
            let index = hdl_compute(method.args.index(0))?;
            let value = hdl_compute(method.args.index(1))?;
            Ok(quote!({
               rust_hdl_core::ast::VerilogExpression::IndexReplace(Box::new(#receiver), Box::new(#index), Box::new(#value))
            }))
        }
        "all" => {
            let target = hdl_compute(method.receiver.as_ref())?;
            Ok(quote!({
                rust_hdl_core::ast::VerilogExpression::Unary(rust_hdl_core::ast::VerilogOpUnary::All, Box::new(#target))
            }))
        }
        "any" => {
            let target = hdl_compute(method.receiver.as_ref())?;
            Ok(quote!({
            rust_hdl_core::ast::VerilogExpression::Unary(rust_hdl_core::ast::VerilogOpUnary::Any,
                Box::new(#target))
            }))
        }
        "xor" => {
            let target = hdl_compute(method.receiver.as_ref())?;
            Ok(quote!({
                rust_hdl_core::ast::VerilogExpression::Unary(rust_hdl_core::ast::VerilogOpUnary::Xor,
                Box::new(#target))
            }))
        }
        "val" | "into" | "index" => {
            let receiver = method.receiver.as_ref();
            hdl_compute(receiver)
        }
        _ => Err(syn::Error::new(
            method.span(),
            "Unsupported method call for hardware conversion",
        )),
    }
}

fn hdl_body(body: &syn::Expr) -> Result<TS> {
    if let Expr::Block(b) = body {
        hdl_block(&b.block)
    } else {
        let statement = hdl_inner_statement(body)?;
        Ok(quote!({ vec![#statement] }))
    }
}

fn hdl_pattern(pat: &Pat) -> Result<String> {
    match pat {
        Pat::Ident(ident) => Ok(ident.ident.to_string()),
        Pat::Lit(lit) => Ok(quote!(#lit).to_string()),
        Pat::Path(pat) => Ok(common::fixup_ident(quote!(#pat).to_string())),
        Pat::Wild(_pat) => Ok("default".to_string()),
        _ => Err(syn::Error::new(
            pat.span(),
            format!(
                "pattern type {:?} is not allowable in match statements for HDL",
                pat
            ),
        )),
    }
}

fn hdl_macro(x: &syn::ExprMacro) -> Result<TS> {
    let ident = &x.mac.path;
    let macro_name = quote!(#ident).to_string();
    let invocation_as_string = quote!(#x).to_string();
    match macro_name.as_ref() {
        "println" => {
            let invocation_as_string = invocation_as_string
                .replace("println ! (\"", "")
                .replace("\")", "");
            Ok(
                quote!(rust_hdl_core::ast::VerilogStatement::Comment(#invocation_as_string.to_string())),
            )
        }
        "comment" => {
            let invocation_as_string = invocation_as_string
                .replace("comment ! (\"", "")
                .replace("\")", "");
            Ok(
                quote!(rust_hdl_core::ast::VerilogStatement::Comment(#invocation_as_string.to_string())),
            )
        }
        "assert" => {
            let invocation_as_string = invocation_as_string
                .replace("assert ! (\"", "")
                .replace("\")", "");
            Ok(
                quote!(rust_hdl_core::ast::VerilogStatement::Comment(#invocation_as_string.to_string())),
            )
        }
        _ => Err(syn::Error::new(
            x.span(),
            "Unsupported macro invocation in HDL",
        )),
    }
}
