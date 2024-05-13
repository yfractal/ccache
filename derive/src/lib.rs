use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyTrait)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl MyTrait for #ident {}
    };
    output.into()
}

#[proc_macro_derive(EncodeDecode)]
pub fn encode_decode_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct being derived for
    let name = &input.ident;

    // Generate the implementation for the trait
    let expanded = quote! {
        impl EncodeDecode for #name {
            type Error = String;
            type Config = ();

            fn encode_to_string(&self, _config: &Self::Config) -> Result<String, Self::Error> {
                Ok("test".to_string())
            }

            fn decode_from_string(_val: &String, _config: &Self::Config) -> Result<String, Self::Error> {
                Ok("abc".to_string())
            }
        }
    };

    // Return the generated implementation as a token stream
    TokenStream::from(expanded)
}
