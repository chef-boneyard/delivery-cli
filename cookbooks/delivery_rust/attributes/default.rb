default['delivery_rust']['rust_version'] = '2015-10-03'
override['omnibus']['ruby_version'] = '2.1.6-x64' if node['platform_family'] == 'windows'
