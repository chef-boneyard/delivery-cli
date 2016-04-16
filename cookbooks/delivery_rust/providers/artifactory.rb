# provides :artifactory

def whyrun_supported?
  true
end

action :create do
  omnibus_path = ::File.join(node['delivery']['workspace']['repo'], 'omnibus')
  omnibus_config = ::File.join(node['delivery']['workspace']['cache'], 'omnibus-publish.rb')

  # Render an Omnibus config file for publishing
  file omnibus_config do
    content <<-EOH
# This file is written by Chef for #{node['fqdn']}.
# Do NOT modify this file by hand.

artifactory_endpoint  '#{new_resource.endpoint}'
artifactory_base_path '#{new_resource.base_path}'
artifactory_username  '#{new_resource.username}'
artifactory_password  '#{new_resource.password}'
  EOH
    mode '0600'
    owner 'dbuild'
    group 'dbuild'
    sensitive true
  end

  packages = []
  if new_resource.package_path
    Dir[new_resource.package_path].each do |pkg|
      packages << pkg
    end
  else
    packages.push new_resource.package_name
  end

  packages.each do |pkg|
    execute "publish artifact '#{new_resource.name}' package #{pkg}" do
      command "#{omnibus_path}/bin/omnibus publish artifactory #{new_resource.repository} #{pkg} " \
              "--config #{omnibus_config} " \
              "--platform #{new_resource.platform} " \
              "--platform-version #{new_resource.platform_version}"
      cwd omnibus_path
    end
  end
end
