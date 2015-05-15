#
# Cookbook Name:: delivery_rust
# Recipe:: default
#
# Copyright (C) Chef Software, Inc. 2014
#

remote_file "#{Chef::Config[:file_cache_path]}/rustup.sh" do
  source "https://static.rust-lang.org/rustup.sh"
end

rust_version = node['delivery_rust']['rust_version']
rustup_cmd = ["bash",
              "#{Chef::Config[:file_cache_path]}/rustup.sh",
              "--channel=nightly",
              "--date=#{rust_version}",
              "--yes"].join(' ')
execute "install rust and cargo" do
  command rustup_cmd
end

node.set['omnibus']['build_user'] = "dbuild"
node.set['omnibus']['build_user_group'] = "dbuild"
node.set['omnibus']['build_user_home'] = '/var/opt/delivery/workspace'

include_recipe "omnibus"

# The omnibus cookbook will try to take over the dbuild user/group. This is
# causing unhappiness with the overall pipeline. The follow code will disable
# those resources so they don't cause the unhappiness.
u = resources(user: 'dbuild')
u.action :nothing

g = resources(group: 'dbuild')
g.action :nothing

d = resources(directory: '/var/opt/delivery/workspace')
d.action :nothing

directory "/opt/delivery-cli" do
  owner 'dbuild'
end

# Make sure all the files are owned by us - keeps us safe after package upgrades
execute "chown -R dbuild /opt/delivery-cli"

# Make a backup so that if the build fails, we can rescue ourselves
execute "rsync -aP --delete /opt/delivery-cli/ /opt/delivery-cli-safe"
