use anyhow::anyhow;
use anyhow::bail;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_loggable(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<syn::DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_loggable_struct(decl),
        Data::Enum(_e) => derive_loggable_enum(decl),
        _ => Err(anyhow!("Only structs and enums can be loggable")),
    }
}

fn derive_loggable_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            for variant in &e.variants.clone() {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    bail!("Only unit variants are supported for loggable enums")
                }
            }
            Ok(quote! {
                impl #impl_generics Loggable for #enum_name #ty_generics #where_clause {
                    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
                        builder.allocate(tag, 0);
                    }
                    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
                        match self {
                            #(
                                Self::#variants => logger.write_string(stringify!(#variants)),
                            )*
                        }
                    }
                }
            })
        }
        _ => Err(anyhow!("Only enums can be loggable")),
    }
}

fn derive_loggable_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, ty_generics, where_clause) = decl.generics.split_for_impl();
    match decl.data {
        Data::Struct(s) => {
            let fields = s.fields.iter().map(|x| &x.ident);
            let fields2 = fields.clone();
            let field_types = s.fields.iter().map(|x| &x.ty);
            Ok(quote! {
                impl #impl_generics Loggable for #struct_name #ty_generics #where_clause {
                    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
                        #(
                            <#field_types as Loggable>::allocate(tag, builder.namespace(stringify!(#fields)));
                        )*
                    }
                    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
                        #(
                            self.#fields2.record(tag, &mut logger);
                        )*
                    }
                }
            })
        }
        _ => Err(anyhow!("Only structs can be loggable")),
    }
}
