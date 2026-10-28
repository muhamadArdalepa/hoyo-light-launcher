@echo off
setlocal
echo ==============================================
echo   Building Hoyo Activity (Rust + Tauri)
echo ==============================================

cargo build --manifest-path src-tauri/Cargo.toml --release

if %ERRORLEVEL% EQU 0 (
    copy /Y src-tauri\target\release\hoyo-last-play.exe .\hoyo-last-play.exe
    echo.
    echo ==============================================
    echo   BUILD SUCCESSFUL!
    echo   File generated: hoyo-last-play.exe
    echo ==============================================
) else (
    echo.
    echo BUILD FAILED with error code %ERRORLEVEL%
)

endlocal
