#
# Copyright 2015 YOUR NAME
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#

# These options are required for all software definitions
name "delivery-cli"

source path: File.expand_path('..', Omnibus::Config.project_root),
       options: {exclude: [".git", "omnibus-delivery-cli", "target", "vendor"]}

dependency "openssl" if windows?

build do
  # Setup a default environment from Omnibus - you should use this Omnibus
  # helper everywhere. It will become the default in the future.
  env = if windows?
          copy "#{install_dir}/embedded/bin/ssleay32.dll", "#{install_dir}/embedded/bin/libssl32.dll"
          with_standard_compiler_flags(with_embedded_path).merge(
            "OPENSSL_LIB_DIR" => "#{install_dir}/embedded/bin",
            "OPENSSL_INCLUDE_DIR" => "#{install_dir}/embedded/include"
          )
        else
          with_standard_compiler_flags(with_embedded_path)
        end

  command "make build", env: env

  mkdir "#{install_dir}/bin"
  if windows?
    copy "#{project_dir}/target/release/delivery.exe", "#{install_dir}/bin/delivery.exe"
    # When using `openssl` dependency, by default it builds the libraries inside
    # `embedded/bin/`. Once we are done building the `delivery-cli.exe` we want to
    # clean what we will package in the MSI. We are copying the libs inside `bin/`
    # and then delete what is in embedded since we dont use it.
    copy "#{install_dir}/embedded/bin/ssleay32.dll", "#{install_dir}/bin/ssleay32.dll"
    copy "#{install_dir}/embedded/bin/libeay32.dll", "#{install_dir}/bin/libeay32.dll"
    copy "#{install_dir}/embedded/bin/zlib1.dll", "#{install_dir}/bin/zlib1.dll"
    delete "#{install_dir}/embedded"
  else
    copy "#{project_dir}/target/release/delivery", "#{install_dir}/bin/delivery"
  end
end
