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

delivery_bus_secrets = DeliverySugar::ChefServer.new.encrypted_data_bag_item('delivery-bus', 'secrets')

#########################################################################
# PUBLISH TO GITHUB
#########################################################################

delivery_github 'Push delivery-cli to GitHub' do
  repo_path delivery_workspace_repo
  cache_path delivery_workspace_cache
  deploy_key delivery_bus_secrets['github_private_key'] # chef-delivery's key
  remote_name 'github'
  remote_url 'git@github.com:chef/delivery-cli.git'
end

#########################################################################
# BUILD
#########################################################################

omnibus_project_dir = File.join(delivery_workspace_repo, 'omnibus-delivery-cli')
omnibus_base_dir    = File.join(delivery_workspace_cache, 'omnibus')

omnibus_build 'delivery-cli' do
  base_dir omnibus_base_dir
  project_dir omnibus_project_dir
  build_user 'dbuild' # TODO: expose this in delivery-sugar DSL
  log_level :internal
  config_overrides(
    base_dir: omnibus_base_dir,
    append_timestamp: true
  )
end

#########################################################################
# PUBLISH TO ARTIFACTORY
#########################################################################

# TODO: set these things in `delivery-bus`
node.set['artifactory-pro']['endpoint']      = 'http://artifactory.chef.co:8081'
node.run_state[:artifactory_client_username] = delivery_bus_secrets['artifactory_username']
node.run_state[:artifactory_client_password] = delivery_bus_secrets['artifactory_password']

# TODO: package path pattern in delivery-bus
artifactory_omnibus_publisher "#{omnibus_base_dir}/**/*.{bff,deb,dmg,msi,rpm,solaris,amd64.sh,i386.sh}" do
  repository 'omnibus-unstable-local'
  base_path 'com/getchef'
  build_record false
  platform_mappings(
    'ubuntu-14.04' => %w(
      ubuntu-12.04
      ubuntu-14.04
    ),
    'el-6' => %w(
      el-6
      el-7
    ),
    'mac_os_x-10.8' => %w(
      mac_os_x-10.9
      mac_os_x-10.10
      mac_os_x-10.11
    ),
    'windows-2008r2' => %w(
      windows-2012
      windows-2012r2
    )
  )
  properties(
    'delivery.change' => delivery_change_id,
    'delivery.sha' => node['delivery']['change']['sha'], # TODO: expose this in delivery-sugar
  )
end
