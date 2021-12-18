use crate::common::get_field_names;
use crate::common::{get_connect_all, get_has_changed, get_update_all, TS};
use quote::quote;
use syn::{Result, TypeGenerics};
use std::collections::HashMap;
use syn::spanned::Spanned;

pub(crate) fn get_impl_for_logic_interface(input: &syn::DeriveInput) -> Result<TS> {
    let fields = get_field_names(input)?;
    let link = get_link(fields.clone())?;
    let link_hdl = get_link_hdl(fields.clone())?;
    let update_all = get_update_all(fields.clone())?;
    let has_changed = get_has_changed(fields.clone())?;
    let connect_all = get_connect_all(fields.clone())?;
    let join_connect = get_join_connect(fields.clone())?;
    let join_hdl = get_join_hdl(fields.clone())?;
    let accept = get_accept(fields.clone())?;
    let nvps = get_nvps_from_attributes(input)?;
    let (impl_generics, ty_generics, _where_clause) = &input.generics.split_for_impl();
    let name = &input.ident;
    let join = if nvps.contains_key("join") {
        let join_impl = get_join(&nvps["join"], fields.clone(), ty_generics)?;
        quote! {
            impl #impl_generics #name #ty_generics {
                #join_impl
            }
        }
    } else {
        TS::default()
    };
    Ok(quote! {
        impl #impl_generics logic::Logic for #name #ty_generics {
            fn update(&mut self) {}
            fn connect(&mut self) {}
        }

        impl #impl_generics block::Block for #name #ty_generics {
            #connect_all
            #update_all
            #has_changed
            #accept
        }

        impl #impl_generics logic::LogicLink for #name #ty_generics {
            #link
            #link_hdl
        }

        impl #impl_generics logic::LogicJoin for #name #ty_generics {
            #join_connect
            #join_hdl
        }

        #join

    })
}

fn get_join_connect(fields: Vec<TS>) -> Result<TS> {
    Ok(quote! {
        fn join_connect(&mut self) {
            #(self.#fields.join_connect();)*
        }
    })
}

fn get_link(fields: Vec<TS>) -> Result<TS> {
    Ok(quote! {
        fn link(&mut self, other: &mut Self) {
            #(self.#fields.link(&mut other.#fields);)*
        }
        fn link_connect_source(&mut self) {
            #(self.#fields.link_connect_source();)*
        }
        fn link_connect_dest(&mut self) {
            #(self.#fields.link_connect_dest();)*
        }
    })
}

fn get_link_hdl(fields: Vec<TS>) -> Result<TS> {
    let fields_as_strings = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    Ok(quote! {
        fn link_hdl(&self, my_name: &str, this: &str, that: &str) -> Vec<ast::VerilogLink> {
            let mut ret = vec![];
            #(ret.append(&mut self.#fields.link_hdl(#fields_as_strings, this, that));)*
            ret
        }
    })
}

fn get_join_hdl(fields: Vec<TS>) -> Result<TS> {
    let fields_as_strings = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    Ok(quote! {
        fn join_hdl(&self, my_name: &str, this: &str, that: &str) -> Vec<ast::VerilogLink> {
            let mut ret = vec![];
            #(ret.append(&mut self.#fields.join_hdl(#fields_as_strings, this, that));)*
            ret
        }
    })
}

fn get_join(other: &str, fields: Vec<TS>, ty_generics: &TypeGenerics) -> Result<TS> {
    let other = syn::Ident::new(other, proc_macro2::Span::call_site());
    Ok(quote! {
        pub fn join(&mut self, other: &mut #other #ty_generics) {
            #(self.#fields.join(&mut other.#fields);)*
        }
    })
}

fn get_accept(fields: Vec<TS>) -> Result<TS> {
    let fields_as_strings = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    Ok(quote! {
        fn accept(&self, name: &str, probe: &mut dyn probe::Probe) {
            probe.visit_start_namespace(name, self);
            #(self.#fields.accept(#fields_as_strings, probe);)*
            probe.visit_end_namespace(name, self);
        }
    })
}

fn get_nvps_from_attributes(input: &syn::DeriveInput) -> Result<HashMap<String, String>> {
    let mut ret = HashMap::new();
    for attr in &input.attrs {
        let meta = attr.parse_meta()?;
        match meta {
            syn::Meta::NameValue(nv) => {
                let path = &nv.path;
                let lit = &nv.lit;
                match lit {
                    syn::Lit::Str(s) => {
                        ret.insert(quote!(#path).to_string(), s.value());
                    }
                    _ => {return Err(syn::Error::new(attr.span(), "Argument to bus attribute should be the name of the class that implements the bus"))}
                }
            }
            _ => {
                // Skip non-NVP attributes
            }
        }
    }
    Ok(ret)
}
