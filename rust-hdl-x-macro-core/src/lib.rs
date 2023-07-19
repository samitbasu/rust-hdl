use std::f32::consts::E;

use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct};

pub fn derive_vcd_writeable(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<Item>(input)?;
    match decl {
        Item::Struct(s) => derive_vcd_writeable_struct(s),
        Item::Enum(e) => derive_vcd_writeable_enum(e),
        _ => Err(anyhow!("Only structs and enums supported")),
    }
}

pub fn derive_vcd_writeable_enum(decl: ItemEnum) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let variants = decl.variants.iter().map(|x| &x.ident);
    Ok(quote! {
        impl VCDWriteable for #enum_name {
            fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                w.allocate(name, 0)
            }
            fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                match self {
                    #(
                        Self::#variants => w.serialize_string(stringify!(#variants)),
                    )*
                }
            }
        }
    })
}

pub fn derive_vcd_writeable_struct(decl: ItemStruct) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    if let syn::Fields::Named(field) = &decl.fields {
        let fields = field.named.iter().map(|f| &f.ident);
        let fields2 = fields.clone();
        Ok(quote! {
            impl VCDWriteable for #struct_name {
                fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                    w.push_scope(name);
                    #(
                        self.#fields.register(stringify!(#fields), w)?;
                    )*
                    w.pop_scope();
                    Ok(())
                }
                fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                    #(
                        self.#fields2.serialize(w)?;
                    )*
                    Ok(())
                }
            }
        })
    } else {
        Err(anyhow!("Only named fields supported for structs"))
    }
}

fn assert_tokens_eq(expected: &TokenStream, actual: &TokenStream) {
    let expected = expected.to_string();
    let actual = actual.to_string();

    if expected != actual {
        println!(
            "{}",
            colored_diff::PrettyDifference {
                expected: &expected,
                actual: &actual,
            }
        );
        println!("expected: {}", &expected);
        println!("actual  : {}", &actual);
        panic!("expected != actual");
    }
}

#[test]
fn test_proc_macro() {
    let decl = quote!(
        pub struct NestedBits {
            nest_1: bool,
            nest_2: u8,
            nest_3: TwoBits,
        }
    );
    let output = derive_vcd_writeable(decl).unwrap();
    let expected = quote! {
        impl VCDWriteable for NestedBits {
            fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                w.push_scope(name);
                self.nest_1.register(stringify!(nest_1), w)?;
                self.nest_2.register(stringify!(nest_2), w)?;
                self.nest_3.register(stringify!(nest_3), w)?;
                w.pop_scope();
                Ok(())
            }
            fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                self.nest_1.serialize(w)?;
                self.nest_2.serialize(w)?;
                self.nest_3.serialize(w)?;
                Ok(())
            }
        }
    };
    assert_tokens_eq(&expected, &output);
}

#[test]
fn test_proc_macro_enum() {
    let decls = quote! {
        pub enum Foo {
            Idle,
            Running
        }
    };
    let output = derive_vcd_writeable(decls).unwrap();
    let expected = quote! {
        impl VCDWriteable for Foo {
            fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                w.allocate(name, 0)
            }
            fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                match self {
                    Self::Idle => w.serialize_string(stringify!(Idle)),
                    Self::Running => w.serialize_string(stringify!(Running)),
                }
            }
        }
    };
    assert_tokens_eq(&expected, &output)
}
