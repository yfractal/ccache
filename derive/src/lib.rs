extern crate proc_macro;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Meta, NestedMeta};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(encode_decode))]
struct Opts {
    lan: Option<String>,
}

// #[proc_macro_derive(EncodeDecode, attributes(config_type, error_type))]
#[proc_macro_derive(EncodeDecode, attributes(encode_decode))]
pub fn encode_decode_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let opts = Opts::from_derive_input(&input).expect("Wrong options");
    print!("{:?} is ...", opts.lan);

    // Generate the implementation for the trait
    let lan = opts.lan.unwrap_or("rust".to_string());
    if lan == "rust" {
        let expanded = quote! {
            impl EncodeDecode for #name {
                // impl<T: EncodeDecode> EncodeDecode for #name<T> {
                type Error = bincode::error::EncodeError;
                type DecodeError = bincode::error::DecodeError;
                type Config = bincode::config::Configuration; // Use the config type provided by the attribute

                fn encode_to_string(&self, config: Self::Config) -> Result<String, Self::Error> {
                    let bytes = bincode::encode_to_vec(&self, config)?;
                    println!("rust encode_to_string");
                    Ok(base64::encode(bytes))
                }

                fn decode_from_string(val: &String, config: Self::Config) -> Result<(Self, usize), Self::DecodeError> {
                    let bytes = base64::decode(val).unwrap();
                    bincode::decode_from_slice(&bytes, config)
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        let expanded = quote! {
            impl EncodeDecode for #name {
                // impl<T: EncodeDecode> EncodeDecode for #name<T> {
                type Error = bincode::error::EncodeError;
                type DecodeError = bincode::error::DecodeError;
                type Config = bincode::config::Configuration; // Use the config type provided by the attribute

                fn encode_to_string(&self, config: Self::Config) -> Result<String, Self::Error> {
                    println!("other encode_to_string");
                    let bytes = bincode::encode_to_vec(&self, config)?;
                    Ok(base64::encode(bytes))
                }

                fn decode_from_string(val: &String, config: Self::Config) -> Result<(Self, usize), Self::DecodeError> {
                    let bytes = base64::decode(val).unwrap();
                    bincode::decode_from_slice(&bytes, config)
                }
            }
        };

        TokenStream::from(expanded)
    }


    // Return the generated implementation as a token stream

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
