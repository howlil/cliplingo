# F2.9 manual Win32 focus/foreground probe. No capture or OCR.
# Run in a real interactive Windows desktop with ClipLingo and model started.
param(
    [ValidateRange(3,90)][int]$Seconds = 20,
    [ValidateRange(20,250)][int]$SampleMs = 40,
    [string]$Output = "f29-focus-transitions.json"
)
$ErrorActionPreference = "Stop"
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ClipLingoFocusProbe {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)]
    public struct GUITHREADINFO {
        public uint cbSize;
        public uint flags;
        public IntPtr hwndActive, hwndFocus, hwndCapture, hwndMenuOwner, hwndMoveSize, hwndCaret;
        public RECT rcCaret;
    }
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
    [DllImport("user32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetGUIThreadInfo(uint threadId, ref GUITHREADINFO info);
    public static GUITHREADINFO NewInfo() {
        var info = new GUITHREADINFO();
        info.cbSize = (uint)Marshal.SizeOf(typeof(GUITHREADINFO));
        return info;
    }
}
'@
$elapsed = [System.Diagnostics.Stopwatch]::StartNew()
$events = [System.Collections.Generic.List[object]]::new()
$last = ""
Write-Host "Focus source editor, select text and press Ctrl+Alt+T during sampling."
Write-Host "Scroll over the native result if tall. Only HWND/PID/focus transitions are saved."
while ($elapsed.Elapsed.TotalSeconds -lt $Seconds) {
    $foreground = [ClipLingoFocusProbe]::GetForegroundWindow()
    $pid = [uint32]0
    $tid = if ($foreground -eq [IntPtr]::Zero) { [uint32]0 }
           else { [ClipLingoFocusProbe]::GetWindowThreadProcessId($foreground, [ref]$pid) }
    $info = [ClipLingoFocusProbe]::NewInfo()
    $focusOk = $tid -ne 0 -and [ClipLingoFocusProbe]::GetGUIThreadInfo($tid, [ref]$info)
    $f = "0x$($foreground.ToInt64().ToString('X'))"
    $focused = if ($focusOk) { "0x$($info.hwndFocus.ToInt64().ToString('X'))" } else { "" }
    $key = "$($f):$($pid):$($focused)"
    if ($key -ne $last) {
        $events.Add([pscustomobject]@{
            elapsedMs = [int]$elapsed.ElapsedMilliseconds
            foregroundHwnd = $f
            foregroundPid = $pid
            focusedHwnd = $focused
            focusQuerySucceeded = [bool]$focusOk
        })
        $last = $key
    }
    Start-Sleep -Milliseconds $SampleMs
}
[pscustomobject]@{
    schema = 1
    mode = "manual_interactive_focus_monitor"
    durationSeconds = $Seconds
    sampleIntervalMs = $SampleMs
    limitations = "Sampling can miss transitions shorter than the sample interval. This does not prove mouse-wheel routing, accessibility tree or DWM visibility. Requires human observation."
    events = $events
} | ConvertTo-Json -Depth 6 | Set-Content -Path $Output -Encoding UTF8
Write-Host "Wrote $Output. No window titles, source text or model output recorded."
