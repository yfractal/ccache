#[macro_use]
extern crate rutie;
extern crate flate2;
extern crate lazy_static;

use ccache::errors::DecodeError;
use ccache::errors::EncodeError;
use ccache::serializable::Serializable;
use derive::Serializable;
use flate2::Compression;
use rutie::rubysys::string;
use rutie::types::{c_char, c_long};
use rutie::{AnyObject, Class, Object, RString, AnyException, Exception, NilClass, VM};
use std::io::Write;

#[derive(Serializable, Debug)]
#[encode_decode(lan = "ruby")]
pub struct RubyObject {
    pub value: rutie::types::Value,
}

pub struct Store {
    inner: ccache::in_memory_store::InMemoryStore<RubyObject>,
    redis_client: redis::Connection,
}

impl Store {
    fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let redis_client = redis::Client::open(redis_url)?;
        let redic_connection = redis_client.get_connection()?;

        let store = Store {
            inner: ccache::in_memory_store::InMemoryStore::new(),
            redis_client: redic_connection
        };

        Ok(store)
    }
}

wrappable_struct!(Store, StoreWrapper, STORE_WRAPPER);
class!(RubyStore);

methods!(
    RubyStore,
    rtself,
    fn ruby_new(redis_host: RString) -> AnyObject {
        let redis_host = redis_host.unwrap().to_string();

        match Store::new(&redis_host) {
            Ok(store) => {
                Class::from_existing("RubyStore").wrap_data(store, &*STORE_WRAPPER)
            },
            Err(error) => {
                let standard_error = Class::from_existing("CcacheRedisError");
                VM::raise(standard_error, &error.to_string());
                NilClass::new().into()
            }
        }
    },
    fn ruby_insert(key: RString, obj: AnyObject) -> RString {
        let rbself = rtself.get_data_mut(&*STORE_WRAPPER);
        let ruby_object = RubyObject {
            value: obj.unwrap().value(),
        };

        let _ = rbself
            .inner
            .insert(key.unwrap().to_str(), ruby_object, &mut rbself.redis_client)
            .unwrap();

        // TODO: return etag as sting
        RString::new_utf8("")
    },
    fn ruby_get(key: RString) -> AnyObject {
        let rbself = rtself.get_data_mut(&*STORE_WRAPPER);
        let result = rbself.inner.get(key.unwrap().to_str(), &mut rbself.redis_client);

        match result {
            Ok(Some(val)) => { AnyObject::from(val.value) },
            Ok(None) => { NilClass::new().into() },
            Err(e) => AnyException::new(&e.to_string(), None).into()
        }
        // let object = rbself
        //     .inner
        //     .get(key.unwrap().to_str(), &mut rbself.redis_client)
        //     .unwrap();
        // match object {
        //     Some(val) => { AnyObject::from(val.value) },
        //     None =>  NilClass::new().into()
        // }
        // AnyObject::from(object.value)
        // AnyException::new("MyGem::MyError", None).into()
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_ruby_example() {
    Class::new("RubyStore", None).define(|klass| {
        klass.def_self("new", ruby_new);
        klass.def("insert", ruby_insert);
        klass.def("get", ruby_get);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccache::in_memory_store::InMemoryStore;
    use rutie::{Boolean, VM};
    use rutie::{AnyException, Exception, Object, Class};


    #[test]
    fn it_works2() {
        VM::init();
        let mut klass = Class::new("MyGem", None);
        let se = Class::from_existing("StandardError");
        let _ = klass.define_nested_class("MyError", Some(&se));

        assert_eq!(
          AnyException::new("MyGem::MyError", None).to_s(),
          "MyGem::MyError"
        );
    }

    #[test]
    fn it_works() {
        VM::init();
        let store = InMemoryStore::new();
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut redis_conn = client.get_connection().unwrap();
        let ruby_object = RubyObject {
            value: Boolean::new(true).value(),
        };
        store.insert("a-key", ruby_object, &mut redis_conn).unwrap();
        let inserted = store.get("a-key", &mut redis_conn).unwrap();

        assert_eq!(true, inserted.unwrap().value == Boolean::new(true).value());
    }
}
