omnibus_path = File.join(node['delivery_builder']['repo'], 'omnibus-delivery-cli')

execute "bundle install --binstubs=#{omnibus_path}/bin" do
  cwd omnibus_path
end

execute "#{omnibus_path}/bin/omnibus build delivery-cli" do
  cwd omnibus_path
end

# Here is where you would put your asset upload

