use anyhow::anyhow;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn derive_vcd_writeable(input: TokenStream) -> anyhow::Result<TokenStream> {
    let decl = syn::parse2::<syn::ItemStruct>(input)?;
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
