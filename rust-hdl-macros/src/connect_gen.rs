use crate::common::TS;
use quote::quote;
use std::collections::HashSet;
use syn::visit::Visit;
use syn::{Expr, ExprAssign, Member, Result};

struct AssignVisitor<'ast> {
    targets: HashSet<&'ast Expr>,
}

impl<'ast> Visit<'ast> for AssignVisitor<'ast> {
    fn visit_expr_assign(&mut self, node: &'ast ExprAssign) {
        if let Expr::Field(field) = node.left.as_ref() {
            if let Member::Named(nxt) = &field.member {
                if nxt.eq("next") {
                    self.targets.insert(&field.base);
                }
            }
        }
    }
}

pub(crate) fn connect_gen(item: &syn::ItemFn) -> Result<TS> {
    let mut t = AssignVisitor {
        targets: HashSet::new(),
    };
    t.visit_item_fn(item);
    let connects = t.targets.iter().collect::<Vec<_>>();
    Ok(quote!(
        fn connect(&mut self) {
            #(
                rust_hdl_core::logic::logic_connect_fn(&mut #connects)
            );*;
        }
    ))
}
