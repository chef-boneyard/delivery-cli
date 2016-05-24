default['delivery_rust']['ruby_version'] = '2.1.5'
default['delivery_rust']['rust_version'] = '1.8.0'

if platform_family == 'windows'
  override['omnibus']['ruby_version'] = '2.1.6-x64'
  # Working around a weird bug in the 7-zip cookbook where sometimes the installer ignores INSTALLDIR
  override['7-zip']['home'] = 'C:\\Program Files\\7-Zip'
end

