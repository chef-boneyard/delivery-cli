# Make sure things still build in case branches merge on each other in a weird way.
rust_execute 'cargo clean' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
end

rust_execute 'cargo build' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
end
