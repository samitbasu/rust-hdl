use crate::common::{get_field_names, get_field_types};
use crate::TS;
use quote::{format_ident, quote};
use syn::Result;

pub(crate) fn get_impl_for_logic_struct(input: &syn::DeriveInput) -> Result<TS> {
    let fields = get_field_names(input)?;
    let field_types = get_field_types(input)?;
    let get_width_names = fields
        .iter()
        .map(|x| format_ident!("get_my_width_{}", x.to_string()))
        .collect::<Vec<_>>();
    let get_offset_names = fields
        .iter()
        .map(|x| format_ident!("get_my_offset_{}", x.to_string()))
        .collect::<Vec<_>>();
    let mut prev_field = vec![];
    for ndx in 0..fields.len() {
        let mut previous_fields = vec![];
        for prev in 0..ndx {
            previous_fields.push(field_types[prev].clone());
        }
        prev_field.push(quote!(#(+<#previous_fields>::BITS)*));
    }
    let (impl_generics, ty_generics, _where_clause) = &input.generics.split_for_impl();
    let name = &input.ident;
    Ok(quote! {
        impl #impl_generics #name #ty_generics {
            #(
              pub fn #get_width_names(&self) -> usize {
                    <#field_types>::BITS
                }

                pub fn #get_offset_names(&self) -> usize {
                    0_usize #prev_field
                }
            )*
        }

        impl #impl_generics From<#name #ty_generics> for Bits<{<#name>::BITS}> {
            fn from(x: #name) -> Self {
                Bits::<{<#name>::BITS}>::default()  #(|
                    (bit_cast::<{<#name>::BITS}, {<#field_types>::BITS}>(x.#fields.into())
                    << (<#name>::default().#get_offset_names().to_bits()))
                )*
            }
        }

        impl #impl_generics Synth for #name #ty_generics {
            const BITS: usize = 0_usize #(+<#field_types>::BITS)*;

            fn descriptor() -> TypeDescriptor {
                TypeDescriptor {
                    name: stringify!(#name).to_string(),
                    kind: TypeKind::Composite(
                        vec![ #( Box::new(
                            TypeField {
                                fieldname: stringify!(#fields).to_string(),
                                kind: <#field_types>::descriptor()
                            }) ,
                        )*]
                    )
                }
            }

            fn vcd(self) -> VCDValue {
                let mut ret = vec![];
                #(ret.push(Box::new(self.#fields.vcd()));)*
                VCDValue::Composite(ret)
            }

            fn verilog(self) -> VerilogLiteral {
                let t: Bits<{Self::BITS}> = self.into();
                t.into()
            }
        }
    })
}
