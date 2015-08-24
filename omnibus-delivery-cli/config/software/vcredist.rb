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

name "vcredist"
default_version "2013"

version "2008" do
  source url: "http://download.microsoft.com/download/d/2/4/d242c3fb-da5a-4542-ad66-f9661d0a8d19/vcredist_x64.exe", sha256: "baaaeddc17bcda8d20c0a82a9eb1247be06b509a820d65dda1342f4010bdb4a0", md5: "a31dc1a74f1dee5caf63aec8ebb5fe20"
end

version "2013" do
  source url: "http://download.microsoft.com/download/2/E/6/2E61CFA4-993B-4DD4-91DA-3737CD5CD6E3/vcredist_x64.exe", md5: "96b61b8e069832e6b809f24ea74567ba"
end

# we just need to stuff this in the cache
# build do
#   with_standard_compiler_flags(with_embedded_path)
# end
