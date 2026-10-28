Write-Host "Building Hoyo Activity (Rust + Tauri)..." -ForegroundColor Cyan

cargo build --manifest-path src-tauri/Cargo.toml --release

if ($LASTEXITCODE -eq 0) {
    Copy-Item src-tauri\target\release\hoyo-last-play.exe .\hoyo-last-play.exe -Force
    Write-Host "BUILD SUCCESSFUL! Copied hoyo-last-play.exe to root directory." -ForegroundColor Green
} else {
    Write-Host "BUILD FAILED!" -ForegroundColor Red
}
