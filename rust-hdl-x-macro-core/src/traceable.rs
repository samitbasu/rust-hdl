use anyhow::anyhow;
use anyhow::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_traceable(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_traceable_struct(decl),
        Data::Enum(_e) => derive_traceable_enum(decl),
        _ => Err(anyhow!("Only structs and enums can be Traceable")),
    }
}

fn derive_traceable_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            for variant in &e.variants.clone() {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    bail!("Only unit variants are supported for Traceable enums")
                }
            }
            Ok(quote! {
                impl #impl_generics Traceable for #enum_name #ty_generics #where_clause {
                    fn register_trace_type(tracer: impl TracerBuilder) {
                        tracer.register(0);
                    }
                    fn record(&self, mut tracer: impl Tracer) {
                        match self {
                            #(
                                Self::#variants => tracer.write_string(stringify!(#variants)),
                            )*
                        }
                    }
                }
            })
        }
        _ => Err(anyhow!("Only enums can be Traceable")),
    }
}

fn derive_traceable_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s.fields.iter().map(|x| &x.ident);
            let fields2 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            Ok(quote! {
                impl #impl_generics Traceable for #struct_name #ty_generics #where_clause {
                    fn register_trace_type(tracer: impl TracerBuilder) {
                        #(
                            <#field_types as Traceable>::register_trace_type(tracer.namespace(stringify!(#fields)));
                        )*
                    }
                    fn record(&self, mut tracer: impl Tracer) {
                        #(
                            self.#fields2.record(&mut tracer);
                        )*
                    }
                }
            })
        }
        _ => Err(anyhow!("Only structs can be Traceable")),
    }
}
