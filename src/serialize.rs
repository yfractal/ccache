trait EncodeDecode {
    type Error;
    type Config;

    fn encode_to_string(&self, config: &Self::Config) -> Result<String, Self::Error>;
    fn decode_from_string(val: &String, config: &Self::Config) -> Result<Self, Self::Error> where Self: Sized;
}

pub struct Store<T: EncodeDecode> {
    pub coder_config: T::Config,
}

impl <T: EncodeDecode> Store<T> {
    pub fn new(coder_config: T::Config) -> Self {
        Store {
            coder_config,
        }
    }

    pub fn encode_to_string(&self, val: &T) -> Result<String, T::Error> {
        val.encode_to_string(&self.coder_config)
    }

    pub fn decode_from_string(&self, val: &String) -> Result<T, T::Error>{
        T::decode_from_string(val, &self.coder_config)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        struct TestCoder {
            x: u32,
        };

        impl EncodeDecode for TestCoder {
            type Error = String;
            type Config = ();

            fn encode_to_string(&self, _config: &Self::Config) -> Result<String, Self::Error> {
                Ok("test".to_string())
            }

            fn decode_from_string(_val: &String, _config: &Self::Config) -> Result<Self, Self::Error>  {
                Ok(Self{x: 1})
            }
        }

        let store: Store<TestCoder> = Store::new(());
        let test_coder = TestCoder{x: 2};
        let encoded = store.encode_to_string(&test_coder).unwrap();
        assert_eq!(encoded, "test");
        let decoded = store.decode_from_string(&"val".to_string()).unwrap();
        assert_eq!(decoded.x, 1);

    }
}
