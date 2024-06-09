module ReferenceKeeper
  # TODO: handle concurrency
  def keep(key, ref)
    @key_to_node ||= {}
    @ref_list ||= CcacheList.new

    node = @ref_list.append(ref)
    @key_to_node[key] = node
  end

  def drop(key)
    node = @key_to_node[key]
    @key_to_node.delete(key)
    @ref_list.delete(node)
  end
end

class RubyStore
  include ReferenceKeeper

  def get(key)
    rs_get(key.to_s)
  end

  def insert(key, val)
    rs_insert(key, val)
  end
end
