execute "cargo clean" do
  cwd node['delivery_builder']['repo']
end

execute "cargo test" do
  if Chef::VERSION !~ /^12/
    environment({
      'RUST_TEST_TASKS' => "1"
    })
  end
  cwd node['delivery_builder']['repo']
end

execute "Cucumber Behavioral Tests" do
  command "make cucumber"
  cwd node['delivery_builder']['repo']
end
