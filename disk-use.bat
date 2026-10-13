@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem =====================================================================
rem  Where did the disk go? Changes NOTHING - only measures and prints.
rem
rem  DOUBLE-CLICK. Takes a minute or two: it walks every folder below.
rem  Copy the whole output back if you want it read.
rem =====================================================================

title Disk use - Concat and friends

echo.
echo  ================================================================
echo   DISK USE - nothing here is changed, only measured
echo  ================================================================
echo.

echo  ---- drives ----------------------------------------------------
for %%d in (C D E F) do if exist "%%d:\" (
    for /f "tokens=3,4" %%a in ('dir %%d:\ 2^>nul ^| findstr /c:"bytes"') do (
        echo   %%d:  free %%a   total %%b
    )
)
echo.

echo  ---- the usual suspects, biggest first -------------------------
for %%p in (
    "C:\dev\kconcat\src\target"
    "C:\dev\kconcat"
    "C:\Concat-deps"
    "%USERPROFILE%\.cargo"
    "%LOCALAPPDATA%\app.concat.editor"
    "%APPDATA%\app.concat.editor"
    "%TEMP%\concat-tools"
    "C:\llvm"
) do (
    if exist "%%~p" (
        for /f %%s in ('powershell -NoProfile -Command "'{0:N0}' -f ((Get-ChildItem -LiteralPath '%%~p' -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum)" 2^>nul') do (
            echo   %%s bytes   %%~p
        )
    ) else (
        echo   absent            %%~p
    )
)
echo.

echo  ---- target\ split by profile ----------------------------------
echo   Each profile is a separate full copy of every dependency.
if exist "C:\dev\kconcat\src\target" (
    for /d %%d in ("C:\dev\kconcat\src\target\*") do (
        for /f %%s in ('powershell -NoProfile -Command "'{0:N0}' -f ((Get-ChildItem -LiteralPath '%%~fd' -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum)" 2^>nul') do (
            echo   %%s bytes   %%~nxd
        )
    )
) else (
    echo   there is no target folder
)
echo.

echo  ---- models, which are downloaded and can be gigabytes ---------
if exist "%LOCALAPPDATA%\app.concat.editor" (
    for /d %%d in ("%LOCALAPPDATA%\app.concat.editor\*") do (
        for /f %%s in ('powershell -NoProfile -Command "'{0:N0}' -f ((Get-ChildItem -LiteralPath '%%~fd' -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum)" 2^>nul') do (
            echo   %%s bytes   %%~nxd
        )
    )
) else (
    echo   the app has made no data folder yet
)
echo.

echo  ================================================================
echo   HOW TO READ THIS
echo  ================================================================
echo.
echo   target\        build output. Deleting it is SAFE - it is rebuilt
echo                  from source. This is nearly always the 22 GB.
echo   .cargo         downloaded crates and the toolchains. Safe to clear,
echo                  re-downloads on the next build.
echo   Concat-deps    FFmpeg, ONNX Runtime, sherpa-onnx. DO NOT delete -
echo                  that is a 10-minute re-download and the build needs it.
echo   app.concat.editor   settings, models, effect cards. The models are
echo                  big and re-downloadable; the settings are not.
echo   Temp\concat-tools   the installers install-build-tools.bat fetched.
echo                  Safe, and large - LLVM alone is ~350 MB.
echo.
echo   Run reclaim-disk.bat to delete the safe ones one at a time.
echo  ================================================================
echo.
pause
