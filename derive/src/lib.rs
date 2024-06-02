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
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let opts = Opts::from_derive_input(&input).expect("Wrong options");

    let lan = opts.lan.unwrap_or("rust".to_string());

    if lan == "rust" {
        let expanded = quote! {

            impl Serializable for #name {
                type EncodeError = EncodeError;
                type DecodeError = DecodeError;
                type Config = bincode::config::Configuration;

                fn serialize(&self, config: &Self::Config) -> Result<Vec<u8>, Self::EncodeError> {
                    let mut encoded = bincode::encode_to_vec(&self, *config)?;
                    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(&mut encoded)?;
                    encoder.finish().map_err(Self::EncodeError::from)
                }

                fn deserialize(val: &Vec<u8>, config: &Self::Config) -> Result<(Self, usize), Self::DecodeError> {
                    let mut writer = Vec::new();
                    let mut z = flate2::write::ZlibDecoder::new(writer);
                    z.write_all(&val[..])?;
                    writer = z.finish()?;
                    bincode::decode_from_slice(&writer, *config).map_err(Self::DecodeError::from)
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
                type EncodeError = EncodeError;
                type DecodeError = DecodeError;
                type Config = ();

                fn serialize(&self, config: &Self::Config) -> Result<Vec<u8>, Self::EncodeError> {
                    let any_obj = rutie::AnyObject::from(self.value);
                    let dumpped = rutie::Marshal::dump(any_obj, rutie::NilClass::new().into());
                    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
                    encoder.write_all(dumpped.to_bytes_unchecked())?;
                    encoder.finish().map_err(Self::EncodeError::from)
                }

                fn deserialize(val: &Vec<u8>, config: &Self::Config) -> Result<(Self, usize), Self::DecodeError> {
                    let mut writer = Vec::new();
                    let mut z = flate2::write::ZlibDecoder::new(writer);
                    z.write_all(&val[..])?;
                    writer = z.finish()?;

                    let bts = writer.as_ptr() as *const c_char;
                    let len = writer.len() as c_long;
                    let str = unsafe { string::rb_str_new(bts, len) };

                    let any_obj = rutie::Marshal::load(RString::from(str));
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
