require "ruby_example/version"
require 'rutie'

module RubyExample
  class Error < StandardError; end
  # Your code goes here...
  Rutie.new(:ruby_example).init 'Init_ruby_example', __dir__
end
