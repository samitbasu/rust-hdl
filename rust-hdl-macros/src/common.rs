use quote::quote;
use syn::spanned::Spanned;
use syn::Data;

pub(crate) type TS = proc_macro2::TokenStream;

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
