default['delivery_rust']['ruby_version'] = '2.1.5'

rust_version = '1.9.0'
time = Time.now.utc.strftime("+%Y-%m-%dT%H:%M:%SZ")
default['delivery_rust']['rust_version'] = rust_version
default['delivery_rust']['cargo_env'] = {
  'RUSTC_VERSION' => rust_version,
  'DELIV_CLI_TIME' => time,
  'OPENSSL_INCLUDE_DIR' => '/opt/chefdk/embedded/include',
  'OPENSSL_LIB_DIR' => '/opt/chefdk/embedded'
}

