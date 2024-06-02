RSpec.describe RubyExample do
  describe 'RubyStore' do
    let(:ruby_store) {
      RubyStore.new("redis://127.0.0.1/")
    }

    class Foo
      attr_reader :a, :b

      def initialize(a, b)
        @a, @b = a, b
      end
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
        skip

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
        skip

        ruby_store.insert('some-key', [])
        fetched = ruby_store.get('some-key')

        expect(fetched).to eq []
      end

      it 'simple array' do
        skip

        ruby_store.insert('some-key', [''])
        fetched = ruby_store.get('some-key')

        expect(fetched).to eq ['']
      end
    end
  end
end
