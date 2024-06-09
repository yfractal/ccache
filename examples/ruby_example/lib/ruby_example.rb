# frozen_string_literal: true

require 'ruby_example/version'
require 'rutie'
require 'ccache_list'

class CcacheRedisError < StandardError; end

module RubyExample
  Rutie.new(:ruby_example).init 'Init_ruby_example', __dir__
end

class RubyStore
  def get(key)
    rs_get(key.to_s)
  end

  def insert(key, val)
    @val = val
    rs_insert(key, val)
  end

  def keep_val(val)
  end
end
