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

#[proc_macro_derive(Serializable2, attributes(encode_decode))]
pub fn encode_decode_derive2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let opts = Opts::from_derive_input(&input).expect("Wrong options");

    let lan = opts.lan.unwrap_or("rust".to_string());

    if lan == "rust" {
        let expanded = quote! {

            impl Serializable2 for #name {
                type EncodeError = EncodeError;
                type DecodeError = bincode::error::DecodeError;
                type Config = bincode::config::Configuration;

                fn serialize(&self, config: &Self::Config) -> Result<Vec<u8>, Self::EncodeError> {
                    let mut encoded = bincode::encode_to_vec(&self, *config)?;
                    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(&mut encoded)?;
                    encoder.finish().map_err(Self::EncodeError::from)
                }

                fn config() -> Self::Config {
                    bincode::config::Configuration::default()
                }
            }
        };

        TokenStream::from(expanded)
    } else {
        let expanded = quote! {
            impl Serializable2 for #name {
                type EncodeError = EncodeError;
                type DecodeError = bincode::error::DecodeError;
                type Config = ();

                fn serialize(&self, config: &Self::Config) -> Result<Vec<u8>, Self::EncodeError> {
                    let any_obj = rutie::AnyObject::from(self.value);
                    let dumpped = rutie::Marshal::dump(any_obj, rutie::NilClass::new().into()).to_string();
                    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(dumpped.as_bytes())?;
                    encoder.finish().map_err(Self::EncodeError::from)
                }

                fn config() -> Self::Config {
                    ()
                }
            }
        };

        TokenStream::from(expanded)
    }
}

#[proc_macro_derive(Serializable, attributes(encode_decode))]
pub fn encode_decode_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let opts = Opts::from_derive_input(&input).expect("Wrong options");

    let lan = opts.lan.unwrap_or("rust".to_string());
    if lan == "rust" {
        let expanded = quote! {
            impl Serializable for #name {
                type Error = bincode::error::EncodeError;
                type DecodeError = bincode::error::DecodeError;
                type Config = bincode::config::Configuration;

                fn encode_to_string(&self, config: &Self::Config) -> Result<String, Self::Error> {
                    let bytes = bincode::encode_to_vec(&self, *config)?;
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
}
