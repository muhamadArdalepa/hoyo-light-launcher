$WshShell = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath('Desktop')
$shortcutPath = Join-Path $desktop "Hoyo Activity.lnk"
$targetPath = Join-Path $PSScriptRoot "hoyo-last-play.exe"

# Remove old shortcut if exists
$oldShortcut = Join-Path $desktop "HoYoPlay Activity Tracker.lnk"
if (Test-Path $oldShortcut) {
    Remove-Item $oldShortcut -Force
}

$shortcut = $WshShell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $targetPath
$shortcut.WorkingDirectory = $PSScriptRoot
$shortcut.IconLocation = "$targetPath,0"
$shortcut.Description = "Hoyo Activity Tracker & Game Launcher (Rust + Tauri)"
$shortcut.Save()

Write-Host "Updated desktop shortcut: $shortcutPath" -ForegroundColor Green
