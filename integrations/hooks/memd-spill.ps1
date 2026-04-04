param(
  [string]$BaseUrl = $(if ($env:MEMD_BASE_URL) { $env:MEMD_BASE_URL } else { "http://127.0.0.1:8787" }),
  [switch]$Apply,
  [switch]$SpillTransient
)

$args = @("--base-url", $BaseUrl, "hook", "spill")
if ($Apply) { $args += "--apply" }
if ($SpillTransient) { $args += "--spill-transient" }
memd @args
