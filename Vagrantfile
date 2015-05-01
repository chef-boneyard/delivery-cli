Vagrant.configure("2") do |config|
  config.vm.box = "chef/windows-8.1-professional"
  config.vm.communicator = "winrm"

  ["virtualbox", "vmware_fusion"].each do |provider_name|
    config.vm.provider provider_name do |v|
      v.gui = true
      v.memory = 2048
      v.cpus = 2
    end
  end

  # Install Chocolatey
  config.vm.provision "shell", inline: "(iex ((new-object net.webclient).DownloadString('https://chocolatey.org/install.ps1')))>$null 2>&1"

  # Install software from public chocolatey repo:
  #
  # Git - needed to interact with the delivery server
  # Mingw - needed to compile C code that comes in some Rust Crates
  #
  # Also, Mingw-64 ships with incomplete default compiler settings, so we need to set env vars or else things won't compile.
  config.vm.provision "shell", inline: <<-INLINE
    choco install git.install mingw -y
    [Environment]::SetEnvironmentVariable('C_INCLUDE_PATH', 'C:/mingw/include', 'User')
  INLINE

  # Build and install custom Chocolatey packages for Super-Specific Dependencies(TM)
  #
  # Rust - We depend upon Rust nightly 2015-04-01, Chocolatey doesn't have this version.
  # OpenSSL - The Crate openssl_sys requires OpenSSL headers and libraries to compile,
  #           but Chocolatey currently only packages OpenSSL "Lite" (no headers or libs).
  #           We're using the non-"Lite" package from the same OpenSSL Windows binary distro at slproweb.com.
  # VCRedist2008 - That OpenSSL distribution requires MSVC2008 runtime 9.0.21022, Chocolatey doesn't have this version.
  #
  # Also, OpenSSL doesn't install into the Mingw toolchain, so we need to set some more env vars.
  config.vm.provision "shell", inline: <<-INLINE
    cd C:/vagrant/chocolatey-repo
    if (!(Test-Path rust*.nupkg)) { choco pack rust/rust.nuspec }
    if (!(Test-Path OpenSSL*.nupkg)) { choco pack OpenSSL/OpenSSL.nuspec }
    if (!(Test-Path vcredist2008*.nupkg)) { choco pack vcredist2008/vcredist2008.nuspec }
    choco install rust OpenSSL -y -s $PWD
    [Environment]::SetEnvironmentVariable('OPENSSL_LIB_DIR', 'C:/OpenSSL-Win64', 'User')
    [Environment]::SetEnvironmentVariable('OPENSSL_INCLUDE_DIR', 'C:/OpenSSL-Win64/include', 'User')
  INLINE
end
