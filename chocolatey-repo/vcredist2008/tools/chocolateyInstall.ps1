try {
  $processor = Get-WmiObject Win32_Processor
  $is64bit = $processor.AddressWidth -eq 64
  Install-ChocolateyPackage 'vcredist2008' 'exe' '/Q' 'http://download.microsoft.com/download/1/1/1/1116b75a-9ec3-481a-a3c8-1777b5381140/vcredist_x86.exe'
  if($is64bit) {
  	Install-ChocolateyPackage 'vcredist2008_x64' 'exe' '/Q' 'http://download.microsoft.com/download/d/2/4/d242c3fb-da5a-4542-ad66-f9661d0a8d19/vcredist_x64.exe'
  }

  # the following is all part of error handling
  Write-ChocolateySuccess 'vcredist2008'
} catch {
  Write-ChocolateyFailure 'vcredist2008' "$($_.Exception.Message)"
  throw
}
