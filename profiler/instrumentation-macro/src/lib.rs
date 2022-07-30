extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn function(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let instrumented_function_name = function.sig.ident.to_string();

    let body = &function.block;
    let new_body: syn::Block = impl_block(body, &instrumented_function_name);

    function.block = Box::new(new_body);

    (quote! {
        #function
    })
    .into()
}

#[cfg(feature = "enabled")]
fn impl_block(
    body: &syn::Block,
    instrumented_function_name: &str,
) -> syn::Block {
    parse_quote! {
        {
            profiler::scope!(#instrumented_function_name);
            #body
        }
    }
}

#[cfg(not(feature = "enabled"))]
fn impl_block(
    body: &syn::Block,
    _instrumented_function_name: &str,
) -> syn::Block {
    parse_quote! {
        {
            #body
        }
    }
}
