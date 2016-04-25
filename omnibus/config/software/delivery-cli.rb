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

license :project_license

source path: File.expand_path('..', Omnibus::Config.project_root),
       options: {exclude: [".git", "omnibus", "target", "vendor"]}

dependency "openssl"
dependency "rust"

build do
  env = with_standard_compiler_flags(with_embedded_path)

  # The rust core libraries are dynamicaly linked
  if linux?
    env['LD_LIBRARY_PATH'] = "#{install_dir}/embedded/lib"
  elsif mac_os_x?
    env['DYLD_FALLBACK_LIBRARY_PATH'] = "#{install_dir}/embedded/lib:"
  end

  # pass version info into the build
  env['DELIV_CLI_VERSION'] = project.build_version
  env['DELIV_CLI_GIT_SHA'] = project.build_git_revision

  if windows?
    copy "#{install_dir}/embedded/bin/ssleay32.dll", "#{install_dir}/embedded/bin/libssl32.dll"
    env["OPENSSL_LIB_DIR"] = "#{install_dir}/embedded/bin"
  end

  command "cargo build -j #{workers} --release", env: env

  mkdir "#{install_dir}/bin"

  if windows?
    copy "#{project_dir}/target/release/delivery.exe", "#{install_dir}/bin/delivery.exe"
    # When using `openssl` dependency, by default it builds the libraries inside
    # `embedded/bin/`. We are copying the libs inside `bin/`.
    copy "#{install_dir}/embedded/bin/ssleay32.dll", "#{install_dir}/bin/ssleay32.dll"
    copy "#{install_dir}/embedded/bin/libeay32.dll", "#{install_dir}/bin/libeay32.dll"
    copy "#{install_dir}/embedded/bin/zlib1.dll", "#{install_dir}/bin/zlib1.dll"
  else
    copy "#{project_dir}/target/release/delivery", "#{install_dir}/bin/delivery"
  end
end
