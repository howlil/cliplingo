# F2.7 interactive Windows foreground probe (no source text, titles or OCR)
# Manual acceptance only. Start it, focus a real app, press Ctrl+Alt+T
# and inspect whether foreground HWND unexpectedly changes to ClipLingo.
param(
  [ValidateRange(2,60)][int]$Seconds = 15,
  [ValidateRange(20,500)][int]$SampleMs = 40,
  [string]$Output = "f27-foreground-transitions.json"
)
$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ClipLingoForegroundProbe {
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
}
'@
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$events = [System.Collections.Generic.List[object]]::new()
$last = ""
Write-Host "Focus Notepad/Chrome/PDF, select English text, then press Ctrl+Alt+T."
Write-Host "This local probe saves only foreground HWND/PID transitions, never text or titles."
while ($timer.Elapsed.TotalSeconds -lt $Seconds) {
  $hwnd = [ClipLingoForegroundProbe]::GetForegroundWindow()
  $pid = [uint32]0
  if ($hwnd -ne [IntPtr]::Zero) {
    [void][ClipLingoForegroundProbe]::GetWindowThreadProcessId($hwnd, [ref]$pid)
  }
  $key = "$($hwnd.ToInt64()):$pid"
  if ($key -ne $last) {
    $events.Add([pscustomobject]@{
      elapsedMs = [int]$timer.ElapsedMilliseconds
      hwnd = "0x$($hwnd.ToInt64().ToString('X'))"
      pid = $pid
    })
    $last = $key
  }
  Start-Sleep -Milliseconds $SampleMs
}
$report = [pscustomobject]@{
  schema = 1; type = "manual_windows_focus_transitions"
  durationSeconds = $Seconds; sampleIntervalMs = $SampleMs
  limitation = "Transitions alone cannot prove keyboard focus or nonactivation; inspect with the source app and a second native focus test."
  events = $events
}
$report | ConvertTo-Json -Depth 5 | Set-Content -Path $Output -Encoding UTF8
Write-Host "Wrote $Output. No source text, window titles, or network traffic."
