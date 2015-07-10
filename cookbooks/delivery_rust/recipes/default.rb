#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

include_recipe "delivery_rust::_prep_builder"


cache_dir = Chef::Config[:file_cache_path]

remote_file "#{cache_dir}/rustup.sh" do
  source "https://static.rust-lang.org/rustup.sh"
end

rust_version = node['delivery_rust']['rust_version']
rustup_cmd = ["bash",
              "#{cache_dir}/rustup.sh",
              "--channel=nightly",
              "--date=#{rust_version}",
              "--yes"].join(' ')

rustup_cmd << " --disable-sudo" if platform_family?('mac_os_x')

execute "install rust and cargo" do
  command rustup_cmd
  not_if { rust_version == current_rust_version }
end

include_recipe "delivery_rust::_openssl"
