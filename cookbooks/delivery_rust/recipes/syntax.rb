delivery_builder_exec "cargo clean" do
  cwd node['delivery_builder']['repo']
end

delivery_builder_exec "cargo build" do
  cwd node['delivery_builder']['repo']
end
