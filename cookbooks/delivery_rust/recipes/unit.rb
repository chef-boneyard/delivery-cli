
execute "cargo clean" do
  cwd node['delivery_builder']['repo']
end

execute "cargo test" do
  cwd node['delivery_builder']['repo']
end
