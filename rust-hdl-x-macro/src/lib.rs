use proc_macro::TokenStream;
use rust_hdl_x_macro_core::derive_vcd_writeable;

#[proc_macro_derive(VCDWriteable)]
pub fn vcd_writeable(input: TokenStream) -> TokenStream {
    derive_vcd_writeable(input.into()).unwrap().into()
}

#[proc_macro_derive(BitSerialize)]
pub fn bit_serialize(input: TokenStream) -> TokenStream {
    rust_hdl_x_macro_core::derive_bit_serialize(input.into())
        .unwrap()
        .into()
}
