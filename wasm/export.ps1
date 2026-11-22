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
# cargo writes its progress to stderr: under 'Stop' a redirected stderr line would end the script
$ErrorActionPreference = 'Continue'
node C:/dev/wasmOnly/tools/dev.mjs build --game-dir $game --release
$ErrorActionPreference = 'Stop'
if ($LASTEXITCODE -ne 0) { throw "release build failed" }
$wasm = Join-Path $game 'dist/pkg/game_bg.wasm'
if ((Get-Item $wasm).Length -gt 50MB) { throw "dist/ holds a debug build ($wasm)" }
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Force $Out | Out-Null
Copy-Item -Recurse -Force (Join-Path $game 'dist/*') $Out
# no Jekyll: serve every file as it is
New-Item -ItemType File -Force (Join-Path $Out '.nojekyll') | Out-Null
# the site's earlier (Kaigen) export registered a service worker at coi-sw.js in its visitors' browsers: this one takes
# its place once and unregisters itself (the pages it controlled reload without it)
$sw = @'
self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.registration.unregister().then(() => self.clients.matchAll({ type: "window" })).then((cs) => cs.forEach((c) => c.navigate(c.url)))));
'@
[IO.File]::WriteAllText((Join-Path $Out 'coi-sw.js'), $sw)
if (-not (Test-Path (Join-Path $Out 'coi-sw.js'))) { throw "coi-sw.js not written" }
$mb = [math]::Round(((Get-ChildItem -Recurse -File $Out | Measure-Object Length -Sum).Sum) / 1MB, 1)
Write-Host "exported to $Out ($mb MB)" -ForegroundColor Green
