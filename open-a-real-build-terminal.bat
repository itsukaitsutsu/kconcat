@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem =====================================================================
rem  Concat build terminal - WITH the MSVC environment
rem
rem  DOUBLE-CLICK this file. Run cargo commands IN THE WINDOW IT OPENS.
rem
rem  Why this file exists instead of open-a-build-terminal.bat:
rem
rem  That file sets Concat's five variables and puts the DLL folders on
rem  PATH, then opens a plain Command Prompt. It never calls vcvars64.bat,
rem  so the MSVC include directories are missing from the environment.
rem  ffmpeg-sys-the-third runs libclang over FFmpeg's headers, and
rem  channel_layout.h begins with #include <stdlib.h>. libclang has no
rem  idea where the MSVC and Windows SDK headers live unless INCLUDE says
rem  so, and dies with:
rem
rem      fatal error: 'stdlib.h' file not found
rem      Unable to generate bindings: ClangDiagnostic(...)
rem
rem  vcvars64.bat is what sets INCLUDE and LIB. This file calls it first,
rem  then sets Concat's five variables on top, then opens the window at
rem  the Concat source folder - found by searching for src\Cargo.toml
rem  rather than hardcoded to C:\dev\concat\src, which is not where this
rem  checkout lives.
rem
rem  Changes nothing on disk. Every variable here is for this window only.
rem =====================================================================

title Concat build terminal (MSVC)

rem =====================================================================
rem 1. MSVC. vcvars64.bat must run FIRST: it prepends to PATH, and the
rem    Concat DLL folders are added after it so they win.
rem =====================================================================
set "VCVARS="

rem (a) vswhere is the supported way to ask where Visual Studio lives
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if exist "%VSWHERE%" for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2^>nul`) do if not defined VCVARS if exist "%%~i\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%~i\VC\Auxiliary\Build\vcvars64.bat"

rem (b) the registry, which records a custom install drive. Visual Studio
rem     is not always on C: - it lands wherever the installer was told to
rem     put it, and a hardcoded list of C:\Program Files folders misses
rem     every one of those.
if not defined VCVARS for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\SxS\VS7" /v 17.0 2^>nul ^| findstr REG_SZ') do if exist "%%b\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%b\VC\Auxiliary\Build\vcvars64.bat"
if not defined VCVARS for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\VisualStudio\SxS\VS7" /v 17.0 2^>nul ^| findstr REG_SZ') do if exist "%%b\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%b\VC\Auxiliary\Build\vcvars64.bat"
if not defined VCVARS for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\SxS\VS7" /v 16.0 2^>nul ^| findstr REG_SZ') do if exist "%%b\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%b\VC\Auxiliary\Build\vcvars64.bat"

rem (c) the folders the installers use by default, plus any edition of VS
rem     sitting at the root of a drive, which is where a custom install
rem     drive tends to put it.
if not defined VCVARS for %%d in ("C:\Program Files\Microsoft Visual Studio\2022\BuildTools" "C:\Program Files\Microsoft Visual Studio\2022\Community" "C:\Program Files\Microsoft Visual Studio\2022\Professional" "C:\Program Files\Microsoft Visual Studio\2022\Enterprise" "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools" "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community") do if exist "%%~d\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%~d\VC\Auxiliary\Build\vcvars64.bat"
if not defined VCVARS for %%d in ("D:\VStudio" "E:\VStudio" "D:\VisualStudio" "C:\VStudio") do if exist "%%~d\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%~d\VC\Auxiliary\Build\vcvars64.bat"
if not defined VCVARS for %%e in (C D E F) do if not defined VCVARS for /d %%d in ("%%e:\*") do if not defined VCVARS if exist "%%~fd\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%%~fd\VC\Auxiliary\Build\vcvars64.bat"

if not defined VCVARS (
    echo.
    echo  [ERROR] vcvars64.bat is nowhere this file can see, so the MSVC
    echo          C++ toolchain is not installed - or it is installed
    echo          without the "Desktop development with C++" workload.
    echo.
    echo          Double-click install-build-tools.bat, let it finish,
    echo          then double-click this file again.
    echo.
    pause
    exit /b 1
)

echo.
echo  ---- MSVC -------------------------------------------------------
echo   calling: %VCVARS%
call "%VCVARS%" >nul
if errorlevel 1 (
    echo  [ERROR] vcvars64.bat returned an error. Its own message is above.
    pause
    exit /b 1
)

rem The proof it worked: INCLUDE is what libclang reads to find stdlib.h.
if not defined INCLUDE (
    echo  [ERROR] vcvars64.bat ran but INCLUDE is still empty, so there is
    echo          no Windows SDK in this installation. Re-run
    echo          install-build-tools.bat and approve every UAC prompt.
    pause
    exit /b 1
)
where cl.exe >nul 2>nul
if errorlevel 1 (
    echo  [ERROR] cl.exe is not on PATH even after vcvars64.bat.
    pause
    exit /b 1
)
echo   [ok] cl.exe found, INCLUDE is set

rem =====================================================================
rem 2. Concat's five variables
rem =====================================================================
set "DEPS=C:\Concat-deps"

set "FFMPEG_DIR=%DEPS%\ffmpeg"
set "ORT_LIB_LOCATION=%DEPS%\ort\lib"
set "ORT_PREFER_DYNAMIC_LINK=1"
set "SHERPA_ONNX_LIB_DIR=%DEPS%\sherpa-libs"

rem LLVM, found the same way the repository's other files find it.
rem LIBCLANG is the folder that CONTAINS libclang.dll, which is exactly
rem what LIBCLANG_PATH wants - no \bin arithmetic, so nothing doubles up.
set "LIBCLANG="
for /f "delims=" %%i in ('where libclang.dll 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do set "LIBCLANG=%%~fd"
if not defined LIBCLANG for %%d in ("C:\llvm" "C:\Program Files\LLVM" "C:\Program Files (x86)\LLVM" "%LOCALAPPDATA%\Programs\LLVM" "C:\Tools\llvm") do if exist "%%~d\bin\libclang.dll" set "LIBCLANG=%%~d\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKCU\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if defined LIBCLANG set "LIBCLANG_PATH=%LIBCLANG%"

echo.
echo  ---- Concat ------------------------------------------------------
echo   FFMPEG_DIR              = %FFMPEG_DIR%
echo   ORT_LIB_LOCATION        = %ORT_LIB_LOCATION%
echo   SHERPA_ONNX_LIB_DIR     = %SHERPA_ONNX_LIB_DIR%
echo   LIBCLANG_PATH           = %LIBCLANG_PATH%

rem =====================================================================
rem 3. PATH: the four DLL folders, AFTER vcvars so they are not buried
rem =====================================================================
set "PATH=%FFMPEG_DIR%\bin;%ORT_LIB_LOCATION%;%DEPS%\sherpa-dlls;%LIBCLANG_PATH%;%PATH%"

rem =====================================================================
rem 4. Say plainly if something is missing
rem =====================================================================
set "MISSING="
if not exist "%FFMPEG_DIR%\include\libavcodec\avcodec.h" set "MISSING=%MISSING% FFmpeg-headers"
if not exist "%FFMPEG_DIR%\lib\avcodec.lib"              set "MISSING=%MISSING% FFmpeg-libs"
if not exist "%ORT_LIB_LOCATION%\onnxruntime.lib"        set "MISSING=%MISSING% ONNXRuntime"
if not exist "%SHERPA_ONNX_LIB_DIR%"                     set "MISSING=%MISSING% sherpa-onnx"
if not defined LIBCLANG_PATH                             set "MISSING=%MISSING% LLVM"

echo.
if defined MISSING (
    echo   [WARNING] not found:%MISSING%
    echo             Run fetch-native-deps.bat, then this file again.
) else (
    echo   [ok] all five libraries are where they should be
)

rem =====================================================================
rem 5. The Concat source folder, found rather than guessed
rem =====================================================================
set "SRC="
if exist "%~dp0src\Cargo.toml" set "SRC=%~dp0src"
if not defined SRC if exist "C:\dev\concat\src\Cargo.toml"  set "SRC=C:\dev\concat\src"
if not defined SRC if exist "C:\dev\kconcat\src\Cargo.toml" set "SRC=C:\dev\kconcat\src"
if not defined SRC for /d %%d in ("%~dp0*") do if not defined SRC if exist "%%~fd\src\Cargo.toml" set "SRC=%%~fd\src"
if not defined SRC for /d %%d in ("C:\dev\*") do if not defined SRC if exist "%%~fd\src\Cargo.toml" set "SRC=%%~fd\src"

echo.
if defined SRC (
    for %%f in ("%SRC%") do set "SRC=%%~ff"
    echo   Current folder: !SRC!
    cd /d "!SRC!"
) else (
    echo   No folder held src\Cargo.toml, so this window opened in your
    echo   home folder. cd to your Concat source folder before building.
)

echo.
echo  ================================================================
echo   Commands to try, ONE line at a time, in this window:
echo.
echo       cargo test -p concat-project      no FFmpeg, proves Rust
echo       cargo build -p concat-media       FFmpeg only, the real test
echo       cargo dev                         the editor window
echo.
echo   The first full build is long - CI budgets 60 minutes for it.
echo   Scrolling output means it is working. If rustc is killed or the
echo   PC freezes, that is the 16 GB of RAM, not a code error: retry
echo   with  cargo dev -- -j 2
echo.
echo   Keep this window open. Closing it discards everything above.
echo  ================================================================
echo.
cmd /k
