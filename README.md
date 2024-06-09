<h1 align="center">Ccache: A Redis Client-Side Caching with Strong Consistency</h1>

## Introduction
Ccache, short for Conditional Cache, works like HTTP conditional requests, providing client-side caching without sacrificing consistency.

Ccache caches data locally, and for subsequent requests, it sends the key with the cached data's ETag to Redis. If the key's data in Redis hasn't changed, Redis returns "no change" and Ccache uses the locally cached data.

![Ccache Benchmark(CPU Time)_cleanup](https://github.com/yfractal/ccache/assets/3775525/9b9ac39e-e8e3-4362-91d1-7aab3e31589e)

Many backend servers save large dynamic settings in Redis. As this data grows, the serialization time increases rapidly. Ccache can reduce such CPU waste by caching data locally, eliminating the need to deserialize data when it remains unchanged.

The image above is a benchmark of Ccache compared to the default implementation (querying Redis and deserializing). We see that Ccache significantly decreases CPU time.

## Why Ccache?
Redis offers [client-side caching](https://redis.io/docs/latest/develop/use/client-side-caching/) to reduce latency and Redis load.

However, it only guarantees weak consistency. For example, when clients request different backend servers, they can receive inconsistent data.

Ccache offers strong consistency guarantees by co-designing the client-server interaction through the ETag likes mechanism.

## Multiple Languages Support
- Rust: [ccache](./)
- Ruby: [ccache_rb](./ccache_rb/)
- Golang: in progress

## Use Case

Sometimes, applications cache large-sized data in Redis, which causes significant latency due to serialization and deserialization.

For example, [rails-settings-cached](https://github.com/huacnlee/rails-settings-cached) is a very popular gem used by almost every Rails application, it caches the entire settings table in Redis and fetches the data for every request.

As the application accumulates more settings, the data size increases rapidly. If an application has several hundred settings, the data size could be several MB, requiring over 10ms to retrieve this data. This not only significantly increases delay but also makes Redis bandwidth a bottleneck.

Redis' default client-side caching doesn't fit in this case, as its weak consistency guarantees can cause users to see inconsistent settings between requests.

Ccache addresses these issues by using an ETag-like mechanism to provide strong consistency. Although it slightly increases the Redis load compared to Redis' default client-side caching, the strong consistency it offers is a worthwhile trade-off.
