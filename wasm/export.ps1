# export.ps1 - Brickhollow (wasm) for GitHub Pages: a release build, dist/ copied to docs/ (or -Out), which Pages
# serves from main at https://mtothexmax.github.io/pokemon-clone/. Every path in the build is relative, so it runs
# under the /pokemon-clone/ base path as it is. dist/ is rebuilt by the dev watcher on the next change.
#
# Usage (from wasm/):
#   powershell -ExecutionPolicy Bypass -File .\export.ps1
#   .\export.ps1 -Out C:\path\to\a\main-checkout\docs
param(
    [string]$Out = (Join-Path (Split-Path -Parent $PSScriptRoot) 'docs')
)
$ErrorActionPreference = 'Stop'
$game = $PSScriptRoot
Write-Host "=== Brickhollow (wasm) export for GitHub Pages -> $Out ===" -ForegroundColor Cyan
node C:/dev/wasmOnly/tools/dev.mjs build --game-dir $game --release
if ($LASTEXITCODE -ne 0) { throw "release build failed" }
$wasm = Join-Path $game 'dist/pkg/game_bg.wasm'
if ((Get-Item $wasm).Length -gt 50MB) { throw "dist/ holds a debug build ($wasm)" }
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Force $Out | Out-Null
Copy-Item -Recurse -Force (Join-Path $game 'dist/*') $Out
# no Jekyll: serve every file as it is
New-Item -ItemType File -Force (Join-Path $Out '.nojekyll') | Out-Null
$mb = [math]::Round(((Get-ChildItem -Recurse -File $Out | Measure-Object Length -Sum).Sum) / 1MB, 1)
Write-Host "exported to $Out ($mb MB)" -ForegroundColor Green
