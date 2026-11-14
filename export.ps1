# export.ps1 - Static web export for GitHub Pages (Brickhollow, Kaigen/HZ engine)
# Builds the single-threaded web bundle (the engine's mobile web profile: no SharedArrayBuffer, so it runs without
# the COOP/COEP headers GitHub Pages cannot send) and copies it into docs/ for
# https://mtothexmax.github.io/pokemon-clone/.
#
# GitHub Pages serves files as they are, with no Content-Encoding: br. Where the engine requests cooked/X.br it
# relies on the server to decode it, so docs/ also holds the raw bytes under the .br name; game.wasm goes out
# uncompressed (Pages gzips it on the fly). All paths are relative: any sub path works.
#
# Usage (from the project root):
#   powershell -ExecutionPolicy Bypass -File .\export.ps1
#   .\export.ps1 -NoBuild      # reuse the last publish build in out/
param(
    [switch]$NoBuild
)

$ErrorActionPreference = 'Stop'
Set-Location -Path $PSScriptRoot

Write-Host "=== Brickhollow export for GitHub Pages ===" -ForegroundColor Cyan
Write-Host "Output: docs/"
Write-Host ""

# 1. Build the single-threaded web bundle
if (-not $NoBuild) {
    Write-Host "[1/4] Building the web bundle (hzbuild wasm ios publish)..." -ForegroundColor Yellow
    & .\hz\hzbuild wasm ios publish
    if ($LASTEXITCODE -ne 0) { throw "hzbuild publish failed (exit $LASTEXITCODE)" }
} else {
    Write-Host "[1/4] Skipping the build (-NoBuild)" -ForegroundColor Green
}

$bundle = Get-ChildItem 'out/wasm-ios/publish' -Recurse -Directory -Filter '*-web-single' |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $bundle) { throw "No web bundle found under out/wasm-ios/publish - run without -NoBuild" }
Write-Host "      bundle: $($bundle.FullName)"

# 2. Fresh docs/
Write-Host "[2/4] Copying the bundle into docs/ (raw files, raw copies under the .br names, no _headers or dev index.html)..." -ForegroundColor Yellow
if (Test-Path 'docs') { Remove-Item 'docs' -Recurse -Force }
New-Item -ItemType Directory -Path 'docs' | Out-Null
$src = $bundle.FullName
# The engine reads some asset TYPES through their brotli sibling (X.br: clips, scene chunks, textures) and expects
# the server to decode it (Content-Encoding: br). GitHub Pages cannot, so those files go out twice: the raw X, and
# the same raw bytes under the X.br name. Font pages (cooked/engine/fonts, most of the size) are read raw: no copy.
$skipTop = @('_headers', 'index.html', 'game.wasm.br')
$fonts = [System.IO.Path]::GetFullPath((Join-Path $src 'cooked/engine/fonts'))
Get-ChildItem $src -Recurse -File | Where-Object {
    -not ($_.DirectoryName -eq $src -and $skipTop -contains $_.Name) -and $_.Extension -ne '.br'
} | ForEach-Object {
    $rel = $_.FullName.Substring($src.Length + 1)
    $dest = Join-Path 'docs' $rel
    $dir = Split-Path $dest -Parent
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
    Copy-Item -LiteralPath $_.FullName $dest
    if ((Test-Path -LiteralPath ($_.FullName + '.br')) -and -not $_.FullName.StartsWith($fonts)) {
        Copy-Item -LiteralPath $_.FullName ($dest + '.br')
    }
}

# 3. game.wasm uncompressed (Node's built-in brotli), and our own start page
Write-Host "[3/4] Unpacking game.wasm and writing the start page..." -ForegroundColor Yellow
$wasmBr = Join-Path $src 'game.wasm.br'
$wasmOut = (Join-Path (Resolve-Path 'docs') 'game.wasm')
& node -e "const z=require('zlib'),fs=require('fs');fs.writeFileSync(process.argv[2],z.brotliDecompressSync(fs.readFileSync(process.argv[1])))" $wasmBr $wasmOut
if ($LASTEXITCODE -ne 0) { throw "Could not decompress game.wasm.br (is Node installed?)" }
Copy-Item 'web_export/index.html' 'docs/index.html' -Force

# 4. Disable Jekyll (it would hide files and folders starting with _ or .)
Write-Host "[4/4] Writing .nojekyll" -ForegroundColor Yellow
New-Item -ItemType File -Path 'docs/.nojekyll' -Force | Out-Null

$files = Get-ChildItem 'docs' -Recurse -File
$size = ($files | Measure-Object -Property Length -Sum).Sum / 1MB
$big = $files | Where-Object { $_.Length -gt 95MB }
Write-Host ""
Write-Host ("Done: {0} files, {1:N1} MB in docs/" -f $files.Count, $size) -ForegroundColor Green
if ($big) { Write-Host "Warning: files over 95 MB (GitHub rejects > 100 MB): $($big.Name -join ', ')" -ForegroundColor Red }
Write-Host "Next: git add docs; git commit; git push  (Pages: main branch, /docs folder)"
