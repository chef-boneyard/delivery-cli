rust_execute 'cargo clean' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
end

rust_execute 'cargo test' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
  environment(
    'RUST_TEST_TASKS' => "1",
    'RUST_BACKTRACE'  => "1"
  )
end

rust_execute 'cargo build --release' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
  environment( node['delivery_builder']['cargo_env'] )
end

ruby_execute 'bundle install --binstubs=bin --path=vendor/bundle && bin/cucumber 2>/dev/null && rm -rf features/tmp' do
  version node['delivery_rust']['ruby_version']
  cwd node['delivery_builder']['repo']
end

