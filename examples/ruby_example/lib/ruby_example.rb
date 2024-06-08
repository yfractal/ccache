require "ruby_example/version"
require 'rutie'

class CcacheRedisError < StandardError; end

module RubyExample
  # Your code goes here...
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
end
