use anyhow::{anyhow, bail};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_bit_serialize(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_bit_serialize_struct(decl),
        Data::Enum(e) => derive_bit_serialize_enum(decl),
        _ => Err(anyhow!("Only structs and enums supported")),
    }
}

fn derive_bit_serialize_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let enum_name = &decl.ident;
    let (impl_generics, type_generics, where_clauses) = decl.generics.split_for_impl();

    match decl.data {
        Data::Enum(e) => {
            let variants = e.variants.iter().map(|x| &x.ident);
            for variant in &e.variants.clone() {
                if !matches!(variant.fields, syn::Fields::Unit) {
                    bail!("Only unit variants supported")
                }
            }
            Ok(quote! {
                impl #impl_generics BitSerialize for #enum_name #type_generics #where_clauses {
                    fn serialize(&self, tag: &'static str, serializer: impl BitSerializer){
                        match self {
                            #(
                                Self::#variants => serializer.string(tag, stringify!(#variants)),
                            )*
                        }
                    }
                }
            })
        }
        _ => Err(anyhow!("Only named fields supported for structs")),
    }
}

fn derive_bit_serialize_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, type_generics, where_clauses) = decl.generics.split_for_impl();

    match decl.data {
        Data::Struct(s) => {
            let fields = s.fields.iter().map(|f| &f.ident);
            let field_types = s.fields.iter().map(|f| &f.ty);
            let fields2 = fields.clone();
            let field_types2 = field_types.clone();
            Ok(quote! {
                impl #impl_generics BitSerialize for #struct_name #type_generics #where_clauses {
                    fn serialize(&self, tag: &'static str, serializer: impl BitSerializer){
                        serializer.enter_struct(tag);
                        #(
                            <#field_types2 as BitSerialize>::serialize(&self.#fields2, stringify!(#fields), &serializer);
                        )*
                        serializer.exit_struct();
                    }
                }
            })
        }
        _ => Err(anyhow!("Only named fields supported for structs")),
    }
}

#[cfg(test)]
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
    let output = derive_bit_serialize(decl).unwrap();
    let expected = quote! {
        impl BitSerialize for NestedBits {
            fn serialize(&self, tag: &'static str, serializer: impl BitSerializer){
                serializer.enter_struct(tag);
                <bool as BitSerialize>::serialize(&self.nest_1, stringify!(nest_1), &serializer);
                <u8 as BitSerialize>::serialize(&self.nest_2, stringify!(nest_2), &serializer);
                <TwoBits as BitSerialize>::serialize(&self.nest_3, stringify!(nest_3), &serializer);
                serializer.exit_struct();
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
    let output = derive_bit_serialize(decls).unwrap();
    let expected = quote! {
        impl BitSerialize for Foo {
            fn serialize(&self, tag: &'static str, serializer: impl BitSerializer){
                match self {
                    Self::Idle => serializer.string(tag, stringify!(Idle)),
                    Self::Running => serializer.string(tag, stringify!(Running)),
                }
            }
        }
    };
    assert_tokens_eq(&expected, &output);
}

#[test]
fn test_proc_macro_generics() {
    let decs = quote! {
        pub struct TwoBits<const N: usize> {
            bit_1: bool,
            bit_2: bool,
            part_3: u8,
            nibble_4: Bits<4>,
        }
    };
    let output = derive_bit_serialize(decs).unwrap();
    let expected = quote! {
        impl<const N: usize> BitSerialize for TwoBits<N> {
            fn serialize(&self, tag: &'static str, serializer: impl BitSerializer){
                serializer.enter_struct(tag);
                <bool as BitSerialize>::serialize(&self.bit_1, stringify!(bit_1), &serializer);
                <bool as BitSerialize>::serialize(&self.bit_2, stringify!(bit_2), &serializer);
                <u8 as BitSerialize>::serialize(&self.part_3, stringify!(part_3), &serializer);
                <Bits<4> as BitSerialize>::serialize(&self.nibble_4, stringify!(nibble_4), &serializer);
                serializer.exit_struct();
            }
        }
    };
    assert_tokens_eq(&expected, &output);
}
