use crate::partitioned_hash_map::PartitionedHashMap;
use crate::serializable::Serializable;
use crate::trace;

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

use likely_stable::{likely, unlikely};
use probe::probe;
use redis::Script;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct CcacheRedisError {
    pub description: String,
}

impl From<redis::RedisError> for CcacheRedisError {
    fn from(e: redis::RedisError) -> Self {
        CcacheRedisError {
            description: e.to_string(),
        }
    }
}

struct DataInner<T>(Vec<u8>, Arc<T>);

impl<T> DataInner<T> {
    pub fn val(&self) -> Arc<T> {
        self.1.clone()
    }

    pub fn etag(&self) -> &Vec<u8> {
        &self.0
    }
}

pub struct InMemoryStore<T: Serializable> {
    coder_config: T::Config,
    map: PartitionedHashMap<String, Arc<DataInner<T>>, RandomState>,
    request_condvar:
        PartitionedHashMap<(String, String), Arc<(Mutex<RedisMessage<T>>, Condvar)>, RandomState>,
}

enum RequestThroughLocalResult {
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

#[derive(Debug)]
enum RedisResult<T> {
    None,
    Unchanged,
    New(Arc<T>),
    Error(CcacheRedisError),
}

struct RedisMessage<T> {
    notified: bool,
    waiting_count: i32,
    redis_result: Option<Arc<RedisResult<T>>>,
}

impl<T> RedisMessage<T> {
    fn new() -> Self {
        RedisMessage {
            notified: false,
            waiting_count: 0,
            redis_result: None,
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
            coder_config: T::config(),
            map: PartitionedHashMap::new(),
            request_condvar: PartitionedHashMap::new(),
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
        let mut map = self.map.write_guard(&key.to_string());
        let etag = self.insert_to_redis(uuid, key, val_arc.clone(), redis_conn)?;

        map.insert(
            key.to_string(),
            Arc::new(DataInner(etag.clone(), val_arc.clone())),
        );

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
    ) -> Result<GetResult<Arc<T>>, CcacheRedisError> {
        let uuid = Uuid::new_v4();

        probe!(
            ccache,
            store,
            trace::Event::new("get", "start", key, &uuid.to_string()).as_ptr()
        );

        let map = self.map.read_guard(&key.to_string());

        let (etag, val) = match map.get(key) {
            Some(d) => (d.etag(), Some(d.val())),
            None => (&ETAG_UNCHANGED.to_vec(), None),
        };

        let request_key = (key.to_string(), String::from_utf8(etag.to_vec()).unwrap());

        let request_read_shard = self.request_condvar.read_guard(&request_key);

        match self
            .request_condvar
            .get_through_shard(&request_key, &request_read_shard)
        {
            // request is undergoing, wait for request
            Some(pair) => {
                return self.wait_for_request(pair.clone(), val, &request_key);
            }
            None => {
                let pair: Arc<(Mutex<RedisMessage<T>>, Condvar)> =
                    Arc::new((Mutex::new(RedisMessage::new()), Condvar::new()));
                // release read lock
                drop(request_read_shard);
                // require write lock
                let mut write_shard = self.request_condvar.write_guard(&request_key);

                match write_shard.insert(request_key.clone(), pair.clone()) {
                    Some(pair) => {
                        // inserted by other thread, drop lock and wait for request
                        drop(write_shard);

                        return self.wait_for_request(pair.clone(), val, &request_key);
                    }
                    None => {
                        // inserted, do request
                        let (lock, cvar) = &*pair;
                        match request_through_etag(uuid, key, etag, redis_conn) {
                            Ok(RequestThroughLocalResult::Unchanged) => {
                                probe!(
                                    ccache,
                                    store,
                                    trace::Event::new("get", "end", key, &uuid.to_string())
                                        .as_ptr()
                                );

                                let mut message = lock.lock().unwrap();
                                message.notified = true;
                                message.redis_result = Some(Arc::new(RedisResult::Unchanged));
                                cvar.notify_one();

                                // map' shard write lock release here
                                return Ok(GetResult::Unchanged(val.unwrap().clone()));
                            }
                            Ok(RequestThroughLocalResult::None) => {
                                probe!(
                                    ccache,
                                    store,
                                    trace::Event::new("get", "end", key, &uuid.to_string())
                                        .as_ptr()
                                );

                                let mut message = lock.lock().unwrap();
                                message.notified = true;
                                message.redis_result = Some(Arc::new(RedisResult::None));
                                cvar.notify_one();

                                return Ok(GetResult::None);
                            }
                            Ok(RequestThroughLocalResult::New(val, etag)) => {
                                let (decoded, _): (T, usize) =
                                    T::deserialize(&val, &self.coder_config).unwrap();
                                let decoded_arc = Arc::new(decoded);

                                // release read lock, acquire write lock and block read
                                drop(map);
                                let mut map = self.map.write_guard(&key.to_string());
                                map.insert(
                                    key.to_string(),
                                    Arc::new(DataInner(etag, decoded_arc.clone())),
                                );

                                probe!(
                                    ccache,
                                    store,
                                    trace::Event::new("get", "end", key, &uuid.to_string())
                                        .as_ptr()
                                );

                                let mut message = lock.lock().unwrap();
                                message.notified = true;
                                message.redis_result =
                                    Some(Arc::new(RedisResult::New(decoded_arc.clone())));
                                cvar.notify_one();

                                return Ok(GetResult::New(decoded_arc));
                            }
                            Err(e) => {
                                probe!(
                                    ccache,
                                    store,
                                    trace::Event::new("get", "end", key, &uuid.to_string())
                                        .as_ptr()
                                );

                                let mut message = lock.lock().unwrap();
                                message.notified = true;
                                let ccache_error: CcacheRedisError = e.into();
                                message.redis_result =
                                    Some(Arc::new(RedisResult::Error(ccache_error.clone())));
                                cvar.notify_one();

                                return Err(ccache_error);
                            }
                        }
                    }
                }
            }
        }
    }

    fn wait_for_request_cleanup(
        &self,
        message: &mut std::sync::MutexGuard<RedisMessage<T>>,
        request_key: &(String, String),
    ) {
        message.waiting_count -= 1;

        if message.waiting_count == 0 {
            let mut write_shard = self.request_condvar.write_guard(&request_key);
            write_shard.remove(request_key);
        }
    }

    fn wait_for_request(
        &self,
        pair: Arc<(Mutex<RedisMessage<T>>, Condvar)>,
        val: Option<Arc<T>>,
        request_key: &(String, String),
    ) -> Result<GetResult<Arc<T>>, CcacheRedisError> {
        let (lock, cvar) = &*pair.clone();
        let mut message = lock.lock().unwrap();
        message.waiting_count += 1;

        while !message.notified {
            let mut message: std::sync::MutexGuard<RedisMessage<T>> = cvar.wait(message).unwrap();
            self.wait_for_request_cleanup(&mut message, request_key);

            match &mut message.redis_result {
                Some(arc_result) => match &**arc_result {
                    RedisResult::None => {
                        return Ok(GetResult::None);
                    }
                    RedisResult::Unchanged => {
                        return Ok(GetResult::Unchanged(val.unwrap().clone()));
                    }
                    RedisResult::New(arc_value) => {
                        return Ok(GetResult::New(arc_value.clone()));
                    }
                    RedisResult::Error(e) => {
                        return Err(e.clone());
                    }
                },
                None => {
                    panic!("Something wrong");
                }
            }
        }

        panic!("Something wrong");
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

#[inline]
fn request_through_etag(
    uuid: Uuid,
    key: &str,
    etag: &Vec<u8>,
    conn: &mut redis::Connection,
) -> Result<RequestThroughLocalResult, redis::RedisError> {
    let redis_result = get_from_redis_through_etag(uuid, key, etag, conn)?;
    if unlikely(redis_result.is_empty()) {
        Ok(RequestThroughLocalResult::None)
    } else if likely(redis_result.get("etag").unwrap() == ETAG_UNCHANGED) {
        Ok(RequestThroughLocalResult::Unchanged)
    } else {
        let val = redis_result.get("val").unwrap().to_vec();
        let etag = redis_result.get("etag").unwrap();
        Ok(RequestThroughLocalResult::New(val, etag.clone()))
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
            let mut map = self.map.write_guard(&key.to_string());
            map.remove(key);
        }

        pub fn update_etag(&self, key: &str, new_etag: &str) {
            let mut map = self.map.write_guard(&key.to_string());
            let data: &mut Arc<DataInner<T>> = map.get_mut(key).unwrap();
            let val = data.val();
            map.insert(
                key.to_string(),
                Arc::new(DataInner(new_etag.as_bytes().to_vec(), val.clone())),
            );
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

        // insert one entity and update etag
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
