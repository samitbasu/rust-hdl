mod common;
mod hdl_gen;
mod logic_block;
mod logic_interface;

use syn::parse_macro_input;
use syn::DeriveInput;

use crate::common::TS;
use crate::hdl_gen::hdl_gen_process;
use crate::logic_block::get_impl_for_logic_block;
use crate::logic_interface::get_impl_for_logic_interface;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(LogicBlock)]
pub fn logic_block(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match get_impl_for_logic_block(&input) {
        Err(e) => e.to_compile_error().into(),
        Ok(x) => x.into(),
    }
}

#[proc_macro_derive(LogicInterface)]
pub fn logic_interface(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match get_impl_for_logic_interface(&input) {
        Err(e) => e.to_compile_error().into(),
        Ok(x) => x.into(),
    }
}

#[proc_macro_attribute]
pub fn hdl_gen(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let orig = TS::from(item.clone());
    let parse = parse_macro_input!(item as syn::ItemFn);

    match hdl_gen_process(parse) {
        Err(e) => e.to_compile_error().into(),
        Ok(hdl_code) => TokenStream::from(quote! {
            #orig

            #hdl_code
        }),
    }
}
