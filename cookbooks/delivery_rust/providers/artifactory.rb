# provides :artifactory

def whyrun_supported?
  true
end

action :create do
  require 'artifactory'
  require 'omnibus'
  require 'omnibus/config'

  Omnibus::Config.artifactory_endpoint(new_resource.endpoint)
  Omnibus::Config.artifactory_base_path(new_resource.base_path)
  Omnibus::Config.artifactory_username(new_resource.username)
  Omnibus::Config.artifactory_password(new_resource.password)

  packages = []
  if new_resource.package_path
    Dir[new_resource.package_path].each do |pkg|
      packages << pkg
    end
  else
    packages.push new_resource.name
  end

  packages.each do |pkg|
    converge_by("Publishing artifact '#{new_resource.name}' package #{pkg}") do
      publisher = Omnibus::ArtifactoryPublisher.new(
        pkg,
        repository: new_resource.repository,
        platform: new_resource.platform,
        platform_version: new_resource.platform_version,
      )
      publisher.publish do |package|
        Chef::Log.debug("Published #{new_resource.name} #{package} to #{new_resource.endpoint}")
      end
    end
  end
end
