execute "bundle install" do
  cwd node['delivery']['repo']
end
