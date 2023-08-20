use anyhow::{anyhow, bail};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub mod bit_serialize;
pub mod loggable;
pub mod traceable;
pub use bit_serialize::derive_bit_serialize;
pub use loggable::derive_loggable;
pub use traceable::derive_traceable;

pub fn derive_vcd_writeable(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<DeriveInput>(input)?;
    match &decl.data {
        Data::Struct(_s) => derive_vcd_writeable_struct(decl),
        Data::Enum(e) => derive_vcd_writeable_enum(decl),
        _ => Err(anyhow!("Only structs and enums supported")),
    }
}

pub fn derive_vcd_writeable_enum(decl: DeriveInput) -> anyhow::Result<TokenStream> {
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
                impl #impl_generics VCDWriteable for #enum_name #type_generics #where_clauses {
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
        _ => Err(anyhow!("Only named fields supported for structs")),
    }
}

pub fn derive_vcd_writeable_struct(decl: DeriveInput) -> anyhow::Result<TokenStream> {
    let struct_name = &decl.ident;
    let (impl_generics, type_generics, where_clauses) = decl.generics.split_for_impl();

    match decl.data {
        Data::Struct(s) => {
            let fields = s.fields.iter().map(|f| &f.ident);
            let fields2 = fields.clone();
            Ok(quote! {
                impl #impl_generics VCDWriteable for #struct_name #type_generics #where_clauses {
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
        }
        _ => Err(anyhow!("Only named fields supported for structs")),
    }
}

#[cfg(test)]
fn assert_tokens_eq(expected: &TokenStream, actual: &TokenStream) {
    let expected = expected.to_string();
    let expected = prettyplease::unparse(&syn::parse_file(&expected).unwrap());
    let actual = actual.to_string();
    let actual = prettyplease::unparse(&syn::parse_file(&actual).unwrap());  

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

#[test]
fn test_proc_macro_generics() {
    let decs = quote! {
        pub struct Foo<const N: usize> {
            bar: Bits<N>,
        }
    };
    let output = derive_vcd_writeable(decs).unwrap();
    let expected = quote! {
        impl<const N: usize> VCDWriteable for Foo<N> {
            fn register(&self, name: &str, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                w.push_scope(name);
                self.bar.register(stringify!(bar), w)?;
                w.pop_scope();
                Ok(())
            }
            fn serialize(&self, w: &mut impl VCDWriter) -> anyhow::Result<()> {
                self.bar.serialize(w)?;
                Ok(())
            }
        }
    };
    assert_tokens_eq(&expected, &output)
}

#[test]
fn test_traceable_proc_macro() {
    let decl = quote!(
        pub struct NestedBits {
            nest_1: bool,
            nest_2: u8,
            nest_3: TwoBits,
        }
    );
    let output = derive_traceable(decl).unwrap();
    let expected = quote! {
        impl Traceable for NestedBits {
            fn register_trace_type(tracer: impl TracerBuilder) {
                <bool as Traceable>::register_trace_type(tracer.namespace(stringify!(nest_1)));
                <u8 as Traceable>::register_trace_type(tracer.namespace(stringify!(nest_2)));
                <TwoBits as Traceable>::register_trace_type(tracer.namespace(stringify!(nest_3)));
            }
            fn record(&self, mut tracer: impl Tracer) {
                self.nest_1.record(&mut tracer);
                self.nest_2.record(&mut tracer);
                self.nest_3.record(&mut tracer);
            }
        }
    };
    assert_tokens_eq(&expected, &output);
}

#[test]
fn test_loggable_proc_macro() {
    let decl = quote!(
        pub struct NestedBits {
            nest_1: bool,
            nest_2: u8,
            nest_3: TwoBits,
        }
    );
    let output = derive_loggable(decl).unwrap();
    let expected = quote! {
        impl Loggable for NestedBits {
            fn allocate(builder: impl LogBuilder) {
                <bool as Loggable>::allocate(builder.namespace(stringify!(nest_1)));
                <u8 as Loggable>::allocate(builder.namespace(stringify!(nest_2)));
                <TwoBits as Loggable>::allocate(builder.namespace(stringify!(nest_3)));
            }
            fn record(&self, mut logger: impl Logger) {
                self.nest_1.record(&mut logger);
                self.nest_2.record(&mut logger);
                self.nest_3.record(&mut logger);
            }
        }
    };
    assert_tokens_eq(&expected, &output);
}

#[test]
fn test_loggable_with_struct() {
    let decl = quote!(
        pub struct Inputs {
            pub input: u32,
            pub write: bool,
            pub read: bool,
        }
    );
    let output = derive_loggable(decl).unwrap();
    let expected = quote! {
        impl Loggable for Inputs {
            fn allocate(builder: impl LogBuilder) {
                <u32 as Loggable>::allocate(builder.namespace(stringify!(input)));
                <bool as Loggable>::allocate(builder.namespace(stringify!(write)));
                <bool as Loggable>::allocate(builder.namespace(stringify!(read)));
            }
            fn record(&self, mut logger: impl Logger) {
                self.input.record(&mut logger);
                self.write.record(&mut logger);
                self.read.record(&mut logger);
            }
        }
    };
    assert_tokens_eq(&expected, &output);
}
