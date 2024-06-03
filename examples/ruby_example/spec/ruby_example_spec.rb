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

  # TODO
  # ruby_store.get 1 should work :joy:
end
