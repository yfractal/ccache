# frozen_string_literal: true

require 'rutie'
require 'ccache_list'
require_relative "ccache_rb/version"

class CcacheRedisError < StandardError; end

module CcacheRb
  Rutie.new(:ruby_example).init 'Init_ccache_rb', __dir__
end

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
