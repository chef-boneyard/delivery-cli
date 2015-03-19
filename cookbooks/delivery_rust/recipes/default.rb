#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

remote_file "#{Chef::Config[:file_cache_path]}/rustup.sh" do
  source "https://static.rust-lang.org/rustup.sh"
end

execute "install rust and cargo" do
  command "bash #{Chef::Config[:file_cache_path]}/rustup.sh --date=2015-03-16"
end

node.set['omnibus']['build_user'] = "dbuild"
node.set['omnibus']['build_user_group'] = "dbuild"
include_recipe "omnibus"

directory "/opt/delivery-cli" do
  owner 'dbuild'
end

chef_gem "omnibus" do
#  compile_time false if Chef::Resource::ChefGem.method_defined?(:compile_time)
end

chef_gem "artifactory" do
#  compile_time false if Chef::Resource::ChefGem.method_defined?(:compile_time)
end
