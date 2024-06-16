use crate::serializable::Serializable;
use crate::trace;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use likely_stable::{if_likely, likely, unlikely};
use probe::probe;
use redis::Script;
use uuid::Uuid;

struct DataInner<T>(Vec<u8>, Arc<T>);

impl<T> DataInner<T> {
    pub fn val(&self) -> Arc<T> {
        self.1.clone()
    }
}

pub struct Data<T: Serializable> {
    inner: HashMap<String, DataInner<T>>,
}

impl<T: Serializable> Data<T> {
    pub fn new() -> Self {
        Data {
            inner: HashMap::new(),
        }
    }
}

pub struct InMemoryStore<T: Serializable> {
    data: RwLock<Data<T>>,
    pub coder_config: T::Config,
}

enum GetThroughLocalResult {
    None,
    Unchanged,
    New(Vec<u8>, Vec<u8>),
}

#[derive(Debug)]
pub enum GetResult<T> {
    None,
    Unchanged(T),
    New(T),
}

impl<T> GetResult<T> {
    pub fn unwrap(self) -> T {
        match self {
            GetResult::Unchanged(val) => val,
            GetResult::New(val) => val,
            GetResult::None => panic!("called `GetResult::unwrap()` on a `None` value"),
        }
    }
}

const GET_FROM_REDIS_SCRIPT: &str = r#"
if (redis.call("HGET", KEYS[1], "etag") == ARGV[1]) then
   return {"etag","-1"}
else
   return redis.call("HGETALL", KEYS[1])
end
"#;

const INSERT_TO_REDIS_SCRIPT: &str = r#"
  local time = redis.call('TIME')[1]
  redis.call("HSET", KEYS[1], "val", ARGV[1], "etag", time)

  return time
"#;

const ETAG_UNCHANGED: &[u8] = "-1".as_bytes();

impl<T: Serializable> InMemoryStore<T> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(Data::new()),
            coder_config: T::config(),
        }
    }

    pub fn insert(
        &self,
        key: &str,
        val: T,
        redis_conn: &mut redis::Connection,
    ) -> Result<Vec<u8>, redis::RedisError> {
        let uuid = Uuid::new_v4();

        probe!(
            ccache,
            store,
            trace::Event::new("insert", "start", key, &uuid.to_string()).as_ptr()
        );

        let val_arc = Arc::new(val);
        let etag = self.insert_to_redis(uuid, key, val_arc.clone(), redis_conn)?;
        let mut data = self.data.write().unwrap();
        data.inner
            .insert(key.to_string(), DataInner(etag.clone(), val_arc.clone()));

        probe!(
            ccache,
            store,
            trace::Event::new("insert", "end", key, &uuid.to_string()).as_ptr()
        );

        Ok(etag)
    }

    #[inline]
    pub fn get(
        &self,
        key: &str,
        redis_conn: &mut redis::Connection,
    ) -> Result<GetResult<Arc<T>>, redis::RedisError> {
        let uuid = Uuid::new_v4();

        probe!(
            ccache,
            store,
            trace::Event::new("get", "start", key, &uuid.to_string()).as_ptr()
        );

        let data = self.data.read().unwrap();

        if_likely! {let Some(DataInner(etag, _)) = data.inner.get(key) => {
                match try_get_from_local(uuid, key, etag, redis_conn) {
                    Ok(GetThroughLocalResult::Unchanged) => {
                        let obj = data.inner.get(key).unwrap().val().clone();

                        probe!(ccache, store, trace::Event::new("get", "end", key, &uuid.to_string()).as_ptr());

                        Ok(GetResult::Unchanged(obj.clone()))
                    }
                    Ok(GetThroughLocalResult::None) => {
                        probe!(ccache, store, trace::Event::new("get", "end", key, &uuid.to_string()).as_ptr());

                        Ok(GetResult::None)
                    },
                    Ok(GetThroughLocalResult::New(val, etag)) => {
                        let (decoded, _): (T, usize) = T::deserialize(&val, &self.coder_config).unwrap();
                        let decoded_arc = Arc::new(decoded);
                        drop(data);
                        let mut data = self.data.write().unwrap();
                        data.inner.insert(key.to_string(), DataInner(etag,  decoded_arc.clone()));

                        probe!(ccache, store, trace::Event::new("get", "end", key, &uuid.to_string()).as_ptr());

                        Ok(GetResult::New(decoded_arc))
                    }
                    Err(e) => {
                        probe!(ccache, store, trace::Event::new("get", "end", key, &uuid.to_string()).as_ptr());

                        Err(e)
                    },
                }
            } else {
                let redis_result = get_from_redis_request(uuid, key, redis_conn)?;
                if redis_result.is_empty() {
                    probe!(ccache, store, trace::Event::new("get", "end", key, &uuid.to_string()).as_ptr());

                    Ok(GetResult::None) // redis missed
                } else {
                    let val = redis_result.get("val").unwrap();

                    let etag = redis_result.get("etag").unwrap();

                    let (decoded, _): (T, usize) = T::deserialize(&val, &self.coder_config).unwrap();
                    let decoded_arc = Arc::new(decoded);

                    drop(data); // release read lock
                    let mut data = self.data.write().unwrap(); // acquire write lock
                    data.inner.insert(key.to_string(), DataInner(etag.clone(),  decoded_arc.clone()));
                    probe!(ccache, store, trace::Event::new("get", "end", key, &uuid.to_string()).as_ptr());

                    Ok(GetResult::New(decoded_arc))
                }
            }
        }
    }

    fn insert_to_redis(
        &self,
        uuid: Uuid,
        key: &str,
        obj: Arc<T>,
        redis_conn: &mut redis::Connection,
    ) -> Result<Vec<u8>, redis::RedisError> {
        probe!(
            ccache,
            store,
            trace::Event::new("insert_to_redis", "start", key, &uuid.to_string()).as_ptr()
        );

        let val = obj.serialize(&self.coder_config).unwrap();
        let etag = self.insert_to_redis_request(uuid, key, val, redis_conn)?;

        probe!(
            ccache,
            store,
            trace::Event::new("insert_to_redis", "end", key, &uuid.to_string()).as_ptr()
        );

        Ok(etag.clone())
    }

    fn insert_to_redis_request(
        &self,
        uuid: Uuid,
        key: &str,
        val: Vec<u8>,
        redis_conn: &mut redis::Connection,
    ) -> Result<Vec<u8>, redis::RedisError> {
        probe!(
            ccache,
            store,
            trace::Event::new("insert_to_redis_request", "start", key, &uuid.to_string()).as_ptr()
        );

        let result: Result<Vec<u8>, redis::RedisError> = Script::new(INSERT_TO_REDIS_SCRIPT)
            .key(key)
            .arg(val)
            .invoke(redis_conn);

        probe!(
            ccache,
            store,
            trace::Event::new("insert_to_redis_request", "end", key, &uuid.to_string()).as_ptr()
        );

        result
    }
}

fn get_from_redis_request(
    uuid: Uuid,
    key: &str,
    conn: &mut redis::Connection,
) -> Result<HashMap<String, Vec<u8>>, redis::RedisError> {
    probe!(
        ccache,
        store,
        trace::Event::new("get_from_redis_request", "start", key, &uuid.to_string()).as_ptr()
    );

    let result = redis::cmd("HGETALL").arg(key.to_string()).query(conn);

    probe!(
        ccache,
        store,
        trace::Event::new("get_from_redis_request", "end", key, &uuid.to_string()).as_ptr()
    );

    result
}

#[inline]
fn try_get_from_local(
    uuid: Uuid,
    key: &str,
    etag: &Vec<u8>,
    conn: &mut redis::Connection,
) -> Result<GetThroughLocalResult, redis::RedisError> {
    let redis_result = get_from_redis_through_etag(uuid, key, etag, conn)?;
    if unlikely(redis_result.is_empty()) {
        Ok(GetThroughLocalResult::None)
    } else if likely(redis_result.get("etag").unwrap() == ETAG_UNCHANGED) {
        Ok(GetThroughLocalResult::Unchanged)
    } else {
        let val = redis_result.get("val").unwrap().to_vec();
        let etag = redis_result.get("etag").unwrap();
        Ok(GetThroughLocalResult::New(val, etag.clone()))
    }
}

#[inline]
fn get_from_redis_through_etag(
    uuid: Uuid,
    key: &str,
    etag: &Vec<u8>,
    conn: &mut redis::Connection,
) -> Result<HashMap<String, Vec<u8>>, redis::RedisError> {
    probe!(
        ccache,
        store,
        trace::Event::new(
            "get_from_redis_through_etag",
            "start",
            key,
            &uuid.to_string()
        )
        .as_ptr()
    );

    // NOTICE HGETALLETAG is in a self build Redis, it works like GET_FROM_REDIS_SCRIPT
    // see: https://github.com/yfractal/redis/commit/629fbc49a7f6167a6f7980e932e7f3554212b031
    // let result = redis::cmd("HGETALLETAG")
    //     .arg(key.to_string())
    //     .arg(etag.to_string())
    //     .query(conn);
    let result = Script::new(GET_FROM_REDIS_SCRIPT)
        .key(key)
        .arg(etag)
        .invoke(conn);

    probe!(
        ccache,
        store,
        trace::Event::new("get_from_redis_through_etag", "end", key, &uuid.to_string()).as_ptr()
    );

    result
}

#[cfg(test)]
mod tests {
    extern crate flate2;
    use super::*;
    use crate::errors::DecodeError;
    use crate::errors::EncodeError;
    use crate::serializable::Serializable;
    use bincode::{Decode, Encode};
    use derive::Serializable;
    use flate2::Compression;
    use std::io::Write;

    impl<T: Serializable> InMemoryStore<T> {
        pub fn delete(&self, key: &str) {
            let mut data = self.data.write().unwrap();
            data.inner.remove(key);
        }

        pub fn update_etag(&self, key: &str, new_etag: &str) {
            let mut data = self.data.write().unwrap();
            data.inner.get_mut(key).unwrap().0 = new_etag.as_bytes().to_vec();
        }
    }

    impl<T: PartialEq> PartialEq for GetResult<T> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (GetResult::None, GetResult::None) => true,
                (GetResult::Unchanged(a), GetResult::Unchanged(b)) => a == b,
                (GetResult::New(a), GetResult::New(b)) => a == b,
                _ => false,
            }
        }
    }

    #[derive(Encode, Decode, Serializable, PartialEq, Debug, Clone)]
    struct Entity {
        x: f32,
        y: f32,
    }

    #[derive(Encode, Decode, Serializable, PartialEq, Debug, Clone)]
    struct World(Vec<Entity>);

    struct TestContext<T: Serializable> {
        in_memory_store: InMemoryStore<T>,
        redis_conn: redis::Connection,
    }

    impl<T: Serializable> Drop for TestContext<T> {
        fn drop(&mut self) {
            let _: () = redis::cmd("FLUSHDB").query(&mut self.redis_conn).unwrap();
        }
    }

    fn setup<T: Serializable>() -> TestContext<T> {
        let in_memory_store = InMemoryStore::new();

        // Connect to Redis
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let redis_conn = client.get_connection().unwrap();

        TestContext {
            in_memory_store,
            redis_conn,
        }
    }

    #[test]
    fn test_get() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;
        let key = "some-key";
        let val = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);
        let _ = in_memory_store.insert(key, val, &mut ctx.redis_conn);

        let result = in_memory_store
            .get(key, &mut ctx.redis_conn)
            .unwrap()
            .unwrap();
        assert_eq!(result.0.first().unwrap().x, 0.0);
        assert_eq!(result.0.first().unwrap().y, 4.0);

        assert_eq!(result.0.get(1).unwrap().x, 10.0);
        assert_eq!(result.0.get(1).unwrap().y, 20.5);
    }

    #[test]
    fn test_get_none_exist() {
        let mut ctx = setup::<World>();
        let in_memory_store = &ctx.in_memory_store;
        let result = in_memory_store
            .get("non-exist-key", &mut ctx.redis_conn)
            .unwrap();

        assert_eq!(result, GetResult::None);
    }

    #[test]
    fn test_get_local_miss_remote_hit() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;

        // insert one and delete from local
        in_memory_store
            .insert("some-key", Entity { x: 0.0, y: 4.0 }, &mut ctx.redis_conn)
            .unwrap();
        in_memory_store.delete("some-key");

        let result = in_memory_store
            .get("some-key", &mut ctx.redis_conn)
            .unwrap()
            .unwrap();
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 4.0);
    }

    #[test]
    fn test_local_cached_remote_does_not_exist() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;

        // insert one and delete redis
        in_memory_store
            .insert("some-key", Entity { x: 0.0, y: 4.0 }, &mut ctx.redis_conn)
            .unwrap();
        redis::cmd("del")
            .arg("some-key".to_string())
            .query::<bool>(&mut ctx.redis_conn)
            .unwrap();

        let result = in_memory_store
            .get("some-key", &mut ctx.redis_conn)
            .unwrap();

        assert_eq!(result, GetResult::None);
    }

    #[test]
    fn test_local_miss() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;

        // insert one and update etag
        in_memory_store
            .insert("some-key", Entity { x: 0.0, y: 4.0 }, &mut ctx.redis_conn)
            .unwrap();
        in_memory_store.update_etag("some-key", "abc");

        let result = in_memory_store
            .get("some-key", &mut ctx.redis_conn)
            .unwrap()
            .unwrap();
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 4.0);
    }
}
