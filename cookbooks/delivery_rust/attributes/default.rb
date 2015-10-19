default['delivery_rust']['rust_version'] = '2015-10-03'
override['omnibus']['ruby_version'] = '2.1.6-x64' if node['platform_family'] == 'windows'
# Working around a weird bug in the 7-zip cookbook where sometimes the installer ignores INSTALLDIR
override['7-zip']['home'] = 'C:\\Program Files\\7-Zip' if node['platform_family'] == 'windows'

openssl_version = '1.0.1p'
default['delivery_rust']['windows']['openssl_url']         = "https://github.com/jaym/windows-openssl-build/releases/download/openssl-#{openssl_version}/openssl-#{openssl_version}-x64-windows.tar.lzma"
default['delivery_rust']['windows']['openssl_checksum']    = 'e857c3c9f892e1b1881689719ef763acd0a01c6ecf6ad0674f6e538c8739a456'
default['delivery_rust']['windows']['openssl_install_dir'] = "C:\\openssl\\#{openssl_version}"

