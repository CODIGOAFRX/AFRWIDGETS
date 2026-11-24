# Compila AFRWIDGETS, le pone icono y manifiesto, genera los renders y el instalador.
# Uso: powershell -ExecutionPolicy Bypass -File scripts/build.ps1
# Cliente OAuth de Google: se toma de AFR_GOOGLE_CLIENT_ID / AFR_GOOGLE_CLIENT_SECRET o, si no están,
# de google-oauth.local.json en la raíz (fuera de git).
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$oauth = Join-Path $root "google-oauth.local.json"
if (-not $env:AFR_GOOGLE_CLIENT_ID -and (Test-Path $oauth)) {
  $o = Get-Content $oauth -Raw | ConvertFrom-Json
  $env:AFR_GOOGLE_CLIENT_ID = $o.client_id
  $env:AFR_GOOGLE_CLIENT_SECRET = $o.client_secret
  "Cliente OAuth de Google: incluido"
}
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
$iscc = Join-Path $env:LOCALAPPDATA "Programs\Inno Setup 6\ISCC.exe"
$rcedit = Join-Path $root "tools\node_modules\rcedit\bin\rcedit-x64.exe"

if (-not (Test-Path $rcedit)) { Push-Location (Join-Path $root "tools"); npm.cmd install rcedit@4 --no-audit --no-fund; Pop-Location }

Push-Location (Join-Path $root "app")
& $cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build ha fallado" }
Pop-Location

$dist = Join-Path $root "dist"
New-Item -ItemType Directory -Force (Join-Path $dist "app") | Out-Null
$exe = Join-Path $dist "app\afrwidgets.exe"
Copy-Item (Join-Path $root "app\target\release\afrwidgets.exe") $exe -Force

& $exe --render-previews (Join-Path $dist "previews") | Out-Host
python (Join-Path $root "scripts\compress_png.py") (Join-Path $dist "previews")
# El antivirus a veces bloquea el exe recién copiado: se reintenta.
for ($i = 0; $i -lt 5; $i++) {
  Start-Sleep -Seconds 2
  & $rcedit $exe --set-icon (Join-Path $dist "previews\afrwidgets.ico") `
    --application-manifest (Join-Path $root "app\afrwidgets.manifest") `
    --set-file-version "1.0.0.0" --set-product-version "1.0.0" `
    --set-version-string "ProductName" "AFRWIDGETS" `
    --set-version-string "FileDescription" "AFRWIDGETS · widgets de escritorio" `
    --set-version-string "CompanyName" "Pedro Gómez" `
    --set-version-string "LegalCopyright" "© 2026 Pedro Gómez · MIT"
  if ($LASTEXITCODE -eq 0) { break }
}
if ($LASTEXITCODE -ne 0) { throw "rcedit ha fallado" }

& $iscc /Q (Join-Path $root "installer\afrwidgets.iss")
if ($LASTEXITCODE -ne 0) { throw "Inno Setup ha fallado" }
Get-ChildItem $dist -Filter "AFRWIDGETS-Setup-*.exe" | ForEach-Object { "{0}  {1:N0} KB" -f $_.Name, ($_.Length / 1KB) }
