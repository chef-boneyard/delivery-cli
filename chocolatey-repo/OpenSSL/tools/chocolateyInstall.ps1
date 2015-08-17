$package = 'OpenSSL'

try {
  $params = @{
    packageName = $package;
    fileType = 'exe';
    #InnoSetup - http://unattended.sourceforge.net/InnoSetup_Switches_ExitCodes.html
    silentArgs = '/silent', '/verysilent', '/sp-', '/suppressmsgboxes' ;
    url = 'http://slproweb.com/download/Win32OpenSSL-1_0_1p.exe'
    url64bit = 'http://slproweb.com/download/Win64OpenSSL-1_0_1p.exe'
  }

  Install-ChocolateyPackage @params

  if (!$Env:OPENSSL_CONF)
  {
    $configPath = 'C:\OpenSSL-Win64\bin\openssl.cfg'

    if (Test-Path $configPath)
    {
      [Environment]::SetEnvironmentVariable(
        'OPENSSL_CONF', $configPath, 'User')

      Write-Host "Configured OPENSSL_CONF variable as $configPath"
    }
  }

  Write-ChocolateySuccess $package
} catch {
  Write-ChocolateyFailure $package "$($_.Exception.Message)"
  throw
}
