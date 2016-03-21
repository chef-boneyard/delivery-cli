#
# Copyright 2015 YOUR NAME
#
# All Rights Reserved.
#

name "delivery-cli"
friendly_name "Delivery CLI"
maintainer "Chef Software, Inc."
homepage "http://chef.io"

# Defaults to C:\chef\delivery-cli on Windows
# and /opt/delivery-cli on all other platforms
if windows?
  install_dir "#{default_root}/chef/#{name}"
else
  install_dir "#{default_root}/#{name}"
end

# Sadly, Wix on Windows doesn't allow the use of Int
# bigged that 32bits, so we need to change the version
# to use the standard semantic version (`M.M.P`)
#
# TODO: When we change the delivery-cli to use this
# version pattern, we will need to get rid of this code
#
# Until then :herewego:
if windows?
  build_version '0.0.1'
else
  build_version Time.now.utc.strftime("%Y%m%d%H%M%S")
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

package :rpm do
  signing_passphrase ENV['OMNIBUS_RPM_SIGNING_PASSPHRASE']
end

package :pkg do
  identifier "io.chef.pkg.delivery-cli"
  signing_identity "Developer ID Installer: Chef Software, Inc. (EU3VF8YLX2)"
end
compress :dmg

package :msi do
  upgrade_code "178C5A9A-3923-4A65-AECB-3851224D0FDD"
  wix_candle_extension 'WixUtilExtension'
  # TODO: Once we have Windows build-nodes ready to build the cli
  # we need to uncomment this line so we can sighn our MSI packages
  #signing_identity "F74E1A68005E8A9C465C3D2FF7B41F3988F0EA09", machine_store: true
end
