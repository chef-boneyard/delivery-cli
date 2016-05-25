default['delivery_rust']['ruby_version'] = '2.1.5'

rust_version = '1.8.0'
time = Time.now.utc.strftime("+%Y-%m-%dT%H:%M:%SZ")
default['delivery_rust']['rust_version'] = rust_version
default['delivery_rust']['cargo_env'] = {
  'RUSTC_VERSION' => rust_version,
  'DELIV_CLI_TIME' => time,
  'OPENSSL_INCLUDE_DIR' => '/opt/chefdk/embedded/include',
  'OPENSSL_LIB_DIR' => '/opt/chefdk/embedded'
}

if platform_family == 'windows'
  override['omnibus']['ruby_version'] = '2.1.6-x64'
  # Working around a weird bug in the 7-zip cookbook where sometimes the installer ignores INSTALLDIR
  override['7-zip']['home'] = 'C:\\Program Files\\7-Zip'
end

