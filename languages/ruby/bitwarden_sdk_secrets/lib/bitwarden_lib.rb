# frozen_string_literal: true

require 'ffi'

module BitwardenSDKSecrets
  module BitwardenLib
    extend FFI::Library

    def self.mac_with_intel?
      `uname -m`.strip == 'x86_64'
    end

    ffi_lib case RUBY_PLATFORM
            when /darwin/
              local_file = if mac_with_intel?
                             File.expand_path('macos-x64/libbitwarden_c.dylib', __dir__)
                           else
                             File.expand_path('macos-arm64/libbitwarden_c.dylib', __dir__)
                           end
              File.exist?(local_file) ? local_file : File.expand_path('../../../../target/debug/libbitwarden_c.dylib', __dir__)
            when /linux/
              local_file = File.expand_path('linux-x64/libbitwarden_c.so', __dir__)
              File.exist?(local_file) ? local_file : File.expand_path('../../../../target/debug/libbitwarden_c.so', __dir__)
            when /mswin|mingw/
              local_file = File.expand_path('windows-x64/bitwarden_c.dll', __dir__)
              File.exist?(local_file) ? local_file : File.expand_path('../../../../target/debug/bitwarden_c.dll', __dir__)
            else
              raise "Unsupported platform: #{RUBY_PLATFORM}"
            end

    attach_function :init, [:string], :pointer
    attach_function :run_command, %i[string pointer], :string
    attach_function :free_mem, [:pointer], :void
    attach_function :get_access_token_organization, [:pointer], :string
  end
end
