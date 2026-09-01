[CmdletBinding()]
param(
    [string]$ExePath = (Join-Path $PSScriptRoot 'cliplingo.exe'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot ("evidence-{0}" -f (Get-Date -Format 'yyyyMMdd-HHmmss')))
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Shortcut = 'Ctrl+Alt+T'
$PrivacySentinel = 'CLIPLINGO_PRIVACY_SENTINEL_001'
$ClipboardSentinel = 'CLIPBOARD_SENTINEL_001_日本語_中文_한국어'
$StderrPath = Join-Path $OutputDirectory 'cliplingo.stderr.log'
$StdoutPath = Join-Path $OutputDirectory 'cliplingo.stdout.log'

function Read-Outcome {
    param([Parameter(Mandatory)][string]$Prompt)

    while ($true) {
        $answer = (Read-Host "$Prompt [y/n]").Trim().ToLowerInvariant()
        switch ($answer) {
            'y' { return $true }
            'n' { return $false }
            default { Write-Host 'Enter y = yes or n = no.' }
        }
    }
}

function Escape-Markdown {
    param([AllowNull()][object]$Value)

    if ($null -eq $Value) { return 'N/A' }
    return $Value.ToString().Replace('|', '\|').Replace("`r", ' ').Replace("`n", ' ')
}

function Get-LogLines {
    $lines = @()
    if (Test-Path $StderrPath) {
        $lines += @(Get-Content $StderrPath -ErrorAction SilentlyContinue)
    }
    if (Test-Path $StdoutPath) {
        $lines += @(Get-Content $StdoutPath -ErrorAction SilentlyContinue)
    }
    return $lines
}

function Get-LatestCaptureSource {
    param([int]$FromLineCount)

    $source = $null
    $lines = @(Get-LogLines | Select-Object -Skip $FromLineCount)
    foreach ($line in $lines) {
        if ($line -match 'metric=capture .*capture_source=(uia|clipboard)') {
            $source = $Matches[1]
        }
    }
    return $source
}

function Read-BuildInfo {
    $result = @{}
    $path = Join-Path $PSScriptRoot 'build-info.txt'
    if (-not (Test-Path $path)) { return $result }

    foreach ($line in Get-Content $path) {
        if ($line -match '^([^=]+)=(.*)$') {
            $result[$Matches[1]] = $Matches[2]
        }
    }
    return $result
}

function Invoke-SmokeCase {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Instructions
    )

    Write-Host ''
    Write-Host "--- $Name ---"
    Write-Host $Instructions

    $before = (Get-LogLines).Count
    Read-Host "Perform the selection + $Shortcut once, wait for the result, then press Enter here"
    Start-Sleep -Milliseconds 500

    $captureSource = Get-LatestCaptureSource $before
    if ([string]::IsNullOrWhiteSpace($captureSource)) { $captureSource = 'unknown' }

    $translationOk = Read-Outcome 'Did the popup show the expected [FAKE] translation?'
    $focusOk = Read-Outcome 'Did the source application remain usable without an unwanted focus steal?'
    $placementOk = Read-Outcome 'Was the popup fully visible and usable on this display configuration?'

    $result = if ($translationOk -and $focusOk -and $placementOk) { 'PASS' } else { 'FAIL' }

    return [pscustomobject]@{
        Application = $Name
        CaptureSource = $captureSource
        Translation = if ($translationOk) { 'PASS' } else { 'FAIL' }
        Focus = if ($focusOk) { 'PASS' } else { 'FAIL' }
        Placement = if ($placementOk) { 'PASS' } else { 'FAIL' }
        Result = $result
    }
}

if (-not (Test-Path $ExePath)) {
    throw "ClipLingo executable not found: $ExePath"
}

$existing = @(Get-Process cliplingo -ErrorAction SilentlyContinue)
if ($existing.Count -gt 0) {
    throw 'Close any existing ClipLingo process before running acceptance so hotkey ownership and logs are unambiguous.'
}

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$OutputDirectory = (Resolve-Path $OutputDirectory).Path
$StderrPath = Join-Path $OutputDirectory 'cliplingo.stderr.log'
$StdoutPath = Join-Path $OutputDirectory 'cliplingo.stdout.log'

$sampleText = Join-Path $OutputDirectory 'sample.txt'
@"
$PrivacySentinel
A small piece of selectable text for ClipLingo alpha smoke.
日本語の選択テキスト
中文选择文本
한국어 선택 텍스트
"@ | Set-Content $sampleText -Encoding utf8

$buildInfo = Read-BuildInfo
$os = Get-CimInstance Win32_OperatingSystem
$displayDescription = Read-Host 'Describe the tested display/DPI setup (example: one monitor 150%)'

Write-Host ''
Write-Host 'Starting ClipLingo alpha smoke...'
$process = Start-Process -FilePath (Resolve-Path $ExePath).Path -RedirectStandardOutput $StdoutPath -RedirectStandardError $StderrPath -PassThru

try {
    Start-Sleep -Seconds 3
    $process.Refresh()
    if ($process.HasExited) {
        throw "ClipLingo exited during startup with code $($process.ExitCode). See $StderrPath"
    }

    Write-Host ''
    Write-Host "Sample file: $sampleText"
    Write-Host 'This collector intentionally checks only the alpha merge gate.'
    Write-Host 'Broad compatibility, performance, memory, CPU, and multi-monitor certification are beta/stable follow-ups.'

    $notepad = Get-Command notepad.exe -ErrorAction SilentlyContinue
    if ($null -ne $notepad) {
        Start-Process -FilePath $notepad.Source -ArgumentList ('"{0}"' -f $sampleText) | Out-Null
    }

    $cases = @()
    $cases += Invoke-SmokeCase -Name 'Notepad' -Instructions "In Notepad, select only '$PrivacySentinel'."

    Write-Host ''
    Write-Host 'Use one additional representative selectable Windows application already available on this machine.'
    $secondApp = ''
    while ([string]::IsNullOrWhiteSpace($secondApp)) {
        $secondApp = (Read-Host 'Second application name + version (for example Edge 151.x or VS Code 1.xx)').Trim()
    }
    $cases += Invoke-SmokeCase -Name $secondApp -Instructions "Open $sampleText or equivalent selectable text in $secondApp and select a non-empty text fragment."

    $clipboardStatus = 'NOT REQUIRED — tested paths used UI Automation'
    $clipboardCase = $cases | Where-Object { $_.CaptureSource -eq 'clipboard' } | Select-Object -First 1
    if ($null -ne $clipboardCase) {
        Write-Host ''
        Write-Host "Clipboard fallback was observed in $($clipboardCase.Application). Checking restoration."
        Set-Clipboard -Value $ClipboardSentinel
        $beforeClipboard = (Get-LogLines).Count
        Read-Host "In $($clipboardCase.Application), select DIFFERENT text, translate once with $Shortcut, then press Enter"
        Start-Sleep -Milliseconds 500

        $source = Get-LatestCaptureSource $beforeClipboard
        $clipboardAfter = Get-Clipboard -Raw
        if ($source -eq 'clipboard' -and $clipboardAfter -ceq $ClipboardSentinel) {
            $clipboardStatus = 'PASS'
        } elseif ($source -ne 'clipboard') {
            $clipboardStatus = 'FAIL — restoration retry did not exercise clipboard fallback'
        } else {
            $clipboardStatus = 'FAIL — clipboard content changed'
        }
    }

    $logText = (Get-LogLines) -join "`n"
    $privacyStatus = if ($logText.Contains($PrivacySentinel)) { 'FAIL' } else { 'PASS' }

    $caseFailures = @($cases | Where-Object { $_.Result -ne 'PASS' })
    $clipboardFailed = $clipboardStatus.StartsWith('FAIL')
    $overall = if ($caseFailures.Count -eq 0 -and $privacyStatus -eq 'PASS' -and -not $clipboardFailed) { 'PASS' } else { 'FAIL' }

    $sourceHead = if ($buildInfo.ContainsKey('source_head_sha')) { $buildInfo['source_head_sha'] } else { 'unknown' }
    $ciCheckout = if ($buildInfo.ContainsKey('ci_checkout_sha')) { $buildInfo['ci_checkout_sha'] } else { 'unknown' }
    $runId = if ($buildInfo.ContainsKey('run_id')) { $buildInfo['run_id'] } else { 'unknown' }

    $evidencePath = Join-Path $OutputDirectory 'iteration-001-alpha-smoke.md'
    $lines = @()
    $lines += '# Iteration 001 — Windows Alpha Smoke Evidence'
    $lines += ''
    $lines += "- Result: **$overall**"
    $lines += "- Collected: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss K')"
    $lines += "- Source head SHA: $(Escape-Markdown $sourceHead)"
    $lines += "- CI checkout SHA: $(Escape-Markdown $ciCheckout)"
    $lines += "- CI run ID: $(Escape-Markdown $runId)"
    $lines += "- Windows: $(Escape-Markdown $os.Caption) $($os.Version)"
    $lines += "- Display/DPI: $(Escape-Markdown $displayDescription)"
    $lines += ''
    $lines += '## Representative application smoke'
    $lines += ''
    $lines += '| Application | Capture | Translation | Focus | Placement | Result |'
    $lines += '|---|---|---|---|---|---|'
    foreach ($case in $cases) {
        $lines += "| $(Escape-Markdown $case.Application) | $($case.CaptureSource) | $($case.Translation) | $($case.Focus) | $($case.Placement) | $($case.Result) |"
    }
    $lines += ''
    $lines += '## Safety checks'
    $lines += ''
    $lines += "- Clipboard fallback/restoration: $clipboardStatus"
    $lines += "- Privacy sentinel absent from stdout/stderr: $privacyStatus"
    $lines += ''
    $lines += '## Scope note'
    $lines += ''
    $lines += 'This is intentionally alpha-grade evidence. Exhaustive application compatibility, broad DPI/multi-monitor coverage, p50/p95 benchmarking, and idle resource certification are follow-up work and are not blockers for this slice.'

    $lines | Set-Content $evidencePath -Encoding utf8

    Write-Host ''
    Write-Host "Alpha smoke result: $overall"
    Write-Host "Evidence written to: $evidencePath"

    if ($overall -ne 'PASS') {
        exit 1
    }
}
finally {
    if ($null -ne $process) {
        try {
            $process.Refresh()
            if (-not $process.HasExited) {
                Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
            }
        }
        catch { }
    }
}
