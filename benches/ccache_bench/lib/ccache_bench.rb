require 'ccache_bench/version'

require 'cpu_time'
require 'objspace'
require 'redis'
require 'securerandom'
require 'zlib'

module CcacheBench
  class Benchmarker
    Record = Struct.new(:id, :val, :val2)
    def initialize(redis_url, records_count, repeat_times = 100)
      @redis_url     = redis_url
      @records_count = records_count
      @repeat_times  = repeat_times
      @redis_key     = "CcacheBench::Benchmarker::key"
    end

    def insert_to_redis
      records = Array.new(@records_count) do
        Record.new(rand(10000000), random_string(32), random_string(32))
      end

      serialized = serialize(records)
      redis = Redis.new(url: @redis_url)
      redis.set(@redis_key, serialized)

      size = (4 + 32 + 32) * @records_count

      serialized_size = serialized.length
      redis_memory_usage = redis.memory('usage', @redis_key)

      puts "Records count:#{@records_count}, size: #{size}, serialized_size: #{serialized_size}, redis_memory_usage=#{redis_memory_usage}"
    end

    def benchmark
      redis_baseline

      time = Array.new(@repeat_times) { read_from_redis }.sum / @repeat_times * 1000
      puts "Read #{@records_count} records takes #{time} ms."

      time
    end

    def read_from_redis
      redis = Redis.new(:url => @redis_url)
      redis.get(1)

      t0 = cpu_time
      records = do_read_from_redis(redis)
      t1 = cpu_time

      raise "records is nill" unless records

      t1 - t0
    end

    def redis_baseline
      redis = Redis.new(:url => @redis_url)
      redis.get 1
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
      SecureRandom.uuid
      # (0...len).map { (65 + rand(26)).chr }.join
    end
  end
end
