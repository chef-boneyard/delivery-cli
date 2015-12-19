#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

include_recipe 'chef-sugar::default'

#########################################################################
# Prepare builder with minimum Engineering Services standards like
# package signing.
#########################################################################

include_recipe 'opscode-ci::delivery_builder'

#########################################################################
# Install Ruby and Rust for verify stage testing
#########################################################################
ruby_install node['delivery_rust']['ruby_version']

rust_install node['delivery_rust']['rust_version'] do
  channel 'nightly'
end

# TODO: make this recipe go away
include_recipe "delivery_rust::_openssl"

#########################################################################
# Unix only: Make a backup so that if the build fails, we can rescue
# ourselves.
#########################################################################
unless windows?

  # Ensure `dbuild` can create new directories in `/opt`..namely the
  # `/opt/delivery-cli` directory. This is important because the
  # `omnibus_build` resource deletes and recreates the `/opt/delivery-cli`
  # direcotry every time it's executed.
  directory '/opt' do
    owner 'dbuild'
  end

  directory "/opt/delivery-cli" do
    owner 'dbuild'
  end

  execute "chown -R dbuild /opt/delivery-cli" do
    only_if "test -d /opt/delivery-cli"
  end

  execute "rsync -aP --delete /opt/delivery-cli/ /opt/delivery-cli-safe" do
    only_if "test -f /opt/delivery-cli/bin/delivery"
  end
end
