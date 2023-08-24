use proc_macro::TokenStream;

#[proc_macro_derive(Loggable)]
pub fn loggable(input: TokenStream) -> TokenStream {
    rust_hdl_x_macro_core::derive_loggable(input.into())
        .unwrap()
        .into()
}
