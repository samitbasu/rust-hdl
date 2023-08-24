use anyhow::{anyhow, bail};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub mod loggable;
pub use loggable::derive_loggable;
pub mod kernel;

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
