require 'chef/handler'
require 'mixlib/shellout'

class OmnibusHandler < Chef::Handler

  def report
    Chef::Log.info("Rolling back to previous delivery-cli")
    cmd = "rsync -aP /opt/delivery-cli-safe/ /opt/delivery-cli"
    so = Mixlib::ShellOut.new(cmd)
    so.run_command
    if so.error?
      Chef::Log.error("ROLLBACK FAILED")
      Chef::Log.error(so.stdout)
    end
  end
end

Chef::Config.exception_handlers << OmnibusHandler.new()
Chef::Config.report_handlers << OmnibusHandler.new()

git_ssh = File.join('/var/opt/delivery/workspace/bin', 'git_ssh')
omnibus_path = File.join(node['delivery']['workspace']['repo'], 'omnibus-delivery-cli')

## Make sure it builds!
execute "bundle install --binstubs=#{omnibus_path}/bin --path=#{File.join(node['delivery_builder']['cache'], 'gems')}" do
  cwd omnibus_path
end

execute "#{omnibus_path}/bin/omnibus build delivery-cli" do
  cwd omnibus_path
end

## Push it to Github
git_ssh = File.join(node['delivery']['workspace']['cache'], 'git_ssh')
deploy_key = File.join(node['delivery']['workspace']['cache'], 'github.pem')
secrets = get_project_secrets

file deploy_key do
  content secrets['github']
  owner 'dbuild'
  mode '0600'
  sensitive true
end

template git_ssh do
  source 'git_ssh.erb'
  owner 'dbuild'
  mode '0755'
end

execute "set_git_username" do
  command "git config user.name 'Delivery'"
  cwd node['delivery']['workspace']['repo']
  environment({"GIT_SSH" => git_ssh})
end

execute "set_git_email" do
  command "git config user.email 'delivery@chef.io'"
  cwd node['delivery']['workspace']['repo']
  environment({"GIT_SSH" => git_ssh})
end

execute "add_github_remote" do
  command "git remote add github git@github.com:chef/delivery-cli.git"
  cwd node['delivery']['workspace']['repo']
  environment({"GIT_SSH" => git_ssh})
  not_if "git remote --verbose | grep ^github"
end

execute "push_to_github" do
  command "git push github master"
  cwd node['delivery']['workspace']['repo']
  environment({"GIT_SSH" => git_ssh})
end

##############################################################

pkg_dir = ::File.join(omnibus_path, 'pkg')

endpoint       = 'http://artifactory.chef.co/'
repository     = 'omnibus-current-local'
base_path      = 'com/getchef'
pattern        = "#{pkg_dir}/*.{deb,rpm}"
username       = 'delivery'
password       = secrets['artifactory_password']

omnibus_config         = ::File.join(node['delivery']['workspace']['cache'], 'omnibus-publish.rb')
platform_mappings_path = ::File.join(node['delivery']['workspace']['cache'], 'omnibus-platform-mappings.json')


# Build on one platform, but can install on several
file platform_mappings_path do
  content <<-EOH
{
  "ubuntu-14.04": [
    "ubuntu-12.04",
    "ubuntu-14.04"
  ],
  "el-6": [
    "el-6",
    "el-7"
  ]
}
EOH
  mode '0600'
  owner 'dbuild'
  group 'dbuild'
end

# Render an Omnibus config file for publishing
file omnibus_config do
  content <<-EOH
# This file is written by Chef for #{node['fqdn']}.
# Do NOT modify this file by hand.
artifactory_endpoint  '#{endpoint}'
artifactory_base_path '#{base_path}'
artifactory_username  '#{username}'
artifactory_password  '#{password}'
  EOH
  mode '0600'
  owner 'dbuild'
  group 'dbuild'
  sensitive true
end

# Use the CLI because RUBY INCEPTION
execute "upload to artifactory" do
  command "#{omnibus_path}/bin/omnibus publish artifactory #{repository} #{pattern} " \
          "--config #{omnibus_config} " \
          "--platform-mappings #{platform_mappings_path} " \
          "--version-manifest #{pkg_dir}/version-manifest.json"
  cwd omnibus_path
end
