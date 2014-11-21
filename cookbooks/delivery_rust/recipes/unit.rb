delivery_builder_exec "cargo clean" do
  cwd node['delivery_builder']['repo']
end

delivery_builder_exec "cargo test" do
  cwd node['delivery_builder']['repo']
end
