use quote::quote;
use syn::Result;

use crate::common;
use crate::common::TS;

pub(crate) fn get_impl_for_logic_block(input: &syn::DeriveInput) -> Result<TS> {
    let fields = common::get_field_names(input)?;
    let update_all = common::get_update_all(fields.clone())?;
    let has_changed = common::get_has_changed(fields.clone())?;
    let connect_all = common::get_connect_all(fields.clone())?;
    let accept = get_accept(fields.clone())?;
    let name = &input.ident;
    let (impl_generics, ty_generics, _where_clause) = &input.generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics block::Block for #name #ty_generics {
            #connect_all
            #update_all
            #has_changed
            #accept
        }
    })
}

fn get_accept(fields: Vec<TS>) -> Result<TS> {
    let fields_as_strings = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    Ok(quote! {
        fn accept(&self, name: &str, probe: &mut dyn probe::Probe) {
            probe.visit_start_scope(name, self);
            #(self.#fields.accept(#fields_as_strings, probe);)*
            probe.visit_end_scope(name, self);
        }
    })
}
