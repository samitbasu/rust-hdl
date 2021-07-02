mod logic_block;
mod logic_interface;
mod common;

use syn::parse_macro_input;
use syn::DeriveInput;

use proc_macro::TokenStream;
use crate::logic_block::get_impl_for_logic_block;
use crate::logic_interface::get_impl_for_logic_interface;

#[proc_macro_derive(LogicBlock)]
pub fn logic_block(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let impl_ts = get_impl_for_logic_block(&input);
    if impl_ts.is_err() {
        return impl_ts.err().unwrap().to_compile_error().into();
    }
    TokenStream::from(impl_ts.unwrap())
}

#[proc_macro_derive(LogicInterface)]
pub fn logic_interface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let impl_ts = get_impl_for_logic_interface(&input);
    if impl_ts.is_err() {
        return impl_ts.err().unwrap().to_compile_error().into();
    }
    TokenStream::from(impl_ts.unwrap())
}