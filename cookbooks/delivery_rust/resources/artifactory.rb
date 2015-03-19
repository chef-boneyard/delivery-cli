actions :create

# provides :artifactory
default_action :create

attribute :package_name, :kind_of => String, :name_attribute => true
attribute :package_path, :kind_of => String
attribute :repository, :kind_of => String, :required => true
attribute :platform, :kind_of => String, :required => true
attribute :platform_version, :kind_of => String, :required => true
attribute :endpoint, :regex => [ /^http(s?):\/\// ], :required => true
attribute :base_path, :kind_of => String, :required => true
attribute :username, :kind_of => String, :required => true
attribute :password, :kind_of => String, :required => true

