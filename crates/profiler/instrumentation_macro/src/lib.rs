extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn, AttributeArgs};

#[proc_macro_attribute]
pub fn function(
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let instrumented_function_name = function.sig.ident.to_string();
    
    // Parse the attribute TokenStream to get the pinned parameter
    let args = parse_macro_input!(attr as AttributeArgs);
    
    // resolve if call was [function(pinned)] or just [function]
    let pinned = match args.len() {
        0 => false,
        1 => {
            let arg = &args[0];
            match arg {
                syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                    path.is_ident("pinned")
                }
                _ => panic!("Invalid attribute argument"),
            }
        }
        _ => panic!("Invalid attribute arguments"),
    };
    
    let body = &function.block;
    let new_body = if !pinned {
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
