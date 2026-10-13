@echo off
setlocal EnableExtensions

rem =====================================================================
rem  Open a build terminal with Concat's libraries on PATH
rem
rem  DOUBLE-CLICK this file. A Command Prompt window opens and stays open.
rem  Run your cargo commands IN THAT WINDOW.
rem
rem  Why this exists: fetch-native-deps.bat sets five environment
rem  variables permanently, but it deliberately does NOT touch PATH if
rem  your PATH is already long, because setx truncates at 1024 characters
rem  and losing the tail of somebody's PATH to save four entries is a bad
rem  trade. This file is the alternative - it puts the four DLL folders on
rem  PATH for ONE session and changes nothing on disk.
rem
rem  Use it whenever a test binary cannot find a DLL, or just use it every
rem  time and never think about PATH again.
rem =====================================================================

title Concat build terminal

set "DEPS=C:\Concat-deps"

rem The five variables, set for this window only.
set "FFMPEG_DIR=%DEPS%\ffmpeg"
set "ORT_LIB_LOCATION=%DEPS%\ort\lib"
set "ORT_PREFER_DYNAMIC_LINK=1"
set "SHERPA_ONNX_LIB_DIR=%DEPS%\sherpa-libs"

rem LLVM, found the same way the other files find it.
set "LIBCLANG="
for /f "delims=" %%i in ('where libclang.dll 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do set "LIBCLANG=%%~fd"
if not defined LIBCLANG for %%d in ("C:\Program Files\LLVM" "C:\Program Files (x86)\LLVM" "C:\llvm" "%LOCALAPPDATA%\Programs\LLVM" "C:\Tools\llvm") do if exist "%%~d\bin\libclang.dll" set "LIBCLANG=%%~d\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKCU\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if not defined LIBCLANG for /f "delims=" %%i in ('where clang 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do if exist "%%~fd\libclang.dll" set "LIBCLANG=%%~fd"
rem Last resort: the folder may hold libclang under a version suffix
if not defined LIBCLANG for %%d in ("C:\Program Files\LLVM\bin" "C:\llvm\bin") do if exist "%%~d\libclang.dll" set "LIBCLANG=%%~d"
if defined LIBCLANG set "LIBCLANG_PATH=%LIBCLANG%"

rem The four folders whose DLLs the test binaries load at run time.
set "PATH=%DEPS%\ffmpeg\bin;%DEPS%\ort\lib;%DEPS%\sherpa-dlls;%LIBCLANG_PATH%;%PATH%"

rem The Concat source, so the window opens there.
set "SRC=C:\dev\concat\src"
if not exist "%SRC%" set "SRC=%USERPROFILE%"

echo  ================================================================
echo   Concat build terminal - this window only, nothing saved
echo  ================================================================
echo.
echo   FFMPEG_DIR              = %FFMPEG_DIR%
echo   ORT_LIB_LOCATION        = %ORT_LIB_LOCATION%
echo   SHERPA_ONNX_LIB_DIR     = %SHERPA_ONNX_LIB_DIR%
echo   LIBCLANG_PATH           = %LIBCLANG_PATH%
echo.

rem Say plainly if something is missing, because a build that fails for
rem want of a folder is much harder to diagnose than this message.
set "MISSING="
if not exist "%FFMPEG_DIR%\lib\avcodec.lib"          set "MISSING=%MISSING% FFmpeg"
if not exist "%ORT_LIB_LOCATION%\onnxruntime.lib"    set "MISSING=%MISSING% ONNXRuntime"
if not exist "%SHERPA_ONNX_LIB_DIR%"                 set "MISSING=%MISSING% sherpa-onnx"
if not defined LIBCLANG_PATH                         set "MISSING=%MISSING% LLVM"

if defined MISSING (
    echo   [WARNING] not found:%MISSING%
    echo.
    echo   Run fetch-native-deps.bat first, or install-build-tools.bat if
    echo   LLVM is the missing one. This window still works for anything
    echo   that does not need them - cargo test -p concat-project, for
    echo   example, needs none of the four.
    echo.
) else (
    echo   [ok] all four libraries are where they should be
    echo.
)

if exist "%SRC%" (
    echo   Current folder: %SRC%
    cd /d "%SRC%"
) else (
    echo   C:\dev\concat\src is not there, so this window opened in your
    echo   home folder. cd to your Concat source folder before building.
)

echo.
echo  ================================================================
echo   Commands to try, ONE line at a time, in this window:
echo.
echo       cargo test -p concat-project
echo       cargo build -p concat-media
echo       cargo dev
echo.
echo   The first needs none of the libraries above and proves the Rust
echo   toolchain. Expect "114 passed" on an untouched main, "215 passed"
echo   with the Tranche A patch applied.
echo.
echo   The first FULL build is long - CI budgets 60 minutes for it with
echo   two parallel jobs. Scrolling output means it is working. If rustc
echo   is killed or the PC freezes, that is the 16 GB of RAM, not a code
echo   error: retry with  cargo dev -- -j 2
echo.
echo   Keep this window open. Closing it discards everything above.
echo  ================================================================
echo.
cmd /k
