# Enforces the include boundaries that CMake targets cannot express (F1, F3, F4).
# System headers are always reachable from MSVC, so a source check is the gate.
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File tools/check_boundaries.ps1 [-SourceRoot <repo>]
[CmdletBinding()]
param(
  [string]$SourceRoot = ''
)

Set-StrictMode -Version 2.0
$ErrorActionPreference = 'Stop'

if (-not $SourceRoot) {
  $script:SourceRoot = Split-Path -Parent $PSScriptRoot
} else {
  $script:SourceRoot = (Resolve-Path -LiteralPath $SourceRoot).Path
}

$sourceDir = Join-Path $script:SourceRoot 'src'
if (-not (Test-Path -LiteralPath $sourceDir)) {
  Write-Error "No src directory under $script:SourceRoot"
  exit 2
}

$script:Failures = 0
$script:Scanned = 0

$osHeaderRule = '^\s*#\s*include\s*[<"](windows|winhttp|winsqlite3|sqlite3|uiautomation|UIAutomation|wincred|slint|shlobj|shellapi|commctrl|shlwapi)'
$outerLayerRule = '^\s*#\s*include\s*[<"](platform|ui)/'
$appLayerRule = '^\s*#\s*include\s*[<"](app)/'

$layers = @(
  @{ Dir = 'src/contracts'; Rules = @(
      @{ Label = 'F1 contracts must stay OS-free'; Pattern = $osHeaderRule }
      @{ Label = 'F1 contracts must not reach platform/ui'; Pattern = $outerLayerRule }
      @{ Label = 'F1 contracts must not reach core/app'; Pattern = '^\s*#\s*include\s*[<"](core)/' }
    ) },
  @{ Dir = 'src/core'; Rules = @(
      @{ Label = 'F1 core must not include OS headers'; Pattern = $osHeaderRule }
      @{ Label = 'F1 core must not reach platform/ui'; Pattern = $outerLayerRule }
      @{ Label = 'F1 core must not reach app'; Pattern = $appLayerRule }
    ) },
  @{ Dir = 'src/platform/windows'; Rules = @(
      @{ Label = 'F3 win32 layer must not include sqlite/winhttp'; Pattern = '^\s*#\s*include\s*[<"](winsqlite3|sqlite3|winhttp)' }
    ) },
  @{ Dir = 'src/platform/storage'; Rules = @(
      @{ Label = 'F4 storage must not include ui/uia/winhttp'; Pattern = '^\s*#\s*include\s*[<"](uiautomation|UIAutomation|winhttp|slint)' }
      @{ Label = 'F4 storage must not reach ui'; Pattern = '^\s*#\s*include\s*[<"]ui/' }
    ) },
  @{ Dir = 'src/platform/translation'; Rules = @(
      @{ Label = 'DEP1 adapter must not include sqlite/uia'; Pattern = '^\s*#\s*include\s*[<"](winsqlite3|sqlite3|uiautomation|UIAutomation)' }
    ) },
  @{ Dir = 'src/ui'; Rules = @(
      @{ Label = 'F2 ui must not include storage/network/uia'; Pattern = '^\s*#\s*include\s*[<"](winsqlite3|sqlite3|winhttp|uiautomation|UIAutomation|platform)' }
    ) }
)

# F3: only platform/windows may touch these OS surfaces.
$osCallRule = '\b(RegisterHotKey|UnregisterHotKey|OpenClipboard|GetClipboardData|SetClipboardData|UIAutomationClient)\b'

function Write-Failure([string]$file, [int]$line, [string]$label) {
  $relative = $file.Substring($script:SourceRoot.Length).TrimStart('\', '/')
  Write-Host ("{0}:{1}: {2}" -f $relative, $line, $label) -ForegroundColor Red
  $script:Failures = $script:Failures + 1
}

function Test-Layer([string]$relativeDirectory, $rules) {
  $directory = Join-Path $script:SourceRoot $relativeDirectory
  if (-not (Test-Path -LiteralPath $directory)) {
    return
  }

  $files = Get-ChildItem -LiteralPath $directory -Recurse -File |
    Where-Object { $_.Extension -in @('.hpp', '.h', '.cpp', '.cc') }

  foreach ($file in $files) {
    $script:Scanned = $script:Scanned + 1
    $lineNumber = 0
    foreach ($line in (Get-Content -LiteralPath $file.FullName)) {
      $lineNumber = $lineNumber + 1
      $code = $line -replace '//.*$', ''
      foreach ($rule in $rules) {
        if ($code -match $rule.Pattern) {
          Write-Failure -file $file.FullName -line $lineNumber -label $rule.Label
        }
      }
      if ($relativeDirectory -ne 'src/platform/windows' -and $code -match $osCallRule) {
        Write-Failure -file $file.FullName -line $lineNumber `
          -label 'F3 OS hotkey/clipboard/UIA calls belong to platform/windows'
      }
    }
  }
}

foreach ($layer in $layers) {
  Test-Layer -relativeDirectory $layer.Dir -rules $layer.Rules
}

if ($script:Failures -gt 0) {
  Write-Host ("FAILED: {0} boundary violation(s) in {1} file(s)." -f $script:Failures, $script:Scanned) -ForegroundColor Red
  exit 1
}

Write-Host ("Layer boundaries OK ({0} file(s) scanned)." -f $script:Scanned) -ForegroundColor Green
exit 0
