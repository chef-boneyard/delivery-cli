#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#
include_recipe 'chef-sugar::default'

include_recipe "delivery_rust::_prep_builder"


if windows?
  windows_package "rust" do
    source "https://static.rust-lang.org/dist/#{node['delivery_rust']['rust_version']}/rust-nightly-x86_64-pc-windows-gnu.msi"
  end
else
  rust_version = node['delivery_rust']['rust_version']

  rust_install "Install #{rust_version}" do
    channel "nightly"
    version rust_version
    prefix "/usr/local"
  end
end

include_recipe "delivery_rust::_openssl"
