require "ccache_bench/version"
require 'zlib'
require 'redis'

module CcacheBench
  class Benchmarker
    def initialize(redis_url, records_count, repeat_times = 100)
      @redis_url     = redis_url
      @records_count = records_count
      @repeat_times  = repeat_times
      @redis_key     = "CcacheBench::Benchmarker::key"
    end

    def insert_to_redis
      records = Array.new(@records_count) { [rand(10000000), random_string(32), random_string(32)] }

      serialized = serialize(records)

      redis = Redis.new(url: @redis_url)
      redis.set(@redis_key, serialized)
    end

    def benchmark
      benchmark_redis

      time = Array.new(@repeat_times) { read_from_redis }.sum / @repeat_times * 1000
      puts "Read #{@records_count} records takes #{time} ms."

      time
    end

    def read_from_redis
      redis = Redis.new(:url => @redis_url)
      t0 = Time.now
      records = do_read_from_redis(redis)
      t1 = Time.now

      raise "records is nill" unless records

      t1 - t0
    end

    def benchmark_redis
      redis = Redis.new(:url => @redis_url)
      time = Array.new(@repeat_times) { do_benchmark_redis(redis) }.sum / @repeat_times * 1000

      puts "Redis empty key takes #{time} ms."
    end

    def delete
      redis = Redis.new(:url => @redis_url)

      redis.del(@redis_key)
    end

    private
    def do_benchmark_redis(redis)
      t0 = Time.now
      result = redis.get("unexist_key")
      t1 = Time.now

      raise "should be nil" if result

      t1 - t0
    end

    def do_read_from_redis(redis)
      val = redis.get(@redis_key)

      inflated = Zlib::Inflate.inflate(val)
      Marshal.load(inflated)
    end


    def serialize(obj)
      serialized = Marshal.dump(obj)

      Zlib::Deflate.deflate(serialized)
    end

    def random_string(len)
      (0...len).map { (65 + rand(26)).chr }.join
    end
  end
end
