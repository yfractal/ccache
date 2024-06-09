# frozen_string_literal: true

require 'rutie'
require 'ccache_list'
require 'ruby_store'
require_relative "ccache_rb/version"

class CcacheRedisError < StandardError; end

module CcacheRb
  Rutie.new(:ruby_example).init 'Init_ccache_rb', __dir__
end
