# export.ps1 - Static web export for GitHub Pages (Brickhollow, Kaigen/HZ engine)
# Builds two web bundles and copies them into docs/ for https://mtothexmax.github.io/pokemon-clone/:
# - the threaded build at the root (desktop and Android): the only one with sound. It needs SharedArrayBuffer, which
#   needs COOP/COEP headers; GitHub Pages cannot send them, so coi-sw.js (a service worker) adds them in the browser
# - the single-threaded build in docs/ios/ (just its scripts and game.wasm) for iPhones and iPads, whose Safari does
#   not run the threaded one. It reads the shared cooked/ at the root (the threaded cook is a superset of its own)
# index.html picks one.
#
# GitHub Pages serves files as they are, with no Content-Encoding: br. Where the engine requests cooked/X.br it
# relies on the server to decode it, so docs/ also holds the raw bytes under the .br name; game.wasm goes out
# uncompressed (Pages gzips it on the fly). All paths are relative: any sub path works.
#
# Usage (from the project root):
#   powershell -ExecutionPolicy Bypass -File .\export.ps1
#   .\export.ps1 -NoBuild      # reuse the last publish builds in out/
#   .\export.ps1 -Local        # to out/web-local/ instead (git-ignored): for playing over Wi-Fi with
#                              tools/serve_docs.py, without touching docs/ (only for GitHub Pages releases)
param(
    [switch]$NoBuild,
    [switch]$Local
)
$Out = if ($Local) { 'out/web-local' } else { 'docs' }

$ErrorActionPreference = 'Stop'
Set-Location -Path $PSScriptRoot

Write-Host "=== Brickhollow export for GitHub Pages ===" -ForegroundColor Cyan
Write-Host "Output: $Out/"
Write-Host ""

# 1. Build both web bundles
if (-not $NoBuild) {
    Write-Host "[1/5] Building the threaded web bundle (hzbuild wasm publish)..." -ForegroundColor Yellow
    & .\hz\hzbuild wasm publish
    if ($LASTEXITCODE -ne 0) { throw "hzbuild wasm publish failed (exit $LASTEXITCODE)" }
    Write-Host "      and the single-threaded one for iOS (hzbuild wasm ios publish)..." -ForegroundColor Yellow
    & .\hz\hzbuild wasm ios publish
    if ($LASTEXITCODE -ne 0) { throw "hzbuild wasm ios publish failed (exit $LASTEXITCODE)" }
} else {
    Write-Host "[1/5] Skipping the builds (-NoBuild)" -ForegroundColor Green
}

function Get-Bundle($root, $filter) {
    $b = Get-ChildItem $root -Recurse -Directory -Filter $filter | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $b) { throw "No web bundle $filter under $root - run without -NoBuild" }
    return $b.FullName
}
$src = Get-Bundle 'out/wasm/publish' '*-web'
$ios = Get-Bundle 'out/wasm-ios/publish' '*-web-single'
Write-Host "      threaded: $src"
Write-Host "      ios:      $ios"

function Expand-Wasm($from, $to) {
    & node -e "const z=require('zlib'),fs=require('fs');fs.writeFileSync(process.argv[2],z.brotliDecompressSync(fs.readFileSync(process.argv[1])))" $from $to
    if ($LASTEXITCODE -ne 0) { throw "Could not decompress $from (is Node installed?)" }
}

# 2. Fresh docs/ with the threaded bundle
Write-Host "[2/5] Copying the threaded bundle into docs/ (raw files, raw copies under the .br names)..." -ForegroundColor Yellow
if (Test-Path $Out) { Remove-Item $Out -Recurse -Force }
New-Item -ItemType Directory -Path $Out -Force | Out-Null
# The engine reads some asset TYPES through their brotli sibling (X.br: clips, scene chunks, textures) and expects
# the server to decode it (Content-Encoding: br). GitHub Pages cannot, so those files go out twice: the raw X, and
# the same raw bytes under the X.br name. Font pages (cooked/engine/fonts, most of the size) are read raw: no copy.
$skipTop = @('_headers', 'index.html', 'game.wasm.br')
$fonts = [System.IO.Path]::GetFullPath((Join-Path $src 'cooked/engine/fonts'))
Get-ChildItem $src -Recurse -File | Where-Object {
    -not ($_.DirectoryName -eq $src -and $skipTop -contains $_.Name) -and $_.Extension -ne '.br'
} | ForEach-Object {
    $rel = $_.FullName.Substring($src.Length + 1)
    $dest = Join-Path $Out $rel
    $dir = Split-Path $dest -Parent
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    Copy-Item -LiteralPath $_.FullName $dest
    if ((Test-Path -LiteralPath ($_.FullName + '.br')) -and -not $_.FullName.StartsWith($fonts)) {
        Copy-Item -LiteralPath $_.FullName ($dest + '.br')
    }
}
Expand-Wasm (Join-Path $src 'game.wasm.br') (Join-Path (Resolve-Path $Out) 'game.wasm')

# 3. The iOS bundle: its scripts and game.wasm only (cooked/ is shared)
Write-Host "[3/5] Copying the single-threaded iOS scripts into docs/ios/..." -ForegroundColor Yellow
New-Item -ItemType Directory -Path "$Out/ios" | Out-Null
Get-ChildItem $ios -File -Filter '*.js' | ForEach-Object { Copy-Item -LiteralPath $_.FullName (Join-Path "$Out/ios" $_.Name) }
Expand-Wasm (Join-Path $ios 'game.wasm.br') (Join-Path (Resolve-Path "$Out/ios") 'game.wasm')

# 4. Our start page and the service worker that adds the isolation headers
Write-Host "[4/5] Writing the start page and coi-sw.js..." -ForegroundColor Yellow
Copy-Item 'web_export/index.html' "$Out/index.html" -Force
Copy-Item 'web_export/coi-sw.js' "$Out/coi-sw.js" -Force

# 5. Disable Jekyll (it would hide files and folders starting with _ or .)
Write-Host "[5/5] Writing .nojekyll" -ForegroundColor Yellow
New-Item -ItemType File -Path "$Out/.nojekyll" -Force | Out-Null

$files = Get-ChildItem $Out -Recurse -File
$size = ($files | Measure-Object -Property Length -Sum).Sum / 1MB
$big = $files | Where-Object { $_.Length -gt 95MB }
Write-Host ""
Write-Host ("Done: {0} files, {1:N1} MB in {2}/" -f $files.Count, $size, $Out) -ForegroundColor Green
if ($big) { Write-Host "Warning: files over 95 MB (GitHub rejects > 100 MB): $($big.Name -join ', ')" -ForegroundColor Red }
if ($Local) { Write-Host "Play it: python tools/serve_docs.py [--lan]  (serves out/web-local/ when it exists)" }
else { Write-Host "Next: git add docs; git commit; git push  (Pages: main branch, /docs folder)" }
