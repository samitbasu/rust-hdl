use crate::common::*;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, Result};

fn get_variant_names(input: &syn::DeriveInput) -> Result<Vec<TS>> {
    let mut variants = vec![];
    match &input.data {
        Data::Enum(ed) => {
            for variant in &ed.variants {
                if variant.fields.len() != 0 {
                    return Err(syn::Error::new(
                        variant.span(),
                        "enum variants cannot have fields",
                    ));
                }
                if variant.discriminant.is_some() {
                    return Err(syn::Error::new(
                        variant.span(),
                        "enum variants cannot have discriminants",
                    ));
                }
                let name = &variant.ident;
                variants.push(quote!(#name));
            }
        }
        _ => {
            return Err(syn::Error::new(
                input.span(),
                "LogicState can only be applied to enums",
            ))
        }
    }
    Ok(variants)
}

pub fn get_logic_state_impls(input: &syn::DeriveInput) -> Result<TS> {
    let variants = get_variant_names(input)?;
    let first_variant = variants[0].clone();
    let num_variants = variants.len();
    let discriminants: Vec<usize> = (0_usize..(variants.len() as usize)).collect();
    let name = &input.ident;
    let name_as_string = name.to_string();
    let variants_as_strings = variants
        .iter()
        .map(|x| format! {"{}::{}", name_as_string, x.to_string()})
        .collect::<Vec<String>>();
    let variants_only_as_strings = variants
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    Ok(quote!(
        impl Synth for #name {
            const BITS: usize = clog2(#num_variants);
            fn descriptor() -> TypeDescriptor {
                TypeDescriptor {
                    name: #name_as_string.to_string(),
                    kind: TypeKind::Enum(vec![#(#variants_as_strings.to_string(),)*])
                }
            }
            fn vcd(self) -> VCDValue {
                match self {
                    #(#name::#variants => VCDValue::String(#variants_only_as_strings.into()),)*
                }
            }
            fn verilog(self) -> VerilogLiteral {
                match self {
                    #(#name::#variants => #discriminants.into(),)*
                }
            }
        }

        impl Into<Bits<{#name::BITS}>> for #name {
            fn into(self) -> Bits<{#name::BITS}> {
                match self {
                    #(#name::#variants => #discriminants.into(),)*
                }
            }
        }

        impl Default for #name {
            fn default() -> #name {
                #name::#first_variant
            }
        }
    ))
}
