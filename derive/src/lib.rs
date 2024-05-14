extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Meta, NestedMeta};

// #[proc_macro_derive(EncodeDecode, attributes(config_type, error_type))]
#[proc_macro_derive(EncodeDecode)]
pub fn encode_decode_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Generate the implementation for the trait
    let expanded = quote! {
        impl EncodeDecode for #name {
        // impl<T: EncodeDecode> EncodeDecode for #name<T> {
            type Error = bincode::error::EncodeError;
            type Config = bincode::config::Configuration; // Use the config type provided by the attribute

            fn encode_to_string(&self, config: Self::Config) -> Result<String, Self::Error> {
                let bytes = bincode::encode_to_vec(&self, config)?;
                Ok(base64::encode(bytes))
            }

            fn decode_from_string(val: &String, config: Self::Config) -> Result<String, Self::Error> {
                Ok("str".to_string())
            }
        }
    };

    // Return the generated implementation as a token stream
    TokenStream::from(expanded)
}

// Helper function to extract the Config type from the encode_decode_config attribute
fn extract_type(field: &str, attrs: &[syn::Attribute]) -> syn::Type {
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::List(list) = meta {
                if list.path.is_ident(field) {
                    if let Some(NestedMeta::Meta(Meta::Path(path))) = list.nested.first() {
                        return syn::parse_quote!(#path);
                    }
                }
            }
        }
    }
    // Default to `()` if no encode_decode_config attribute is found
    syn::parse_quote!(())
}
