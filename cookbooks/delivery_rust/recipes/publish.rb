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

omnibus_path = File.join(delivery_workspace, 'omnibus-delivery-cli')
secrets = get_project_secrets

#########################################################################
# PUBLISH TO GITHUB
#########################################################################

delivery_github 'Push delivery-cli to GitHub' do
  cache_path delivery_workspace_cache
  deploy_key secrets['github']
  remote_name 'github'
  remote_url 'git@github.com:chef/delivery-cli.git'
end

#########################################################################
# BUILD
#########################################################################

omnibus_build 'delivery-cli' do
  project_dir omnibus_path
  build_user 'dbuild' # TODO: expose this in delivery-sugar DSL
  log_level :internal
  config_overrides(
    append_timestamp: true
  )
end

#########################################################################
# PUBLISH TO ARTIFACTORY
#########################################################################

# TODO: config data pushed up into delivery-bus
node.set['artifactory-pro']['endpoint'] = 'http://artifactory.chef.co'
node.run_state[:artifactory_client_username] = 'delivery'
node.run_state[:artifactory_client_password] = secrets['artifactory_password']

# TODO: package path pattern in delivery-bus
artifactory_omnibus_publisher "#{omnibus_path}/**/*.{bff,deb,dmg,msi,rpm,solaris,amd64.sh,i386.sh}" do
  repository 'omnibus-unstable-local'
  base_path 'com/getchef'
  build_record false
  platform_mappings(
    'ubuntu-12.04' => %w(
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
