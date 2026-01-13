extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Ident, ItemFn, Result,
};

struct MyAttributeArgs {
    pinned: bool,
}

impl Parse for MyAttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pinned = false;

        if input.is_empty() {
            return Ok(MyAttributeArgs { pinned });
        }

        // Input is not empty - expect valid flag (only pinned for now)

        let ident: Ident = input.parse()?;
        if ident == "pinned" {
            pinned = true;
        } else {
            return Err(input.error("Expected `pinned`"));
        }

        Ok(MyAttributeArgs { pinned })
    }
}

#[proc_macro_attribute]
pub fn function(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let instrumented_function_name = function.sig.ident.to_string();

    // Parse the attribute TokenStream to get the pinned parameter
    let args = parse_macro_input!(attr as MyAttributeArgs);

    let body = &function.block;
    let new_body = if !args.pinned {
        parse_quote! {
            {
                profiler::scope!(concat!(module_path!(), "::", #instrumented_function_name));
                #body
            }
        }
    } else {
        parse_quote! {
            {
                profiler::scope!(concat!(module_path!(), "::", #instrumented_function_name), pinned);
                #body
            }
        }
    };

    function.block = Box::new(new_body);

    (quote! {
        #function
    })
    .into()
}
