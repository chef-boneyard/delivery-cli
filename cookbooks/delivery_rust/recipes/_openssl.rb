case node['platform_family']
when 'mac_os_x'
  execute "install openssl via brew" do
    command "brew install openssl"
    not_if "brew list|grep -q openssl"
  end
when 'windows'
  url = node['delivery_rust']['windows']['openssl_url']
  install_dir = node['delivery_rust']['windows']['openssl_install_dir']
  lzma_file = File.join(Chef::Config['file_cache_path'], File.basename(url))
  tar_file  = File.join(Chef::Config['file_cache_path'], File.basename(url, '.*'))

  remote_file 'OpenSSL Archive' do
    source   url
    path     lzma_file
    checksum node['delivery_rust']['windows']['openssl_checksum']
  end

  execute 'Uncompress LZMA' do
    command "7z.exe x #{lzma_file} -o#{tar_file} -r -y"
    creates tar_file
  end

  execute 'Unpack TAR' do
    command "7z.exe x #{tar_file} -o#{install_dir} -r -y"
    creates install_dir
  end

  legacy_dll = File.join(install_dir, 'bin', 'ssleay32.dll')
  copied_dll = File.join(install_dir, 'bin', 'libssl32.dll')
  powershell_script 'copy dll' do
    code "Copy-Item #{legacy_dll} #{copied_dll}"
    creates copied_dll
  end
else
  log "Linux detected"
  include_recipe "build-essential"

  # Need developement headers for openssl.
  case node['platform_family']
  when "debian"
    package "libssl-dev"
  when "rhel"
    package "openssl-devel"
  end

  openssl_version = "1.0.1m"

  build_deps = "/opt/delivery-cli-build-deps"

  directory build_deps do
    recursive true
  end

  remote_file "#{build_deps}/openssl-#{openssl_version}.tar.gz" do
    source "https://www.openssl.org/source/openssl-#{openssl_version}.tar.gz"
  end

  execute "unpack openssl tarball" do
    cwd build_deps
    command "tar xzf openssl-#{openssl_version}.tar.gz"
    not_if "test -d openssl-#{openssl_version}"
  end

  link "#{build_deps}/openssl" do
    to "#{build_deps}/openssl-#{openssl_version}"
  end

  bash "build openssl" do
    cwd "#{build_deps}/openssl"
    code <<-EOH
    ./config no-idea no-mdc2 no-rc5 -fPIC
    make depend
    make
    EOH
    not_if "test -f #{build_deps}/openssl/libssl.a"
  end
end
