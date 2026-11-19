# Release build of Brickhollow for the phone: builds with --release and copies dist/ to out/lan/ (git-ignored),
# which tools/serve_lan.py serves over HTTPS in the Wi-Fi. dist/ is then rebuilt by the dev watcher on the next change.
$ErrorActionPreference = 'Stop'
$game = Split-Path -Parent $PSScriptRoot
node C:/dev/wasmOnly/tools/dev.mjs build --game-dir $game --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$out = Join-Path $game 'out/lan'
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item -Recurse -Force (Join-Path $game 'dist/*') $out
Write-Output "copied to $out"
