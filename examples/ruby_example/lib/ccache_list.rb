# frozen_string_literal: true

class CcacheList
  class Node
    attr_accessor :pre, :nex, :val

    def initialize(val)
      @val = val
    end

    def link(item)
      self.nex = item
      item.pre = self
    end
  end

  def initialize
    @head = Node.new(nil)
    @tail = @head
  end

  def append(val)
    node = Node.new(val)
    @tail.link(node)
    @tail = @tail.nex

    node
  end

  def delete(node)
    node.pre.nex = node.nex
    node.nex.pre = node.pre if node.nex # not tail
  end
end
