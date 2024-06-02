lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "ruby_example/version"

Gem::Specification.new do |spec|
  spec.name          = "ruby_example"
  spec.version       = RubyExample::VERSION
  spec.authors       = ["Mike Yang"]
  spec.email         = ["yfractal@gmail.com"]

  spec.summary       = %q{A simple Ruby demo}
  spec.description   = %q{A simple Ruby demo}

  spec.license       = "MIT"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files         = Dir.chdir(File.expand_path('..', __FILE__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.add_dependency 'rutie', '~> 0.0.3'

  spec.add_development_dependency "bundler", "~> 2.5.10"
  spec.add_development_dependency "rake", "~> 10.0"
  spec.add_development_dependency "rspec", "~> 3.0"
  spec.add_development_dependency "byebug"
end
