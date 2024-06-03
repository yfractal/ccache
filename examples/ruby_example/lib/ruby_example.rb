require "ruby_example/version"
require 'rutie'

class CcacheRedisError < StandardError; end

module RubyExample
  # Your code goes here...
  Rutie.new(:ruby_example).init 'Init_ruby_example', __dir__
end
