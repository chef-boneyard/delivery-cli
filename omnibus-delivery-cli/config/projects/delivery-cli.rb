#
# Copyright 2015 YOUR NAME
#
# All Rights Reserved.
#

name "delivery-cli"
maintainer "Chef Software, Inc."
homepage "http://chef.io"

# Defaults to C:/delivery-cli on Windows
# and /opt/delivery-cli on all other platforms
install_dir "#{default_root}/#{name}"

build_version Time.now.utc.strftime("%Y%m%d%H%M%S")
build_iteration 1

# Creates required build directories
dependency "preparation"

# delivery-cli dependencies/components
dependency "delivery-cli"

# Version manifest file
dependency "version-manifest"

exclude "**/.git"
exclude "**/bundler/git"

