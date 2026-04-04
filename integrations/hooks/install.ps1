param(
  [string]$Prefix = $(if ($env:MEMD_HOOK_PREFIX) { $env:MEMD_HOOK_PREFIX } else { Join-Path $HOME "bin" })
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
New-Item -ItemType Directory -Force -Path $Prefix | Out-Null

Copy-Item -Force (Join-Path $scriptDir "memd-context.ps1") (Join-Path $Prefix "memd-context.ps1")
Copy-Item -Force (Join-Path $scriptDir "memd-spill.ps1") (Join-Path $Prefix "memd-spill.ps1")

Write-Host "Installed memd hooks to $Prefix"
Write-Host "Add $Prefix to PATH if needed."
