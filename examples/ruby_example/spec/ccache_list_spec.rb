RSpec.describe CcacheList do
  class CcacheList
    def all_vals
      curr = @head
      vals = []

      while node = curr.nex
        vals << node.val
        curr = curr.nex
      end

      vals
    end
  end

  describe 'insert' do
    it 'works' do
      list = CcacheList.new
      list.append(1)
      list.append(2)
      list.append(3)

      expect(list.all_vals).to eq [1, 2, 3]
    end
  end

  describe 'delete' do
    before do
      @list = CcacheList.new
      @node0 = @list.append(1)
      @node1 = @list.append(2)
      @node2 = @list.append(3)
    end

    it 'deeltes middle' do
      @list.delete(@node1)

      expect(@list.all_vals).to eq [1, 3]
    end

    it 'deletes head' do
      @list.delete(@node0)
      expect(@list.all_vals).to eq [2, 3]
    end

    it 'deletes tail' do
      @list.delete(@node2)

      expect(@list.all_vals).to eq [1, 2]
    end
  end
end
