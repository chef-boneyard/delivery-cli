#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

include_recipe 'chef-sugar::default'

#########################################################################
# Install Ruby and Rust for verify stage testing
#########################################################################
ruby_install node['delivery_rust']['ruby_version']

rust_install node['delivery_rust']['rust_version'] do
  channel 'nightly'
end

# Install Knife-Supermarket Gem
chef_gem 'knife-supermarket' do
  version '0.2.2'
end
