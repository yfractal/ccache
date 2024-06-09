# frozen_string_literal: true

RSpec.describe CcacheRb do
  def memory_usage
    _, size = `ps ax -o pid,rss | grep -E "^[[:space:]]*#{$$}"`.strip.split.map(&:to_i)

    size
  end

  let(:ruby_store) {
    RubyStore.new('redis://127.0.0.1/')
  }

  describe 'RubyStore' do
    class Foo
      attr_reader :a, :b

      def initialize(a, b)
        @a, @b = a, b
      end
    end

    describe 'get' do
      it 'gets unexist key' do
        rv = ruby_store.get('unexist-key')
        expect(rv).to eq nil
      end

      it 'gets value from redis' do
        store = RubyStore.new('redis://127.0.0.1/')
        store.insert('one-key', 123456)
  
        rv = ruby_store.get('one-key')
        expect(rv).to eq 123456
      end

      it 'key can be integer' do
        rv = ruby_store.get(1)
        expect(rv).to eq nil
      end
    end

    describe 'insert' do
      it 'insert returns etag' do
        etag = ruby_store.insert('some-key', true)
        expect(etag).not_to eq nil
        expect(etag.to_i).not_to eq 0
      end
    end

    describe 'works for different type value' do
      describe 'number' do
        it 'works for number' do
          ruby_store.insert('number-key', 0)
          fetched = ruby_store.get('number-key')
          expect(fetched).to eq(0)

          ruby_store.insert('number-key', -1)
          fetched = ruby_store.get('number-key')
          expect(fetched).to eq(-1)

          ruby_store.insert('number-key', -1.00001)
          fetched = ruby_store.get('number-key')
          expect(fetched).to eq(-1.00001)
        end
      end

      describe 'string' do
        it 'works for empty string' do
          ruby_store.insert('str-key', '')
          fetched = ruby_store.get('str-key')
  
          expect(fetched).to eq('')
        end

        it 'works for empty string' do
          ruby_store.insert('str-key', 'abc')
          fetched = ruby_store.get('str-key')
  
          expect(fetched).to eq('abc')
        end
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

      describe 'works for defined class' do
        it 'simpl class' do
          foo = Foo.new(1, "b")
          ruby_store.insert('some-key', foo)
          fetched = ruby_store.get('some-key')
    
          expect(fetched.a).to eq 1
          expect(fetched.b).to eq 'b'
          expect(fetched).to eq foo
        end
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
    def insert_random(ruby_store)
      ruby_store.insert("key", Foo.new(rand, rand))
    end

    it "inserted value should not be garbage collected" do
      ruby_store.insert("key", Foo.new(1, 2))

      sleep 0.1
      GC.start # trigger gc manually

      val = ruby_store.get("key")

      expect(val.class).to eq Foo
      expect(val.a).to eq 1
      expect(val.b).to eq 2
    end

    it 'Ruby should collect unreferenced objects' do
      skip "TODO: when ruby_store has been reclaimed, should clear Rust struct too"

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

      expect(after - before).to be < 1000
      expect(memory_after - memory_before).to be < 1000
    end
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
