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
# the phone's console goes to out/phone.log (tools/serve_lan.py): load the relay before the game
Copy-Item (Join-Path $PSScriptRoot 'phone_log.js') $out
$index = Join-Path $out 'index.html'
(Get-Content -Raw $index).Replace('<head>', '<head><script src="phone_log.js"></script>') | Set-Content -Encoding utf8 -NoNewline $index
Write-Output "copied to $out"
