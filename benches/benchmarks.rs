use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nimbus::in_memory_store::InMemoryStore;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    random_string
}

fn set_redis(key: &str, val: String, redis_conn: &mut redis::Connection) {
    let _: () = redis::cmd("SET")
        .arg(key)
        .arg(val)
        .query(redis_conn)
        .unwrap();
}

fn get_from_redis(key: &str, redis_conn: &mut redis::Connection) -> String {
    redis::cmd("GET").arg(key).query(redis_conn).unwrap()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut redis_conn = client.get_connection().unwrap();

    // setup for raw redis
    set_redis("10_characters", generate_random_string(10), &mut redis_conn); // 10B
    set_redis(
        "1024_characters",
        generate_random_string(1024),
        &mut redis_conn,
    ); // 1KB
    set_redis(
        "1024*16_characters",
        generate_random_string(1024 * 16),
        &mut redis_conn,
    ); // 16KB
    set_redis(
        "1024*1024_characters",
        generate_random_string(1024 * 2024),
        &mut redis_conn,
    ); // 1MB

    // setup for in memory store
    let in_memory_store = InMemoryStore::new();
    in_memory_store
        .insert(
            "10_characters_1",
            generate_random_string(10),
            &mut redis_conn,
        )
        .unwrap();
    in_memory_store
        .insert(
            "1024_characters_1",
            generate_random_string(1024),
            &mut redis_conn,
        )
        .unwrap();
    in_memory_store
        .insert(
            "1024*16_characters_1",
            generate_random_string(1024 * 16),
            &mut redis_conn,
        )
        .unwrap();
    in_memory_store
        .insert(
            "1024*1024_characters_1",
            generate_random_string(1024 * 2024),
            &mut redis_conn,
        )
        .unwrap();

    c.bench_function("in memory cache: 10 characters", |b| {
        b.iter(|| in_memory_store.get(black_box("10_characters_1"), &mut redis_conn))
    });
    c.bench_function("in memory cache: 1024 characters", |b| {
        b.iter(|| in_memory_store.get(black_box("1024_characters_1"), &mut redis_conn))
    });
    c.bench_function("in memory cache: 1024 * 16 characters", |b| {
        b.iter(|| in_memory_store.get(black_box("1024*16_characters_1"), &mut redis_conn))
    });
    c.bench_function("in memory cache: 1024*1024_characters", |b| {
        b.iter(|| in_memory_store.get(black_box("1024*1024_characters_1"), &mut redis_conn))
    });

    c.bench_function("10 characters", |b| {
        b.iter(|| get_from_redis(black_box("10_characters"), &mut redis_conn))
    });
    c.bench_function("1024 characters", |b| {
        b.iter(|| get_from_redis(black_box("1024_characters"), &mut redis_conn))
    });
    c.bench_function("1024 * 16 characters", |b| {
        b.iter(|| get_from_redis(black_box("1024*16_characters"), &mut redis_conn))
    });
    c.bench_function("1024*1024_characters", |b| {
        b.iter(|| get_from_redis(black_box("1024*1024_characters"), &mut redis_conn))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
