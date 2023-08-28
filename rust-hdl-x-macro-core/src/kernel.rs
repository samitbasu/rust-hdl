use quote::quote;
use syn::spanned::Spanned;
type TS = proc_macro2::TokenStream;
type Result<T> = syn::Result<T>;

pub fn hdl_block(block: &syn::Block) -> Result<TS> {
    let stmts = block.stmts.iter().map(stmt).collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        vec![#(#stmts),*]
    })
}

fn stmt(statement: &syn::Stmt) -> Result<TS> {
    match statement {
        syn::Stmt::Local(local) => stmt_local(local),
        syn::Stmt::Expr(expr, semi) => {
            let expr = hdl_expr(expr)?;
            if semi.is_some() {
                Ok(quote! {
                    rust_hdl_x::ast::Stmt::Semi(#expr)
                })
            } else {
                Ok(quote! {
                    rust_hdl_x::ast::Stmt::Expr(#expr)
                })
            }
        }
        _ => Err(syn::Error::new(
            statement.span(),
            "Unsupported statement type",
        )),
    }
}

fn stmt_local(local: &syn::Local) -> Result<TS> {
    let pattern = local_pattern(&local.pat)?;
    let local_init = local
        .init
        .as_ref()
        .map(|x| hdl_expr(&x.expr))
        .ok_or(syn::Error::new(
            local.span(),
            "Unsupported local declaration",
        ))??;
    Ok(quote! {
        rust_hdl_x::ast::Stmt::Local(Local{pattern: #pattern, value: Box::new(#local_init)})
    })
}

fn local_pattern(pat: &syn::Pat) -> Result<TS> {
    match pat {
        syn::Pat::Ident(ident) => {
            let name = &ident.ident;
            let mutability = ident.mutability.is_some();
            if ident.by_ref.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "Unsupported reference pattern",
                ));
            }
            Ok(quote! {
                rust_hdl_x::ast::LocalPattern::Ident(
                    rust_hdl_x::ast::LocalIdent{
                        name: stringify!(#name).to_string(),
                        mutable: #mutability
                    }
                )
            })
        }
        syn::Pat::TupleStruct(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(local_pattern)
                .collect::<Result<Vec<_>>>()?;
            Ok(quote! {
                rust_hdl_x::ast::LocalPattern::Tuple(vec![#(#elems),*])
            })
        }
        _ => Err(syn::Error::new(pat.span(), "Unsupported pattern type")),
    }
}

fn hdl_expr(expr: &syn::Expr) -> Result<TS> {
    match expr {
        syn::Expr::Lit(lit) => hdl_lit(lit),
        syn::Expr::Binary(binary) => hdl_binary(binary),
        syn::Expr::Unary(unary) => hdl_unary(unary),
        syn::Expr::Paren(paren) => hdl_expr(&paren.expr),
        syn::Expr::Assign(assign) => hdl_assign(assign),
        syn::Expr::Path(path) => hdl_path(&path.path),
        syn::Expr::Struct(structure) => hdl_struct(structure),
        _ => Err(syn::Error::new(expr.span(), "Unsupported expression type")),
    }
}

fn hdl_struct(structure: &syn::ExprStruct) -> Result<TS> {
    let path = hdl_path(&structure.path)?;
    let fields = structure
        .fields
        .iter()
        .map(hdl_field)
        .collect::<Result<Vec<_>>>()?;
    if structure.qself.is_some() {
        return Err(syn::Error::new(
            structure.span(),
            "Unsupported qualified self",
        ));
    }
    let rest = structure.rest.as_ref().map(|x| hdl_expr(&x)).transpose()?;
    Ok(quote! {
        rust_hdl_x::ast::Expr::Struct(
            rust_hdl_x::ast::ExprStruct {
                path: #path,
                fields: vec![#(#fields),*],
                rest: #rest,
            }
        )
    })
}

fn hdl_path(path: &syn::Path) -> Result<TS> {
    let ident = path
        .get_ident()
        .ok_or(syn::Error::new(path.span(), "Unsupported path expression"))?;
    Ok(quote! {
        rust_hdl_x::ast::Expr::Ident(stringify!(#ident).to_string())
    })
}

fn hdl_assign(assign: &syn::ExprAssign) -> Result<TS> {
    let left = hdl_expr(&assign.left)?;
    let right = hdl_expr(&assign.right)?;
    Ok(quote! {
        rust_hdl_x::ast::Expr::Assign(Box::new(#left), Box::new(#right))
    })
}

fn hdl_field(field: &syn::FieldValue) -> Result<TS> {
    let member = hdl_member(&field.member)?;
    let expr = hdl_expr(&field.expr)?;
    Ok(quote! {
        rust_hdl_x::ast::ExprField {
            member: #member,
            expr: Box::new(#expr),
        }
    })
}

fn hdl_member(member: &syn::Member) -> Result<TS> {
    Ok(match member {
        syn::Member::Named(ident) => quote! {
            rust_hdl_x::ast::Member::Named(stringify!(#ident).to_string())
        },
        syn::Member::Unnamed(index) => {
            let index = index.index;
            quote! {
                rust_hdl_x::ast::Member::Unnamed(#index)
            }
        }
    })
}

fn hdl_unary(unary: &syn::ExprUnary) -> Result<TS> {
    let op = match unary.op {
        syn::UnOp::Neg(_) => quote!(rust_hdl_x::ast::UnOp::Neg),
        syn::UnOp::Not(_) => quote!(rust_hdl_x::ast::UnOp::Not),
        _ => return Err(syn::Error::new(unary.span(), "Unsupported unary operator")),
    };
    let expr = hdl_expr(&unary.expr)?;
    Ok(quote! {
        rust_hdl_x::ast::Expr::Unary(
            rust_hdl_x::ast::ExprUnary
            {
                op: #op,
                expr: Box::new(#expr)
            }
        )
    })
}

fn hdl_binary(binary: &syn::ExprBinary) -> Result<TS> {
    let op = match binary.op {
        syn::BinOp::Add(_) => quote!(rust_hdl_x::ast::BinOp::Add),
        syn::BinOp::Sub(_) => quote!(rust_hdl_x::ast::BinOp::Sub),
        syn::BinOp::Mul(_) => quote!(rust_hdl_x::ast::BinOp::Mul),
        syn::BinOp::And(_) => quote!(rust_hdl_x::ast::BinOp::And),
        syn::BinOp::Or(_) => quote!(rust_hdl_x::ast::BinOp::Or),
        syn::BinOp::BitXor(_) => quote!(rust_hdl_x::ast::BinOp::BitXor),
        syn::BinOp::BitAnd(_) => quote!(rust_hdl_x::ast::BinOp::BitAnd),
        syn::BinOp::BitOr(_) => quote!(rust_hdl_x::ast::BinOp::BitOr),
        syn::BinOp::Shl(_) => quote!(rust_hdl_x::ast::BinOp::Shl),
        syn::BinOp::Shr(_) => quote!(rust_hdl_x::ast::BinOp::Shr),
        syn::BinOp::Eq(_) => quote!(rust_hdl_x::ast::BinOp::Eq),
        syn::BinOp::Lt(_) => quote!(rust_hdl_x::ast::BinOp::Lt),
        syn::BinOp::Le(_) => quote!(rust_hdl_x::ast::BinOp::Le),
        syn::BinOp::Ne(_) => quote!(rust_hdl_x::ast::BinOp::Ne),
        syn::BinOp::Ge(_) => quote!(rust_hdl_x::ast::BinOp::Ge),
        syn::BinOp::Gt(_) => quote!(rust_hdl_x::ast::BinOp::Gt),
        syn::BinOp::AddAssign(_) => quote!(rust_hdl_x::ast::BinOp::AddAssign),
        syn::BinOp::SubAssign(_) => quote!(rust_hdl_x::ast::BinOp::SubAssign),
        syn::BinOp::MulAssign(_) => quote!(rust_hdl_x::ast::BinOp::MulAssign),
        syn::BinOp::BitXorAssign(_) => quote!(rust_hdl_x::ast::BinOp::BitXorAssign),
        syn::BinOp::BitAndAssign(_) => quote!(rust_hdl_x::ast::BinOp::BitAndAssign),
        syn::BinOp::BitOrAssign(_) => quote!(rust_hdl_x::ast::BinOp::BitOrAssign),
        syn::BinOp::ShlAssign(_) => quote!(rust_hdl_x::ast::BinOp::ShlAssign),
        syn::BinOp::ShrAssign(_) => quote!(rust_hdl_x::ast::BinOp::ShrAssign),
        _ => {
            return Err(syn::Error::new(
                binary.span(),
                "Unsupported binary operator",
            ))
        }
    };
    let left = hdl_expr(&binary.left)?;
    let right = hdl_expr(&binary.right)?;
    Ok(quote! {
        rust_hdl_x::ast::Expr::Binary(
            rust_hdl_x::ast::ExprBinary {
                op: #op,
                lhs: Box::new(#left),
                rhs: Box::new(#right),
            }
        )
    })
}

fn hdl_lit(lit: &syn::ExprLit) -> Result<TS> {
    let lit = &lit.lit;
    match lit {
        syn::Lit::Int(int) => {
            let value = int.token();
            Ok(quote! {
                rust_hdl_x::ast::Expr::Lit(
                    rust_hdl_x::ast::ExprLit::Int(stringify!(#value).to_string())
                )
            })
        }
        syn::Lit::Bool(boolean) => {
            let value = boolean.value;
            Ok(quote! {
                rust_hdl_x::ast::Expr::Lit(
                    rust_hdl_x::ast::ExprLit::Bool(#value)
                )
            })
        }
        _ => Err(syn::Error::new(lit.span(), "Unsupported literal type")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic_block() {
        let test_code = quote! {
            {
                let a = 1;
                let b = 2;
                let q = 0x1234_u32;
                let c = a + b;
                let mut d = 3;
                let g = Foo {r: 1, g: 120, b: 33};
                //let h = g.r;
                c
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = hdl_block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rust_hdl_x :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
    #[test]
    fn test_precedence_parser() {
        let test_code = quote! {
            {
                1 + 3 * 9
            }
        };
        let block = syn::parse2::<syn::Block>(test_code).unwrap();
        let result = hdl_block(&block).unwrap();
        let result = format!("fn jnk() -> Vec<Stmt> {{ {} }}", result);
        let result = result.replace("rust_hdl_x :: ast :: ", "");
        let result = prettyplease::unparse(&syn::parse_file(&result).unwrap());
        println!("{}", result);
    }
}
