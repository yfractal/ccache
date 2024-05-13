extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;   
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(EncodeDecode)]
pub fn encode_decode_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct being derived for
    let name = &input.ident;

    // Check if the struct has named fields
    let has_named_fields = if let Data::Struct(data_struct) = input.data {
        if let Fields::Named(_) = data_struct.fields {
            true
        } else {
            false
        }
    } else {
        false
    };

    // Generate the implementation for the trait
    let expanded = if has_named_fields {
        quote! {
            impl EncodeDecode for #name {
                type Error = String;
                type Config = ();

                fn encode_to_string(&self, _config: &Self::Config) -> Result<String, Self::Error> {
                    Ok("test".to_string())
                }

                fn decode_from_string(_val: &String, _config: &Self::Config) -> Result<Self, Self::Error>  {
                    Ok(Self{x: 1})
                }
            }
        }
    } else {
        quote! {
            // Handling for tuple structs or unit structs
            compile_error!("EncodeDecode can only be derived for structs with named fields");
        }
    };

    // Return the generated implementation as a token stream
    TokenStream::from(expanded)
}
