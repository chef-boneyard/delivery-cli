rust_execute 'cargo clean' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
end

rust_execute 'cargo test' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
  environment(
    'RUST_TEST_TASKS' => "1",
    'RUST_BACKTRACE'  => "1",
    'GIT_COMMITTER_NAME' => "dbuild@test.com",
    'GIT_AUTHOR_NAME' => "dbuild@test.com",
    'EMAIL' => "dbuild@test.com"
  )
end

rust_execute 'cargo build --release' do
  version node['delivery_rust']['rust_version']
  cwd node['delivery_builder']['repo']
  environment( node['delivery_builder']['cargo_env'] )
end

ruby_execute "bundle install --binstubs=bin --path=#{node['delivery_builder']['cache']}/vendor/bundle" do
  version node['delivery_rust']['ruby_version']
  cwd node['delivery_builder']['repo']
  environment(
    'GIT_COMMITTER_NAME' => "dbuild@test.com",
    'GIT_AUTHOR_NAME' => "dbuild@test.com",
    'EMAIL' => "dbuild@test.com"
  )
end

ruby_execute "bin/cucumber --no-color --format progress 2> #{node['delivery_builder']['cache']}/cucumber_output.txt && rm -rf features/tmp" do
  version node['delivery_rust']['ruby_version']
  cwd node['delivery_builder']['repo']
  environment(
    'GIT_COMMITTER_NAME' => "dbuild@test.com",
    'GIT_AUTHOR_NAME' => "dbuild@test.com",
    'EMAIL' => "dbuild@test.com",
    'BUNDLE_PATH' => "#{node['delivery_builder']['cache']}/vendor/bundle"
  )
end
