#
# Copyright 2014 Chef Software, Inc.
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

#
# openssl 1.0.0m fixes a security vulnerability:
#   https://www.openssl.org/news/secadv_20140605.txt
# Since the rubyinstaller.org doesn't release ruby when a dependency gets
# patched, we are manually patching the dependency until we get a new
# ruby release on windows.
# This component should be removed when we upgrade to the next version of
# rubyinstaller > 1.9.3-p545 and 2.0.0-p451
#
# openssl 1.0.0n fixes more security vulnerabilities...
#   https://www.openssl.org/news/secadv_20140806.txt

# Ruby 2.0.0 doesn't support openssl 1.0.1 - so we still need to maintain 1.0.0
# series for those rubies.

name "delivery-openssl-windows"
default_version "1.0.1p"

version('1.0.1p') do
  source url: "https://github.com/jaym/windows-openssl-build/releases/download/openssl-1.0.1p/openssl-1.0.1p-x64-windows.tar.lzma",
    md5: "ffdcef3e4fd1eca457a2e9a08cdd5aa1"
end

build do
  env = with_standard_compiler_flags(with_embedded_path)

  tmpdir = File.join(Omnibus::Config.cache_dir, "openssl-cache")

  tar_filename = "openssl-#{version}-x64-windows.tar"

  # Ensure the directory exists
  mkdir tmpdir

  # First extract the tar file out of lzma archive.
  command "7z.exe x #{project_file} -o#{tmpdir} -r -y", env: env

  # Now extract the files out of tar archive.
  command "7z.exe x #{File.join(tmpdir, tar_filename)} -o#{tmpdir} -r -y", env: env

  # Copy over the required dlls into embedded/bin
  copy "#{tmpdir}/bin/libeay32.dll", "#{install_dir}/bin/"
  copy "#{tmpdir}/bin/ssleay32.dll", "#{install_dir}/bin/"
  copy "#{tmpdir}/bin/ssleay32.dll", "#{install_dir}/bin/libssl32.dll"

end
