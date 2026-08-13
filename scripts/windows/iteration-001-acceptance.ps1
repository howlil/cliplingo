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
        $answer = (Read-Host "$Prompt [y/n/s]").Trim().ToLowerInvariant()
        switch ($answer) {
            'y' { return 'yes' }
            'n' { return 'no' }
            's' { return 'skip' }
            default { Write-Host 'Enter y = yes, n = no, or s = unavailable/skip.' }
        }
    }
}

function Escape-Markdown {
    param([AllowNull()][object]$Value)

    if ($null -eq $Value) { return 'N/A' }
    return $Value.ToString().Replace('|', '\|').Replace("`r", ' ').Replace("`n", ' ')
}

function Get-LogLines {
    if (-not (Test-Path $StderrPath)) { return @() }
    return @(Get-Content $StderrPath -ErrorAction SilentlyContinue)
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

function Get-PercentileMilliseconds {
    param(
        [double[]]$ValuesMicroseconds,
        [double]$Percentile
    )

    if ($null -eq $ValuesMicroseconds -or $ValuesMicroseconds.Count -eq 0) { return $null }
    $sorted = @($ValuesMicroseconds | Sort-Object)
    $rank = [Math]::Ceiling($Percentile * $sorted.Count)
    $index = [Math]::Max(0, [int]$rank - 1)
    return [Math]::Round(([double]$sorted[$index] / 1000.0), 2)
}

function Get-TimingSummary {
    param([string[]]$Lines)

    $records = @{}
    foreach ($line in $Lines) {
        if ($line -notmatch 'request_id=(\d+) metric=([a-z_]+) duration_us=(\d+)') { continue }

        $requestId = [int64]$Matches[1]
        $metric = $Matches[2]
        $duration = [double]$Matches[3]
        if ($metric -eq 'capture' -and $line -match 'status=error') { continue }

        if (-not $records.ContainsKey($requestId)) {
            $records[$requestId] = @{}
        }
        $records[$requestId][$metric] = $duration
    }

    $readyIds = @(
        $records.Keys |
            Where-Object { $records[$_].ContainsKey('hotkey_to_ready_request') } |
            Sort-Object |
            Select-Object -Last 20
    )

    $show = @()
    $capture = @()
    $ready = @()
    foreach ($requestId in $readyIds) {
        $record = $records[$requestId]
        if ($record.ContainsKey('hotkey_to_popup_show_request')) { $show += [double]$record['hotkey_to_popup_show_request'] }
        if ($record.ContainsKey('capture')) { $capture += [double]$record['capture'] }
        if ($record.ContainsKey('hotkey_to_ready_request')) { $ready += [double]$record['hotkey_to_ready_request'] }
    }

    return [pscustomobject]@{
        Samples = $readyIds.Count
        ShowP50Ms = Get-PercentileMilliseconds $show 0.50
        ShowP95Ms = Get-PercentileMilliseconds $show 0.95
        CaptureP50Ms = Get-PercentileMilliseconds $capture 0.50
        CaptureP95Ms = Get-PercentileMilliseconds $capture 0.95
        ReadyP50Ms = Get-PercentileMilliseconds $ready 0.50
        ReadyP95Ms = Get-PercentileMilliseconds $ready 0.95
    }
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

function Invoke-CompatibilityCase {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Version,
        [Parameter(Mandatory)][string]$Instructions
    )

    Write-Host ''
    Write-Host "--- $Name ---"
    Write-Host $Instructions
    $availability = Read-Outcome "Can this case be tested now?"
    if ($availability -ne 'yes') {
        return [pscustomobject]@{
            Application = $Name
            Version = $Version
            CaptureSource = 'NOT RUN'
            Bounds = 'NOT RUN'
            Placement = 'NOT RUN'
            Result = 'PENDING'
        }
    }

    $before = (Get-LogLines).Count
    Read-Host "Perform the selection + $Shortcut once, dismiss it with $Shortcut, then press Enter here"
    Start-Sleep -Milliseconds 500
    $captureSource = Get-LatestCaptureSource $before
    if ([string]::IsNullOrWhiteSpace($captureSource)) { $captureSource = 'unknown' }

    $behavior = Read-Outcome 'Did the popup show the expected [FAKE] translation without stealing source-app focus?'
    $bounds = Read-Outcome 'After moving the mouse away before the hotkey, did the popup appear near the selected text?'
    $placement = Read-Outcome 'Did the popup remain fully inside the active monitor work area?'

    $result = if ($behavior -eq 'yes' -and $placement -eq 'yes') { 'PASS' } else { 'FAIL' }
    $boundsLabel = if ($bounds -eq 'yes') { 'selection bounds observed' } elseif ($bounds -eq 'no') { 'cursor/unknown fallback' } else { 'NOT OBSERVED' }
    $placementLabel = if ($placement -eq 'yes') { 'PASS' } elseif ($placement -eq 'no') { 'FAIL' } else { 'NOT OBSERVED' }

    return [pscustomobject]@{
        Application = $Name
        Version = $Version
        CaptureSource = $captureSource
        Bounds = $boundsLabel
        Placement = $placementLabel
        Result = $result
    }
}

if (-not (Test-Path $ExePath)) {
    throw "ClipLingo executable not found: $ExePath"
}

$existing = @(Get-Process cliplingo -ErrorAction SilentlyContinue)
if ($existing.Count -gt 0) {
    throw 'Close any existing ClipLingo process before running acceptance; otherwise global-hotkey ownership and resource numbers are ambiguous.'
}

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$OutputDirectory = (Resolve-Path $OutputDirectory).Path
$StderrPath = Join-Path $OutputDirectory 'cliplingo.stderr.log'
$StdoutPath = Join-Path $OutputDirectory 'cliplingo.stdout.log'

$sampleText = Join-Path $OutputDirectory 'sample.txt'
$sampleHtml = Join-Path $OutputDirectory 'sample.html'
@"
$PrivacySentinel
日本語の選択テキスト
中文选择文本
한국어 선택 텍스트
"@ | Set-Content $sampleText -Encoding utf8
@"
<!doctype html><meta charset="utf-8"><title>ClipLingo acceptance</title>
<p>$PrivacySentinel</p><p>日本語の選択テキスト</p><p>中文选择文本</p><p>한국어 선택 텍스트</p>
"@ | Set-Content $sampleHtml -Encoding utf8

$buildInfo = Read-BuildInfo
$os = Get-CimInstance Win32_OperatingSystem
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$computer = Get-CimInstance Win32_ComputerSystem
$ramGb = [Math]::Round(([double]$computer.TotalPhysicalMemory / 1GB), 2)
$displayDescription = Read-Host 'Describe display/DPI configuration (example: primary 150%, secondary 100%; one monitor if applicable)'

Write-Host ''
Write-Host 'Starting ClipLingo acceptance executable...'
$process = Start-Process -FilePath (Resolve-Path $ExePath).Path -RedirectStandardOutput $StdoutPath -RedirectStandardError $StderrPath -PassThru

try {
    Start-Sleep -Seconds 5
    $process.Refresh()
    if ($process.HasExited) {
        throw "ClipLingo exited during startup with code $($process.ExitCode). See $StderrPath"
    }

    Write-Host 'Measuring idle resource usage for 60 seconds with the popup hidden...'
    $idleStart = Get-Process -Id $process.Id
    $cpuStart = [double]$idleStart.CPU
    Start-Sleep -Seconds 60
    $idleEnd = Get-Process -Id $process.Id
    $cpuDelta = [Math]::Max(0.0, ([double]$idleEnd.CPU - $cpuStart))
    $idleWorkingSetMb = [Math]::Round(([double]$idleEnd.WorkingSet64 / 1MB), 2)
    $idlePrivateMb = [Math]::Round(([double]$idleEnd.PrivateMemorySize64 / 1MB), 2)
    $idleCpuSingleCorePercent = [Math]::Round(($cpuDelta / 60.0 * 100.0), 3)

    Write-Host ''
    Write-Host "Sample text: $sampleText"
    Write-Host "Sample HTML: $sampleHtml"
    Write-Host 'For bounds observation, select text, then move the mouse clearly away from the selection before pressing the hotkey.'

    $cases = @()

    $notepadVersion = 'unknown'
    $notepad = Get-Command notepad.exe -ErrorAction SilentlyContinue
    if ($null -ne $notepad) {
        try { $notepadVersion = (Get-Item $notepad.Source).VersionInfo.FileVersion } catch { }
        Start-Process -FilePath $notepad.Source -ArgumentList ('"{0}"' -f $sampleText) | Out-Null
    }
    $cases += Invoke-CompatibilityCase -Name 'Notepad' -Version $notepadVersion -Instructions "Select only '$PrivacySentinel' in the opened sample file."

    $browserVersion = Read-Host 'Chromium browser + version (for example Edge 151.x / Chrome 151.x), or blank if unavailable'
    if ([string]::IsNullOrWhiteSpace($browserVersion)) { $browserVersion = 'unavailable' }
    $cases += Invoke-CompatibilityCase -Name 'Chromium browser' -Version $browserVersion -Instructions "Open $sampleHtml in the Chromium browser, select one sample line, move the mouse away, then use the hotkey."

    $codeVersion = 'unknown'
    $code = Get-Command code -ErrorAction SilentlyContinue
    if ($null -ne $code) {
        try { $codeVersion = ((& $code.Source --version 2>$null) | Select-Object -First 1) } catch { }
    }
    $cases += Invoke-CompatibilityCase -Name 'VS Code' -Version $codeVersion -Instructions "Open $sampleText in VS Code, select one sample line, move the mouse away, then use the hotkey."

    $pdfVersion = Read-Host 'Selectable PDF reader + version, or blank if unavailable'
    if ([string]::IsNullOrWhiteSpace($pdfVersion)) { $pdfVersion = 'unavailable' }
    $cases += Invoke-CompatibilityCase -Name 'Selectable PDF reader' -Version $pdfVersion -Instructions 'Open any PDF that contains real selectable text (not a scanned image), select text, move the mouse away, then use the hotkey.'

    $stderrText = if (Test-Path $StderrPath) { Get-Content $StderrPath -Raw -ErrorAction SilentlyContinue } else { '' }
    $notepadRan = ($cases | Where-Object { $_.Application -eq 'Notepad' }).Result -ne 'PENDING'
    $privacyStatus = if (-not $notepadRan) {
        'PENDING'
    } elseif ($stderrText.Contains($PrivacySentinel)) {
        'FAIL'
    } else {
        'PASS'
    }

    $clipboardStatus = 'PENDING — no natural clipboard fallback observed'
    $clipboardCase = $cases | Where-Object { $_.CaptureSource -eq 'clipboard' } | Select-Object -First 1
    if ($null -ne $clipboardCase) {
        Write-Host ''
        Write-Host "Clipboard fallback was observed in $($clipboardCase.Application). Running restoration check."
        Set-Clipboard -Value $ClipboardSentinel
        $beforeClipboard = (Get-LogLines).Count
        Read-Host "In $($clipboardCase.Application), select DIFFERENT text and perform one $Shortcut translation + dismiss, then press Enter"
        Start-Sleep -Milliseconds 500
        $source = Get-LatestCaptureSource $beforeClipboard
        $clipboardAfter = Get-Clipboard -Raw
        if ($source -eq 'clipboard' -and $clipboardAfter -ceq $ClipboardSentinel) {
            $clipboardStatus = 'PASS'
        } elseif ($source -ne 'clipboard') {
            $clipboardStatus = 'PENDING — retry did not use clipboard fallback'
        } else {
            $clipboardStatus = 'FAIL — clipboard value changed'
        }
    }

    Write-Host ''
    Write-Host 'Warm latency collection requires 20 successful translation requests.'
    Write-Host 'Use one successful application. For each sample: select text -> Ctrl+Alt+T -> wait for Ready -> Ctrl+Alt+T to dismiss.'
    $beforeWarm = (Get-LogLines).Count
    Read-Host 'Perform at least 20 warm interactions now, then press Enter'
    Start-Sleep -Milliseconds 500
    $warmLines = @(Get-LogLines | Select-Object -Skip $beforeWarm)
    $timing = Get-TimingSummary $warmLines

    $knownGaps = @()
    foreach ($case in $cases) {
        if ($case.Result -ne 'PASS') { $knownGaps += "$($case.Application): $($case.Result)" }
    }
    if ($privacyStatus -ne 'PASS') { $knownGaps += "Privacy log sentinel: $privacyStatus" }
    if ($clipboardStatus -ne 'PASS') { $knownGaps += "Clipboard restoration: $clipboardStatus" }
    if ($timing.Samples -lt 20) { $knownGaps += "Warm latency samples: $($timing.Samples)/20" }
    if ($knownGaps.Count -eq 0) { $knownGaps += 'No blocker observed by this collector; review the evidence before marking the iteration DONE.' }

    $sourceHead = if ($buildInfo.ContainsKey('source_head_sha')) { $buildInfo['source_head_sha'] } else { 'unknown' }
    $ciCheckout = if ($buildInfo.ContainsKey('ci_checkout_sha')) { $buildInfo['ci_checkout_sha'] } else { 'unknown' }
    $runId = if ($buildInfo.ContainsKey('run_id')) { $buildInfo['run_id'] } else { 'unknown' }

    $evidencePath = Join-Path $OutputDirectory 'iteration-001-evidence.md'
    $lines = @()
    $lines += '# Iteration 001 — Local Windows Acceptance Evidence'
    $lines += ''
    $lines += '## Environment'
    $lines += ''
    $lines += "- Collected: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss K')"
    $lines += "- Source head SHA: $sourceHead"
    $lines += "- CI checkout SHA: $ciCheckout"
    $lines += "- CI run ID: $runId"
    $lines += "- Windows: $($os.Caption) $($os.Version) build $($os.BuildNumber)"
    $lines += "- CPU: $(Escape-Markdown $cpu.Name)"
    $lines += "- RAM: $ramGb GB"
    $lines += "- Display/DPI: $(Escape-Markdown $displayDescription)"
    $lines += ''
    $lines += '## Compatibility'
    $lines += ''
    $lines += '| Application | Version | Capture path | Bounds | Placement | Result |'
    $lines += '|---|---|---|---|---|---|'
    foreach ($case in $cases) {
        $lines += "| $(Escape-Markdown $case.Application) | $(Escape-Markdown $case.Version) | $(Escape-Markdown $case.CaptureSource) | $(Escape-Markdown $case.Bounds) | $(Escape-Markdown $case.Placement) | $(Escape-Markdown $case.Result) |"
    }
    $lines += ''
    $lines += '## Privacy'
    $lines += ''
    $lines += "- Selected-text sentinel absent from ClipLingo stderr: **$privacyStatus**"
    $lines += '- Review both stdout/stderr files before committing evidence; this automatic check covers the known sentinel only.'
    $lines += ''
    $lines += '## Clipboard restoration'
    $lines += ''
    $lines += "- Result: **$(Escape-Markdown $clipboardStatus)**"
    $lines += ''
    $lines += '## Latency'
    $lines += ''
    $lines += 'Nearest-rank p50/p95 from the last up-to-20 successful Ready requests after the warm-test marker. Units are milliseconds.'
    $lines += ''
    $lines += '| Metric | Samples | p50 | p95 |'
    $lines += '|---|---:|---:|---:|'
    $lines += "| Hotkey -> popup show request | $($timing.Samples) | $(Escape-Markdown $timing.ShowP50Ms) | $(Escape-Markdown $timing.ShowP95Ms) |"
    $lines += "| Capture duration | $($timing.Samples) | $(Escape-Markdown $timing.CaptureP50Ms) | $(Escape-Markdown $timing.CaptureP95Ms) |"
    $lines += "| Hotkey -> ready request | $($timing.Samples) | $(Escape-Markdown $timing.ReadyP50Ms) | $(Escape-Markdown $timing.ReadyP95Ms) |"
    $lines += ''
    $lines += '## Idle resources'
    $lines += ''
    $lines += '| Build | Working set | Private memory | CPU observation |'
    $lines += '|---|---:|---:|---|'
    $lines += "| CI debug acceptance executable | $idleWorkingSetMb MB | $idlePrivateMb MB | $idleCpuSingleCorePercent% average single-core equivalent over 60 s |"
    $lines += ''
    $lines += '## Known gaps / blockers'
    $lines += ''
    foreach ($gap in $knownGaps) { $lines += "- $(Escape-Markdown $gap)" }
    $lines += ''
    $lines += '## Raw logs'
    $lines += ''
    $lines += ('- stderr: `{0}`' -f (Split-Path $StderrPath -Leaf))
    $lines += ('- stdout: `{0}`' -f (Split-Path $StdoutPath -Leaf))
    $lines += ''
    $lines += 'Do not mark Iteration 001 DONE solely because this script completed. Review the evidence and resolve every failed/pending required case.'

    $lines | Set-Content $evidencePath -Encoding utf8

    Write-Host ''
    Write-Host "Evidence written to: $evidencePath"
    Write-Host "Raw stderr: $StderrPath"
    Write-Host "Raw stdout: $StdoutPath"
    if ($knownGaps.Count -eq 1 -and $knownGaps[0] -like 'No blocker observed*') {
        Write-Host 'Collector found no required blocker. Evidence still needs review before iteration closeout.'
    } else {
        Write-Host 'Iteration remains IN PROGRESS. Review the Known gaps / blockers section.'
    }
}
finally {
    if ($null -ne $process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
}
