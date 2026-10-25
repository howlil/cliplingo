# Manual Windows-only diagnostic. Does NOT capture screen/text or upload telemetry.
param(
  [string]$ProcessName = "cliplingo",
  [int]$Samples = 12,
  [int]$IntervalMs = 1000,
  [string]$Output = "f0-resource-baseline.json"
)
$ErrorActionPreference = "Stop"
if ($Samples -lt 1 -or $Samples -gt 3600 -or $IntervalMs -lt 100) { throw "Invalid sample count or interval" }
$root = Get-Process -Name $ProcessName -ErrorAction Stop | Select-Object -First 1
$rows = @()
for ($i = 0; $i -lt $Samples; $i++) {
  $processes = @(Get-CimInstance -ClassName Win32_Process)
  $owned = @([int]$root.Id)
  do {
    $next = @($processes | Where-Object { $owned -contains [int]$_.ParentProcessId -and $owned -notcontains [int]$_.ProcessId })
    foreach ($child in $next) { $owned += [int]$child.ProcessId }
  } while ($next.Count -gt 0)
  $picked = @($processes | Where-Object { $owned -contains [int]$_.ProcessId })
  $private = ($picked | Measure-Object -Property PrivatePageCount -Sum).Sum
  $working = ($picked | Measure-Object -Property WorkingSetSize -Sum).Sum
  $rows += [pscustomobject]@{
    sample = $i
    timeUtc = (Get-Date).ToUniversalTime().ToString("o")
    processCount = $picked.Count
    privateBytes = [int64]$private
    workingSetBytes = [int64]$working
  }
  if ($i -lt ($Samples - 1)) { Start-Sleep -Milliseconds $IntervalMs }
}
$report = [pscustomobject]@{
  schema = 1
  target = "F0 experimental; NOT a model inference benchmark"
  processId = [int]$root.Id
  source = "Windows Win32_Process; parent + discovered descendants"
  samples = $rows
}
$report | ConvertTo-Json -Depth 5 | Set-Content -Path $Output -Encoding utf8
Write-Host "Wrote $Output (no captured content)."
