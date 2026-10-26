# Diagnostic only: inspect ClipLingo's native popup UI Automation tree on an
# actual interactive Windows desktop. No source, translation or window title
# content is written to console, JSON, or logs.
#
# Usage: powershell.exe -NoProfile -File scripts/probe-f29-uia.ps1 -Seconds 30
# Start ClipLingo first. During sampling, select public test text in Notepad,
# press Ctrl+Alt+T and keep the result popup visible until a sample is taken.
# This cannot replace a Narrator/NVDA test or prove focus was never stolen.
param(
    [ValidateRange(3, 90)][int]$Seconds = 30,
    [ValidateRange(20, 500)][int]$PollMs = 100,
    [string]$Output = "f29-uia-observation.json"
)
$ErrorActionPreference = "Stop"
if ($PSVersionTable.PSEdition -ne "Desktop") {
    throw "This probe needs Windows PowerShell 5.1 (powershell.exe) with desktop UIAutomationClient."
}
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ClipLingoUiaProbeWin32 {
    [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr FindWindowW(string className, string windowName);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll")]
    public static extern IntPtr GetForegroundWindow();
}
'@

$watch = [System.Diagnostics.Stopwatch]::StartNew()
$observed = $false
$summary = [ordered]@{
    schema = 1
    mode = "manual_interactive_uia_probe"
    windowFound = $false
    popupVisible = $false
    uiaElementAvailable = $false
    textPatternAvailable = $false
    documentRangeNonempty = $false
    documentUtf16Units = 0
    namePropertyNonempty = $false
    nameUtf16Units = 0
    keyboardFocusable = $null
    controlType = ""
    foregroundStableDuringProbe = $null
    inspectionSucceeded = $false
    limitations = "Reports metadata/lengths only. Does not prove translation correctness, real screen-reader announcement, focus across hotkey, wheel routing or GUI accessibility. Full text may be exposed by Name without TextPattern."
}
Write-Host "Trigger ClipLingo Ctrl+Alt+T with public test text while this probe is running."
Write-Host "The report will contain no captured text, translation content, or window titles."
while ($watch.Elapsed.TotalSeconds -lt $Seconds -and -not $observed) {
    $hwnd = [ClipLingoUiaProbeWin32]::FindWindowW("ClipLingoNativePopupF0", $null)
    if ($hwnd -ne [IntPtr]::Zero) {
        $summary.windowFound = $true
        if ([ClipLingoUiaProbeWin32]::IsWindowVisible($hwnd)) {
            $summary.popupVisible = $true
            $before = [ClipLingoUiaProbeWin32]::GetForegroundWindow()
            try {
                $element = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
                if ($null -ne $element) {
                    $summary.uiaElementAvailable = $true
                    $name = $element.Current.Name
                    $summary.namePropertyNonempty = -not [string]::IsNullOrWhiteSpace($name)
                    $summary.nameUtf16Units = if ($null -eq $name) { 0 } else { $name.Length }
                    $summary.keyboardFocusable = [bool]$element.Current.IsKeyboardFocusable
                    $summary.controlType = $element.Current.ControlType.ProgrammaticName
                    $pattern = $null
                    $hasText = $element.TryGetCurrentPattern(
                        [System.Windows.Automation.TextPattern]::Pattern, [ref]$pattern
                    )
                    $summary.textPatternAvailable = [bool]$hasText
                    if ($hasText -and $null -ne $pattern) {
                        $range = ([System.Windows.Automation.TextPattern]$pattern).DocumentRange
                        $fullText = $range.GetText(-1)
                        $summary.documentRangeNonempty = -not [string]::IsNullOrWhiteSpace($fullText)
                        $summary.documentUtf16Units = if ($null -eq $fullText) { 0 } else { $fullText.Length }
                    }
                    # No text or name is printed or persisted.
                    $summary.inspectionSucceeded = $true
                }
            } catch {
                # Do not persist exception details; they may include external UIA
                # provider diagnostics or third-party window metadata.
                $summary.inspectionSucceeded = $false
            } finally {
                $after = [ClipLingoUiaProbeWin32]::GetForegroundWindow()
                $summary.foregroundStableDuringProbe = ($before -eq $after)
            }
            $observed = $true
        }
    }
    if (-not $observed) { Start-Sleep -Milliseconds $PollMs }
}
[pscustomobject]$summary | ConvertTo-Json -Depth 4 |
    Set-Content -LiteralPath $Output -Encoding UTF8
Write-Host "UIA probe result saved to $Output (metadata only)."
if (-not $summary.popupVisible) {
    Write-Warning "Popup was not visible during sampling. Repeat with a verified model and active Notepad selection."
} elseif (-not $summary.textPatternAvailable) {
    Write-Warning "UIA TextPattern not exposed. Screen-reader acceptance remains blocked; this is expected with the current GDI-only HWND."
}
