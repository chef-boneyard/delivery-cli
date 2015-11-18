rust_execute 'cargo clean' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
end

rust_execute 'cargo test' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
  environment(
    'RUST_TEST_TASKS' => "1"
  )
end

ruby_execute 'make cucumber' do
  version node['delivery_rust']['ruby_version']
  cwd node['delivery_builder']['repo']
end
