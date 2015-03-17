omnibus_path = File.join(node['delivery_builder']['repo'], 'omnibus-delivery-cli')

execute "bundle install --binstubs=#{omnibus_path}/bin --path=#{File.join(node['delivery_builder']['cache'], 'gems')}" do
  cwd omnibus_path
end

execute "#{omnibus_path}/bin/omnibus build delivery-cli" do
  cwd omnibus_path
end

#
# This was used manually to upload delivery-cli packages. When we actually get new builders
# in production, we need to replace the values here with ones that come from the builders
# themsleves, so that we avoid showing the credentials to the world. For now, they are here
# for posterity, and lastpass has the password for the delivery artifactory user.
#
# [ "12.04", "14.04" ].each do |pv|
#   delivery_rust_artifactory "delivery-cli" do
#     package_path File.join(omnibus_path, "pkg", "*.deb")
#     repository 'omnibus-current-local'
#     platform 'ubuntu'
#     platform_version pv
#     endpoint 'http://artifactory.chef.co/'
#     base_path 'com/getchef'
#     username 'delivery'
#     password 'SECRET'
#   end
# end
