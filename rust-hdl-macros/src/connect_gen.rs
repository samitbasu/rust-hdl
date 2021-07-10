use crate::common::{TS, fixup_ident};
use syn::{Result, ExprAssign, Expr, visit};
use syn::visit::Visit;
use quote::quote;
use quote::format_ident;
use std::collections::HashSet;


struct AssignVisitor {
    targets: HashSet<Expr>,
}

impl<'ast> Visit<'ast> for AssignVisitor {
    fn visit_expr_assign(&mut self, node: &'ast ExprAssign) {
        let expr = &node.left;
        let expr = syn::parse_str(&quote!(#expr).to_string().replace(".next",".connect()")).expect("unable to parse");
        self.targets.insert(expr);
        visit::visit_expr_assign(self, node);
    }
}

pub(crate) fn connect_gen(item: &syn::ItemFn) -> Result<TS> {
    let mut t = AssignVisitor {
        targets: HashSet::new()
    };
    t.visit_item_fn(item);
    let connects = t.targets.iter().collect::<Vec<_>>();
    Ok(quote!(
        fn connect(&mut self) {
            #(#connects);*;
        }
    ))
}