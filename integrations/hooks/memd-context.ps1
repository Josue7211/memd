param(
  [string]$BaseUrl = $(if ($env:MEMD_BASE_URL) { $env:MEMD_BASE_URL } else { "http://127.0.0.1:8787" }),
  [Parameter(Mandatory = $true)][string]$Project = $(if ($env:MEMD_PROJECT) { $env:MEMD_PROJECT } else { throw "MEMD_PROJECT is required" }),
  [Parameter(Mandatory = $true)][string]$Agent = $(if ($env:MEMD_AGENT) { $env:MEMD_AGENT } else { throw "MEMD_AGENT is required" }),
  [string]$Route = $(if ($env:MEMD_ROUTE) { $env:MEMD_ROUTE } else { "auto" }),
  [string]$Intent = $(if ($env:MEMD_INTENT) { $env:MEMD_INTENT } else { "general" }),
  [int]$Limit = $(if ($env:MEMD_LIMIT) { [int]$env:MEMD_LIMIT } else { 8 }),
  [int]$MaxChars = $(if ($env:MEMD_MAX_CHARS) { [int]$env:MEMD_MAX_CHARS } else { 280 })
)

$bundleRoot = if ($env:MEMD_BUNDLE_ROOT) { $env:MEMD_BUNDLE_ROOT } else { ".memd" }
$backendEnvPath = Join-Path $bundleRoot "backend.env.ps1"
$envPath = Join-Path $bundleRoot "env.ps1"
if (Test-Path $backendEnvPath) {
  . $backendEnvPath
}
if (Test-Path $envPath) {
  . $envPath
}

memd --base-url $BaseUrl hook context `
  --project $Project `
  --agent $Agent `
  --route $Route `
  --intent $Intent `
  --limit $Limit `
  --max-chars-per-item $MaxChars
