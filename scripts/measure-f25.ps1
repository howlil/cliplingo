# Manual Windows process-tree memory diagnostic. No capture or network.
param(
  [string]$ProcessName = "cliplingo",
  [int]$TargetProcessId = 0,
  [ValidateSet("idle-tray","settings-open","cold-model","warm-model","evicted")]
  [string]$Phase = "idle-tray",
  [ValidateRange(1,3600)][int]$Samples = 30,
  [ValidateRange(100,60000)][int]$IntervalMs = 1000,
  [string]$Output = "f25-memory.json"
)
$ErrorActionPreference = "Stop"
if ($TargetProcessId -gt 0) {
  $root = Get-Process -Id $TargetProcessId -ErrorAction Stop
} else {
  $processes = @(Get-Process -Name $ProcessName -ErrorAction Stop)
  if ($processes.Count -ne 1) {
    throw "Expected exactly one process; specify -TargetProcessId"
  }
  $root = $processes[0]
}
$processId = [int]$root.Id
$rows = [System.Collections.Generic.List[object]]::new()
for ($i = 0; $i -lt $Samples; $i++) {
  $all = @(Get-CimInstance Win32_Process)
  $owned = [System.Collections.Generic.HashSet[int]]::new()
  [void]$owned.Add($processId)
  do {
    $added = 0
    foreach ($proc in $all) {
      $id = [int]$proc.ProcessId
      if (-not $owned.Contains($id) -and $owned.Contains([int]$proc.ParentProcessId)) {
        [void]$owned.Add($id); $added++
      }
    }
  } while ($added -gt 0)
  $members = @($all | Where-Object { $owned.Contains([int]$_.ProcessId) })
  if ($members.Count -eq 0) { throw "Root process exited while sampling" }
  $private = [long](($members | Measure-Object -Property PrivatePageCount -Sum).Sum)
  $working = [long](($members | Measure-Object -Property WorkingSetSize -Sum).Sum)
  $rows.Add([pscustomobject]@{
    sample=$i; utc=(Get-Date).ToUniversalTime().ToString("o")
    processCount=$members.Count; privateBytes=$private; workingSetBytes=$working
  })
  if ($i -lt ($Samples - 1)) { Start-Sleep -Milliseconds $IntervalMs }
}
function Percentile([long[]]$values, [double]$p) {
  $sorted = @($values | Sort-Object)
  $index = [Math]::Min($sorted.Count-1,[Math]::Max(0,[int][Math]::Ceiling($sorted.Count*$p)-1))
  return [long]$sorted[$index]
}
$privates = [long[]]@($rows | ForEach-Object { $_.privateBytes })
$working = [long[]]@($rows | ForEach-Object { $_.workingSetBytes })
$report = [pscustomobject]@{
  schema=1; phase=$Phase; rootProcessId=$processId
  source="Win32_Process approximate process tree; PID reuse can affect samples"
  note="Private/working memory only; no user text or screen content"
  summary=[pscustomobject]@{
    samples=$Samples
    privateBytesP50=(Percentile $privates 0.50)
    privateBytesP95=(Percentile $privates 0.95)
    privateBytesMax=(Percentile $privates 1.0)
    workingSetBytesP50=(Percentile $working 0.50)
    workingSetBytesP95=(Percentile $working 0.95)
    workingSetBytesMax=(Percentile $working 1.0)
  }
  measurements=$rows
}
$report | ConvertTo-Json -Depth 5 | Set-Content -Path $Output -Encoding UTF8
Write-Host "Wrote $Phase process-tree memory sample to $Output."
