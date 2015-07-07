case node['platform_family']
when 'mac_os_x'
  execute "install openssl via brew" do
    command "brew install openssl"
    not_if "brew list|grep -q openssl"
  end
when 'windows'
  log "windows: implement openssl install, please!"
else
  log "Linux detected"
  include_recipe "build-essential"

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
