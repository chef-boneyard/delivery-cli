$Url = 'http://static.rust-lang.org/dist/2015-08-17/rust-nightly-i686-pc-windows-gnu.msi'
$Url64 = 'http://static.rust-lang.org/dist/2015-08-17/rust-nightly-x86_64-pc-windows-gnu.msi'

Install-ChocolateyPackage 'rust' 'msi' "/qn ADDLOCAL=Rustc,Gcc,Docs,Cargo,Path" "$Url" "$Url64"
