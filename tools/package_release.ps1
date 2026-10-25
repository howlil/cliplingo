param(
  [Parameter(Mandatory=$true)][string]$Executable,
  [Parameter(Mandatory=$true)][string]$NativePrefix,
  [Parameter(Mandatory=$true)][string]$VcpkgPrefix,
  [Parameter(Mandatory=$true)][string]$OutputFolder,
  [Parameter(Mandatory=$true)][string]$Version
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$exe = (Resolve-Path $Executable).Path
$native = (Resolve-Path $NativePrefix).Path
$vcpkg = (Resolve-Path $VcpkgPrefix).Path
$dist = Join-Path $OutputFolder 'ClipLingo'
if (Test-Path $dist) { Remove-Item $dist -Recurse -Force }
New-Item -ItemType Directory -Force $dist | Out-Null

Copy-Item -LiteralPath $exe -Destination (Join-Path $dist 'ClipLingo.exe')
# DLLs are deliberately copied beside ClipLingo.exe. Do not rely on the PATH
# of the machine that built the artifact or on side-loaded system DLLs.
$nativeBin = Join-Path $native 'bin'
$vcpkgBin = Join-Path $vcpkg 'bin'
foreach ($dllFolder in @($nativeBin, $vcpkgBin)) {
  if (!(Test-Path $dllFolder)) { throw "Missing runtime DLL directory: $dllFolder" }
  Get-ChildItem $dllFolder -Filter '*.dll' -File | ForEach-Object {
    $dest = Join-Path $dist $_.Name
    if (Test-Path $dest) {
      $other = (Get-FileHash $dest -Algorithm SHA256).Hash
      $new = (Get-FileHash $_.FullName -Algorithm SHA256).Hash
      if ($other -ne $new) { throw "Conflicting same-name DLL: $($_.Name)" }
    } else { Copy-Item $_.FullName $dest }
  }
}
$dllNames = @(Get-ChildItem $dist -File -Filter '*.dll' | ForEach-Object Name)
# CTranslate2 is built as a shared DLL. SentencePiece 0.2.1 is
# installed as a static library with the pinned source build; it is linked
# into ClipLingo.exe and therefore has NO sentencepiece.dll to distribute.
# Never invent or require a DLL that the verified linkage does not import.
if ($dllNames -notcontains 'ctranslate2.dll') {
  throw 'Runtime DLL missing: ctranslate2.dll'
}
if (!(@($dllNames | Where-Object { $_ -like '*tesseract*.dll' }).Count)) {
  throw 'Tesseract runtime DLL not included'
}
if (!(@($dllNames | Where-Object { $_ -like '*lept*.dll' }).Count)) {
  throw 'Leptonica runtime DLL not included'
}

$tess = Join-Path $dist 'tessdata'
New-Item -ItemType Directory -Force $tess | Out-Null
$ocrSource = 'https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/4.1.0/eng.traineddata'
Invoke-WebRequest -Uri $ocrSource -OutFile (Join-Path $tess 'eng.traineddata')
if ((Get-Item (Join-Path $tess 'eng.traineddata')).Length -lt 1000000) {
  throw 'Tesseract English traineddata download looks truncated'
}

$lic = Join-Path $dist 'THIRD_PARTY_NOTICES'
New-Item -ItemType Directory -Force $lic | Out-Null
$repoNative = Join-Path $env:RUNNER_TEMP 'ctranslate2'
$repoSp = Join-Path $env:RUNNER_TEMP 'sentencepiece'
foreach ($pair in @(
  @{Name='CTranslate2-LICENSE.txt';Path=(Join-Path $repoNative 'LICENSE')},
  @{Name='SentencePiece-LICENSE.txt';Path=(Join-Path $repoSp 'LICENSE')}
)) {
  if (!(Test-Path $pair.Path)) { throw "Missing upstream license $($pair.Path)" }
  Copy-Item $pair.Path (Join-Path $lic $pair.Name)
}
$share = Join-Path $vcpkg 'share'
Get-ChildItem $share -Directory | ForEach-Object {
  $copyright = Join-Path $_.FullName 'copyright'
  if (Test-Path $copyright) {
    Copy-Item $copyright (Join-Path $lic "$($_.Name)-copyright.txt")
  }
}
Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/4.1.0/LICENSE' -OutFile (Join-Path $lic 'TessdataFast-LICENSE.txt')
Copy-Item -LiteralPath 'RELEASE_NOTES.md' -Destination (Join-Path $dist 'README-RELEASE.md')
@(
  "ClipLingo $Version - Windows 11 x64 portable beta"
  'Model installation: Settings > Language Models > Install verified model pack'
  "Tesseract traineddata: tessdata_fast tag 4.1.0"
  "Tesseract traineddata SHA256: $((Get-FileHash (Join-Path $tess 'eng.traineddata') -Algorithm SHA256).Hash)"
  "The executable and runtime DLLs are unsigned preview artifacts."
  'See README-RELEASE.md and THIRD_PARTY_NOTICES for details.'
) | Set-Content -Encoding UTF8 (Join-Path $dist 'PACKAGE-MANIFEST.txt')

# A ZIP is a release asset only after building and testing against its own
# extracted contents. No stub-inference builds are accepted.
$archive = Join-Path $OutputFolder "ClipLingo-$Version-win-x64.zip"
if (Test-Path $archive) { Remove-Item $archive -Force }
Compress-Archive -Path "$dist\*" -DestinationPath $archive -CompressionLevel Optimal
if (!(Test-Path $archive) -or (Get-Item $archive).Length -lt 1000000) {
  throw 'Release archive is missing or suspiciously small'
}
Write-Host "Prepared $archive"
