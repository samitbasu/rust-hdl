use proc_macro::TokenStream;
use rust_hdl_x_macro_core::derive_vcd_writeable;

#[proc_macro_derive(VCDWriteable)]
pub fn vcd_writeable(input: TokenStream) -> TokenStream {
    derive_vcd_writeable(input.into()).unwrap().into()
}
