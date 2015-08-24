#
# Copyright 2015 YOUR NAME
#
# All Rights Reserved.
#

name "delivery-cli"
maintainer "Chef Software, Inc."
homepage "http://chef.io"

# Defaults to C:\chef\delivery-cli on Windows
# and /opt/delivery-cli on all other platforms
if windows?
  install_dir "#{default_root}/chef/#{name}"
else
  install_dir "#{default_root}/#{name}"
end

build_version do
  source :git, from_dependency: "delivery-cli"
  output_format :semver
end

build_iteration 1

# Creates required build directories
dependency "preparation"

# delivery-cli dependencies/components
dependency "delivery-cli"

# Version manifest file
dependency "version-manifest"

exclude "**/.git"
exclude "**/bundler/git"

package :msi do
  upgrade_code "178C5A9A-3923-4A65-AECB-3851224D0FDD"
  bundle_msi true
  wix_light_extension 'WixBalExtension'
  wix_candle_extension 'WixBalExtension'
end
