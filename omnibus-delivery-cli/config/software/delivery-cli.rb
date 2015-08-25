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

dependency "delivery-windows-openssl" if windows?
dependency "vcredist" if windows?

build do
  # Setup a default environment from Omnibus - you should use this Omnibus
  # helper everywhere. It will become the default in the future.
  env = if windows?
          ssldir = File.join(Omnibus::Config.cache_dir, "openssl-cache")
          tmp = with_embedded_path.merge("OPENSSL_LIB_DIR" => "#{ssldir}", "OPENSSL_INCLUDE_DIR" => "#{ssldir}/include")
          with_standard_compiler_flags(tmp)
        else
          with_standard_compiler_flags(with_embedded_path)
        end

  command "make build", env: env

  mkdir "#{install_dir}/bin"
  if windows?
    copy "#{project_dir}/target/release/delivery.exe", "#{install_dir}/bin/delivery.exe"
    copy "#{project_dir}/target/release/libdelivery.rlib", "#{install_dir}/bin/libdelivery.rlib"
  else
    copy "#{project_dir}/target/release/delivery", "#{install_dir}/bin/delivery"
  end
end
