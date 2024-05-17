use ccache::in_memory_store::InMemoryStore;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
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
        "512_characters",
        generate_random_string(1024),
        &mut redis_conn,
    ); // 512B
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
            "512_characters_1",
            generate_random_string(1024),
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

    c.bench_function("10 characters", |b| {
        b.iter(|| get_from_redis(black_box("10_characters"), &mut redis_conn))
    });
    c.bench_function("512 characters", |b| {
        b.iter(|| get_from_redis(black_box("512_characters"), &mut redis_conn))
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

    c.bench_function("in memory cache: 10 characters", |b| {
        b.iter(|| in_memory_store.get(black_box("10_characters_1"), &mut redis_conn))
    });
    c.bench_function("in memory cache: 512 characters", |b| {
        b.iter(|| in_memory_store.get(black_box("512_characters_1"), &mut redis_conn))
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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

// Benchmark result
//      Running benches/benchmarks.rs (target/release/deps/benchmarks-47f29179f47dc569)
// 10 characters           time:   [15.339 µs 15.380 µs 15.417 µs]
//                         change: [-0.7397% -0.3811% -0.0521%] (p = 0.03 < 0.05)
//                         Change within noise threshold.
// Found 11 outliers among 100 measurements (11.00%)
//   4 (4.00%) low severe
//   2 (2.00%) low mild
//   3 (3.00%) high mild
//   2 (2.00%) high severe

// 512 characters          time:   [16.422 µs 16.459 µs 16.497 µs]
// Found 12 outliers among 100 measurements (12.00%)
//   1 (1.00%) low severe
//   1 (1.00%) low mild
//   9 (9.00%) high mild
//   1 (1.00%) high severe

// 1024 characters         time:   [16.220 µs 16.311 µs 16.395 µs]
//                         change: [-0.6704% -0.1588% +0.3597%] (p = 0.53 > 0.05)
//                         No change in performance detected.
// Found 10 outliers among 100 measurements (10.00%)
//   1 (1.00%) low severe
//   1 (1.00%) low mild
//   2 (2.00%) high mild
//   6 (6.00%) high severe

// 1024 * 16 characters    time:   [20.256 µs 20.306 µs 20.352 µs]
//                         change: [-2.3232% -2.0104% -1.7094%] (p = 0.00 < 0.05)
//                         Performance has improved.
// Found 9 outliers among 100 measurements (9.00%)
//   3 (3.00%) low mild
//   3 (3.00%) high mild
//   3 (3.00%) high severe

// 1024*1024_characters    time:   [510.98 µs 514.21 µs 517.82 µs]
//                         change: [+0.4764% +1.5642% +2.7098%] (p = 0.01 < 0.05)
//                         Change within noise threshold.
// Found 6 outliers among 100 measurements (6.00%)
//   4 (4.00%) high mild
//   2 (2.00%) high severe

// in memory cache: 10 characters
//                         time:   [16.065 µs 16.120 µs 16.173 µs]
//                         change: [-1.1085% -0.6258% -0.1752%] (p = 0.01 < 0.05)
//                         Change within noise threshold.
// Found 7 outliers among 100 measurements (7.00%)
//   2 (2.00%) low severe
//   3 (3.00%) high mild
//   2 (2.00%) high severe

// in memory cache: 512 characters
//                         time:   [16.234 µs 16.270 µs 16.304 µs]
//                         change: [-0.2712% +0.0655% +0.3773%] (p = 0.69 > 0.05)
//                         No change in performance detected.
// Found 3 outliers among 100 measurements (3.00%)
//   1 (1.00%) low mild
//   2 (2.00%) high mild

// in memory cache: 1024 characters
//                         time:   [16.171 µs 16.200 µs 16.231 µs]
//                         change: [-0.9520% -0.5451% -0.1669%] (p = 0.01 < 0.05)
//                         Change within noise threshold.
// Found 2 outliers among 100 measurements (2.00%)
//   1 (1.00%) high mild
//   1 (1.00%) high severe

// in memory cache: 1024 * 16 characters
//                         time:   [16.295 µs 16.326 µs 16.359 µs]
//                         change: [+0.0778% +0.4794% +0.8665%] (p = 0.02 < 0.05)
//                         Change within noise threshold.
// Found 4 outliers among 100 measurements (4.00%)
//   1 (1.00%) low mild
//   3 (3.00%) high mild

// in memory cache: 1024*1024_characters
//                         time:   [16.262 µs 16.361 µs 16.516 µs]
//                         change: [-0.1653% +0.4529% +1.2218%] (p = 0.22 > 0.05)
//                         No change in performance detected.
// Found 8 outliers among 100 measurements (8.00%)
//   1 (1.00%) low severe
//   2 (2.00%) low mild
//   3 (3.00%) high mild
//   2 (2.00%) high severe
