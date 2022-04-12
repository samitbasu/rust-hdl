use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, Expr, Token};
use syn::parse::{Parse, ParseStream};

pub(crate) type TS = proc_macro2::TokenStream;

pub(crate) fn get_field_types(input: &syn::DeriveInput) -> syn::Result<Vec<TS>> {
    let mut fields = vec![];
    match &input.data {
        Data::Struct(ds) => {
            for field in &ds.fields {
                if field.ident.is_none() {
                    return Err(syn::Error::new(
                        field.span(),
                        "Unnamed fields are not supported",
                    ));
                }
                let name = &field.ident.as_ref();
                let qname = quote!(#name);
                let qname_string = qname.to_string();
                if ["config", "wire", "reg", "module", "edge", "disable"]
                    .contains(&qname_string.as_str())
                {
                    return Err(syn::Error::new(
                        field.span(),
                        "Cannot use an HDL keyword here",
                    ));
                }
                if !qname_string.starts_with("_") {
                    let ty = &field.ty;
                    fields.push(quote!(#ty));
                }
            }
        }
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "Logic Derive can only be applied to structs",
            ));
        }
    }
    Ok(fields)
}

pub(crate) fn get_field_names(input: &syn::DeriveInput) -> syn::Result<Vec<TS>> {
    let mut fields = vec![];
    match &input.data {
        Data::Struct(ds) => {
            for field in &ds.fields {
                if field.ident.is_none() {
                    return Err(syn::Error::new(
                        field.span(),
                        "Unnamed fields are not supported",
                    ));
                }
                let name = &field.ident.as_ref();
                let qname = quote!(#name);
                let qname_string = qname.to_string();
                if ["config", "wire", "reg", "module", "edge", "disable"]
                    .contains(&qname_string.as_str())
                {
                    return Err(syn::Error::new(
                        field.span(),
                        "Cannot use an HDL keyword here",
                    ));
                }
                if !qname_string.starts_with("_") {
                    fields.push(qname);
                }
            }
        }
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "Logic Derive can only be applied to structs",
            ));
        }
    }
    Ok(fields)
}

pub fn get_connect_all(fields: Vec<TS>) -> syn::Result<TS> {
    Ok(quote! {
        fn connect_all(&mut self) {
            self.connect();
            #(self.#fields.connect_all();)*
        }
    })
}

pub fn get_update_all(fields: Vec<TS>) -> syn::Result<TS> {
    Ok(quote! {
        fn update_all(&mut self) {
            self.update();
            #(self.#fields.update_all();)*
        }
    })
}

pub fn get_has_changed(fields: Vec<TS>) -> syn::Result<TS> {
    if fields.is_empty() {
        Ok(quote! {
            fn has_changed(&self) -> bool {
                false
            }
        })
    } else {
        Ok(quote! {
            fn has_changed(&self) -> bool {
                #(self.#fields.has_changed())||*
            }
        })
    }
}

pub fn squash(x: &str) -> String {
    let y = x.to_string().replace(" ", "").replace("\n", "");
    y
}

pub fn fixup_ident(x: String) -> String {
    let y = x
        .replace(" ", "")
        .replace("self.", "")
        .replace(".", "$")
        .replace("::", "$")
        .replace("&mut", "");
    assert_ne!(y, "config");
    assert_ne!(y, "input");
    assert_ne!(y, "output");
    y
}


// The dff_setup macro uses clock, reset, dfflist arguments
#[derive(Debug)]
pub struct DFFSetupArgs {
    pub me: Expr,
    pub clock: Expr,
    pub reset: Expr,
    pub dffs: Vec<Expr>,
}

impl Parse for DFFSetupArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let clock: Expr;
        let reset: Expr;
        let mut dffs = Vec::new();

        let me: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        clock = input.parse()?;
        input.parse::<Token![,]>()?;
        reset = input.parse()?;
        input.parse::<Token![,]>()?;
        while !input.is_empty() {
            let dff_name: Expr = input.parse()?;
            dffs.push(dff_name);
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(DFFSetupArgs {
            me, clock, reset, dffs
        })
    }
}
