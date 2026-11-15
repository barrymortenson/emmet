# SPDX-License-Identifier: AGPL-3.0-or-later
# Installs, reinstalls, and uninstalls the NSIS installer on this Windows machine, and checks what each step leaves
# (plan session 5B): a per-user install that touches nothing machine-wide, an install over an existing one that keeps
# the user's data, and an uninstall that removes the app and keeps the data. The uninstaller's "Delete the application
# data" box is the only way to remove the data, and only a person can tick it (docs/security/windows-pass.md).
# CI's nsis job runs this on the installer `just release-dry` built. Usage: pwsh scripts/nsis-check.ps1 <installer.exe>
param([Parameter(Mandatory)][string]$Installer)
$ErrorActionPreference = "Stop"

function Check([bool]$ok, [string]$what) {
  if (-not $ok) { throw "nsis-check: $what" }
  Write-Host "ok: $what"
}

$installer = (Resolve-Path $Installer).Path
$dir = Join-Path $env:LOCALAPPDATA "Emmet"
$app = Join-Path $dir "Emmet.exe"
$userKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Emmet"
$machineKeys = "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Emmet", "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Emmet"
$data = Join-Path $env:APPDATA "com.emmetbuilder.emmet"
$kept = Join-Path $data "nsis-check-keep.txt"

function Install([string]$step) {
  $p = Start-Process -FilePath $installer -ArgumentList "/S" -Wait -PassThru
  Check ($p.ExitCode -eq 0) "$step`: the installer exits 0 (it gave $($p.ExitCode))"
  Check (Test-Path $app) "$step`: the app is in $dir"
  foreach ($sidecar in "emmet.exe", "uv.exe") { Check (Test-Path (Join-Path $dir $sidecar)) "$step`: $sidecar is beside it" }
  Check (Test-Path $userKey) "$step`: it's listed under this user's installed apps"
  foreach ($k in $machineKeys) { Check (-not (Test-Path $k)) "$step`: nothing under $k" }
  foreach ($root in $env:ProgramFiles, ${env:ProgramFiles(x86)}) { Check (-not (Test-Path (Join-Path $root "Emmet"))) "$step`: nothing in $root" }
}

Check (-not (Test-Path $app)) "the machine starts without Emmet"
Install "install"
$version = (& (Join-Path $dir "emmet.exe") --version) -join " "
Check ($LASTEXITCODE -eq 0 -and $version -match "\d+\.\d+\.\d+") "the installed emmet runs and says its version ($version)"
Check ((Get-ItemProperty $userKey).DisplayVersion -match "^\d+\.\d+\.\d+") "the installed apps list shows a version"

# The user's data: the library, Python, settings, and the trust secret live here.
New-Item -ItemType Directory -Force $data | Out-Null
Set-Content -Path $kept -Value "kept"

Install "install over itself"
Check (Test-Path $kept) "install over itself: the app data is still there"

$uninstaller = Join-Path $dir "uninstall.exe"
Check (Test-Path $uninstaller) "the uninstaller is in $dir"
# The uninstaller copies itself to a temp folder and returns at once, so wait for the app to go.
Start-Process -FilePath $uninstaller -ArgumentList "/S" -Wait
$deadline = (Get-Date).AddSeconds(90)
while ((Test-Path $app) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
Check (-not (Test-Path $app)) "uninstall: the app is gone"
while ((Test-Path $userKey) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 500 }
Check (-not (Test-Path $userKey)) "uninstall: it's off this user's installed apps"
Check (Test-Path $kept) "uninstall: the app data is kept"
Remove-Item $kept
Write-Host "nsis-check: ok"
