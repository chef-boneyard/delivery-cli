#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

# Ensure the Omnibus cookbook and this build cookbook have their Ruby
# versions in sync.
node.set['omnibus']['ruby_version'] = node['delivery_rust']['ruby_version']

# The Omnibus build user should be `dbuild`
node.set['omnibus']['build_user']          = 'dbuild'
node.set['omnibus']['build_user_group']    = 'root'
node.set['omnibus']['build_user_home']     = delivery_workspace

include_recipe 'chef-sugar::default'
include_recipe 'omnibus::default'
include_recipe "delivery_rust::_prep_builder"

ruby_install node['delivery_rust']['ruby_version']

rust_install node['delivery_rust']['rust_version'] do
  channel 'nightly'
end

include_recipe "delivery_rust::_openssl"

# Ensure `dbuild` can create new directories in `/opt`..namely the
# `/opt/delivery-cli` directory. This is important because the
# `omnibus_build` resource deletes and recreates the `/opt/delivery-cli`
# direcotry every time it's executed.
directory '/opt' do
  owner 'dbuild'
end
