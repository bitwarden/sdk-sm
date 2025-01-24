# frozen_string_literal: true

require_relative 'lib/version'

Gem::Specification.new do |spec|
  spec.name = 'bitwarden-sdk-secrets'
  spec.version = BitwardenSDKSecrets::VERSION
  spec.authors = ['Bitwarden Inc.']
  spec.email = ['hello@bitwarden_sdk.com']

  spec.summary = 'Bitwarden Secrets Manager SDK.'
  spec.description = 'Ruby wrapper for Bitwarden secrets manager SDK.'
  spec.homepage = 'https://bitwarden.com/products/secrets-manager/'
  spec.required_ruby_version = '>= 3.0.0'

  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = 'https://github.com/bitwarden/sdk-sm'
  spec.metadata['changelog_uri'] = 'https://github.com/bitwarden/sdk-sm/blob/main/languages/ruby/CHANGELOG.md'

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(__dir__) do
    `git ls-files -z`.split("\x0").reject do |f|
      (File.expand_path(f) == __FILE__) || f.start_with?(*%w[bin/ test/ spec/ features/ .git Gemfile])
    end
  end

  spec.files += Dir.glob('lib/linux-x64/**/*')
  spec.files += Dir.glob('lib/macos-x64/**/*')
  spec.files += Dir.glob('lib/windows-x64/**/*')
  spec.files += Dir.glob('lib/macos-arm64/**/*')
  spec.files += Dir.glob('lib/schemas.rb')

  spec.bindir = 'exe'
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']

  # Uncomment to register a new dependency of your gem
  # spec.add_dependency "example-gem", "~> 1.0"
  spec.add_dependency 'dry-struct', '~> 1.6'
  spec.add_dependency 'dry-types', '~> 1.7'
  spec.add_dependency 'ffi', '~> 1.15'
  spec.add_dependency 'json', '~> 2.6'
  spec.add_dependency 'rake', '~> 13.0'
  spec.add_dependency 'rubocop', '~> 1.21'

end
