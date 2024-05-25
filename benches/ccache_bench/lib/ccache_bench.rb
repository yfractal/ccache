require "ccache_bench/version"
require 'active_support/time'
require 'zlib'

module CcacheBench
  class Record
    attr_accessor :key, :val

    def initialize(key, val)
      @key, @val = key, val
    end
  end

  class << self
    def generate_records(records_count)
      records = []
      records_count.times do
        key = random_string(8)
        val = random_string(8)
        records << Record.new(key, val)
      end

      records
    end

    # CcacheBench.benchmark_records(1000)
    #  500 ---> 0.49 ms
    # 1000 ---> 0.76 ms
    # 2000 ---> 1.27 ms
    # 3000 ---> 1.92 ms
    # 4000 ---> 2.44 ms
    # 5000 ---> 3.02 ms
    def benchmark_records(count)
      records = Array.new(count) do
        key = random_string(32)
        val = random_string(32)

        Record.new(key, val)
      end

      serialized = serialize(records)
      # Zlib::Inflate.inflate(serialized)

      ms = benchmark2(100, serialized) / 100 * 1000

      puts "#{ms} ms"
    end

    def benchmark2(n, value)
      start_time = Time.now

      n.times do
        Marshal.load(Zlib::Inflate.inflate(value))
      end

      end_time = Time.now

      end_time - start_time
    end

    # CcacheBench.benchmark_time_zone(1000)
    def benchmark_time_zone(count)
      Time.zone = 'Eastern Time (US & Canada)'
      times = Array.new(count) {|_| Time.zone.now }
      ms = benchmark(100, times) / 100 * 1000

      puts "#{ms} ms"
    end

    # CcacheBench.benchmark_string(10)
    # CcacheBench.benchmark_string(100)
    # CcacheBench.benchmark_string(1000)
    # CcacheBench.benchmark_string(10000)
    def benchmark_string(length)
      str = random_string(length)

      ms = benchmark(10, str) / 10 * 1000
      puts "#{ms} ms"
    end

    def benchmark(n, obj)
      start_time = Time.now

      n.times do
        serialize(obj)
      end

      end_time = Time.now

      end_time - start_time
    end

    private
    def serialize(obj)
      Zlib::Deflate.deflate(Marshal.dump(obj))
      # Marshal.dump(obj)
    end

    def random_string(len)
      (0...len).map { (65 + rand(26)).chr }.join
    end
  end

  class Error < StandardError; end
  # Your code goes here...
end
