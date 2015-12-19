#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

#########################################################################
# Prepare builder with minimum Engineering Services standards like
# package signing.
#########################################################################

include_recipe 'opscode-ci::delivery_builder'

# Ensure `dbuild` can create new directories in `/opt`..namely the
# `/opt/delivery-cli` directory. This is important because the
# `omnibus_build` resource deletes and recreates the `/opt/delivery-cli`
# direcotry every time it's executed.
directory '/opt' do
  owner 'dbuild'
end

include_recipe "delivery_rust::_prep_builder"

#########################################################################
# Install Ruby and Rust for verify stage testing
#########################################################################
ruby_install node['delivery_rust']['ruby_version']

rust_install node['delivery_rust']['rust_version'] do
  channel 'nightly'
end

include_recipe "delivery_rust::_openssl"
