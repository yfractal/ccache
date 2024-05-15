extern crate proc_macro;
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(encode_decode))]
struct Opts {
    lan: Option<String>,
}

#[proc_macro_derive(Serializable, attributes(encode_decode))]
pub fn encode_decode_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let opts = Opts::from_derive_input(&input).expect("Wrong options");

    // Generate the implementation for the trait
    let lan = opts.lan.unwrap_or("rust".to_string());
    if lan == "rust" {
        let expanded = quote! {
            impl Serializable for #name {
                type Error = bincode::error::EncodeError;
                type DecodeError = bincode::error::DecodeError;
                type Config = bincode::config::Configuration; // Use the config type provided by the attribute

                fn encode_to_string(&self, config: &Self::Config) -> Result<String, Self::Error> {
                    let bytes = bincode::encode_to_vec(&self, *config)?;
                    println!("rust encode_to_string");
                    Ok(base64::encode(bytes))
                }

                fn decode_from_string(val: &String, config: &Self::Config) -> Result<(Self, usize), Self::DecodeError> {
                    let bytes = base64::decode(val).unwrap();
                    bincode::decode_from_slice(&bytes, *config)
                }

                fn config() -> Self::Config {
                    bincode::config::Configuration::default()
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        let expanded = quote! {
            impl Serializable for #name {
                type Error = ();
                type DecodeError = ();
                type Config = ();

                fn encode_to_string(&self, config: &Self::Config) -> Result<String, Self::Error> {
                    let any_obj = rutie::AnyObject::from(self.value);
                    let rv = rutie::Marshal::dump(any_obj, rutie::NilClass::new().into()).to_string();

                    Ok(rv)
                }

                fn decode_from_string(val: &String, config: &Self::Config) -> Result<(Self, usize), Self::DecodeError> {
                    let any_obj = rutie::Marshal::load(RString::new(val));
                    let obj = Self{value: any_obj.value()};
                    Ok((obj, 0))
                }

                fn config() -> Self::Config {
                    ()
                }
            }
        };

        TokenStream::from(expanded)
    }

    // Return the generated implementation as a token stream
}
