execute "cargo clean" do
  cwd node['delivery_builder']['repo']
end

execute "cargo build" do
  cwd node['delivery_builder']['repo']
end
