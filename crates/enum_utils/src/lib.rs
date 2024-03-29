
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, Fields};

/// This macro is used to generate the ToIndex trait implementation for an enum.
///
/// Example:
///
/// Macro input:
///
/// ```rust
/// #[derive(ToIndex)]
/// enum Message {
///     Greeting(String),
///     Number(u32),
///     Point { x: i32, y: i32 },
///     Quit,
/// }
/// ```
///
/// Macro output:
///
/// ```rust
/// impl Message {
///     pub fn to_index(&self) -> u32 {
///         match self {
///             Message::Greeting(_) => 0,
///             Message::Number(_) => 1,
///             Message::Point {..} => 2,
///             Message::Quit => 3,
///         }
///     }
/// }
/// ```
///
/// Usage:
/// ```rust
/// let message = Message::Greeting("Hello".to_string());
/// let index = message.to_index();
/// assert_eq!(index, 0);
/// ```
///
#[proc_macro_derive(ToIndex)]
pub fn derive_to_index(input: TokenStream) -> TokenStream {
    // Take any enum and generate the ToIndex trait implementation for it
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let name = &ast.ident;
    let variants = match &ast.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Only enums are supported"),
    };
    
    // Generate the trait implementation for the enum
    let mut match_arms = Vec::new();
    for (index, variant) in variants.iter().enumerate() {
        let i = index as u32;
        let variant_name = &variant.ident;
        let fields = match &variant.fields {
            Fields::Named(_) => quote! { {..} },
            Fields::Unnamed(_) => quote! { (..) },
            Fields::Unit => quote! { },
        };
        match_arms.push(quote! {
            #name::#variant_name #fields => #i,
        });
    }
    
    let gen = quote! {
        impl #name {
            pub fn to_index(&self) -> u32 {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };
    
    // Concatenate the trait definition and implementation and return the resulting token stream
    gen.into()
}
