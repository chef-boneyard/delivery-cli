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
  command "bash #{Chef::Config[:file_cache_path]}/rustup.sh"
end

node.set['omnibus']['build_user'] = "dbuild"
node.set['omnibus']['build_user_group'] = "dbuild"
include_recipe "omnibus"

directory "/opt/delivery-cli" do
  owner 'dbuild'
end
