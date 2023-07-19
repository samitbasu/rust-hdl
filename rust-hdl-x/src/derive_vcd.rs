use quote::quote;

#[test]
fn test_proc_macro() {
    let decl = quote!(
        pub struct NestedBits {
            nest_1: bool,
            nest_2: u8,
            nest_3: TwoBits,
        }
    );
    let decl = syn::parse2::<syn::ItemStruct>(decl).unwrap();
    let struct_name = &decl.ident;
    if let syn::Fields::Named(field) = &decl.fields {
        let fields = field.named.iter().map(|f| &f.ident);
        let fields2 = fields.clone();
        let output = quote! {
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
        };
        println!("{}", output);
    } else {
        panic!("Expected named fields");
    }
}
