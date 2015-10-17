#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#
include_recipe 'chef-sugar::default'
include_recipe 'omnibus::default'

include_recipe "delivery_rust::_prep_builder"

rust_version = node['delivery_rust']['rust_version']

rust_install "Install #{rust_version}" do
  channel 'nightly'
  version rust_version
  prefix '/usr/local' unless windows?
end

include_recipe "delivery_rust::_openssl"
