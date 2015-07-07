require 'chef/handler'
require 'mixlib/shellout'

class OmnibusErrorHandler < Chef::Handler

  def report
    Chef::Log.error("Rolling back to previous delivery-cli")
    cmd = "rsync -aP /opt/delivery-cli-safe/ /opt/delivery-cli"
    so = Mixlib::ShellOut.new(cmd)
    so.run_command
    if so.error?
      Chef::Log.error("ROLLBACK FAILED")
      Chef::Log.error(so.stdout)
    end
  end
end

Chef::Config.exception_handlers << OmnibusErrorHandler.new()

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

## Upload package to internal artifactory
# Right now we are only building ubuntu since that is what our builders are.
# We will likely introduce other platforms in the future.
[ "12.04", "14.04" ].each do |pv|
  delivery_rust_artifactory "delivery-cli" do
    package_path ::File.join(omnibus_path, "pkg", "*.deb")
    repository 'omnibus-current-local'
    platform 'ubuntu'
    platform_version pv
    endpoint 'http://artifactory.chef.co/'
    base_path 'com/getchef'
    username 'delivery'
    password secrets['artifactory_password']
    sensitive true
  end
end
