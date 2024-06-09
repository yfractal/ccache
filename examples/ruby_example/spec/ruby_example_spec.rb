RSpec.describe RubyExample do
  let(:ruby_store) {
    RubyStore.new("redis://127.0.0.1/")
  }

  describe 'RubyStore' do
    class Foo
      attr_reader :a, :b

      def initialize(a, b)
        @a, @b = a, b
      end
    end

    it 'none exist key' do
      rv = ruby_store.get('none-exist-key')
      expect(rv).to eq nil
    end

    it 'insert returns etag' do
      etag = ruby_store.insert('some-key', true)
      expect(etag).not_to eq nil
      expect(etag.to_i).not_to eq 0
    end

    it 'get value from redis' do
      store = RubyStore.new('redis://127.0.0.1/')
      store.insert('one-key', 123456)

      rv = ruby_store.get('one-key')
      expect(rv).to eq 123456
    end

    it 'gets return nil' do
      rv = ruby_store.get('none-exist-key')
      expect(rv).to eq nil
    end

    it 'get works for integer' do
      rv = ruby_store.get(1)
      expect(rv).to eq nil
    end

    it 'works for simple obj' do
      ruby_store.insert('some-key', true)
      expect(ruby_store.get('some-key')).to eq true
    end

    it 'works for obj' do
      foo = Foo.new(1, "b")
      ruby_store.insert('some-key', foo)
      fetched = ruby_store.get('some-key')

      expect(fetched.a).to eq 1
      expect(fetched.b).to eq 'b'
      expect(fetched).to eq foo
    end

    describe 'hash' do
      it 'empty hash' do
        ruby_store.insert('some-key', {})
        fetched = ruby_store.get('some-key')

        expect(fetched).to eq({})
      end

      it 'simple hash' do
        ruby_store.insert('some-key', {:a => 1})
        fetched = ruby_store.get('some-key')

        expect(fetched).to eq({:a => 1})
      end
    end

    describe 'array' do
      it 'empaty array' do
        ruby_store.insert('some-key', [])
        fetched = ruby_store.get('some-key')

        expect(fetched).to eq []
      end

      it 'simple array' do
        ruby_store.insert('some-key', ['a'])
        fetched = ruby_store.get('some-key')

        expect(fetched).to eq ['a']
      end
    end
  end

  describe 'exception handling' do
    it 'should raise redis exception' do
      RubyStore.new("redis://-1.-1.-1.-1")
    rescue => e
      expect(e.class).to eq CcacheRedisError
    end
  end

  describe "GC" do
    def insert
      ruby_store.insert("key", Foo.new(1, 2))
    end

    it "inserted value should not be garbage collected" do
      insert

      GC.start # trigger gc manually

      val = ruby_store.get("key")

      expect(val.class).to eq Foo
      expect(val.a).to eq 1
      expect(val.b).to eq 2
    end
  end

  describe "GC example" do
    def memory_usage
      _, size = `ps ax -o pid,rss | grep -E "^[[:space:]]*#{$$}"`.strip.split.map(&:to_i)

      size
    end

    def insert
      ruby_store.insert("key", Foo.new(1, 2))
    end

    def insert_random(ruby_store)
      ruby_store.insert("key", Foo.new(rand, rand))
    end

    it 'inserted value should not be garbage collected' do
      insert

      GC.start # trigger gc manually

      val = ruby_store.get("key")

      expect(val.class).to eq Foo
      expect(val.a).to eq 1
      expect(val.b).to eq 2
    end

    it 'Ruby should collect unreferenced objects' do
      skip "when ruby_store has been reclaimed, it should clear Rust struct"

      total = 100_000
      ruby_store

      before = ObjectSpace.count_objects[:TOTAL]
      memory_before = memory_usage
      ruby_store = RubyStore.new("redis://127.0.0.1/")

      total.times { insert_random(ruby_store) }

      ruby_store = nil
      sleep 0.1

      GC.start

      after = ObjectSpace.count_objects[:TOTAL]
      memory_after = memory_usage

      expect(after - before).to be < 100_000 / 2
      expect(memory_after - memory_before).to be < 20_000
    end

    describe 'Ruby GC feature tests' do
      it 'does not reclaim objects when they are referenced' do
        total = 10_000
        before = ObjectSpace.count_objects[:T_OBJECT]

        array = Array.new(total) { Foo.new(rand, rand) }

        GC.start

        after = ObjectSpace.count_objects[:T_OBJECT]
        expect(after - before).to be >= 9000
      end

      it 'collects unreferenced objects' do
        total = 1_000_000
        before = ObjectSpace.count_objects[:T_OBJECT]
        memory_before = memory_usage

        total.times { Foo.new(rand, rand) }

        GC.start

        after = ObjectSpace.count_objects[:T_OBJECT]
        memory_after = memory_usage

        expect(after - before).to be < 2000
        expect(memory_after - memory_before).to be < 2000
      end
    end
  end
end
