@echo off
setlocal EnableExtensions

rem =====================================================================
rem  Concat for Windows - step 1, v2
rem
rem  DOUBLE-CLICK this file.
rem
rem  What changed from v1:
rem    - v1 only looked for LLVM in two hardcoded folders and downloaded
rem      350 MB when it did not find it there. This version searches PATH,
rem      eight common install locations, and the Windows registry.
rem    - v1 had an inverted `if not errorlevel 1` test. Batch's errorlevel
rem      means "the last command's exit code", and `if errorlevel 1` means
rem      "1 or more", i.e. it FAILED. Fixed, and the version checks no
rem      longer depend on that subtlety at all.
rem    - Every download now prints its URL and keeps curl's own error text.
rem    - A DETECTED section lists what was found and where, so a wrong
rem      guess is visible rather than silent.
rem
rem  Safe to re-run. Anything already installed is skipped.
rem =====================================================================

title Concat - installing build tools

set "DL=%TEMP%\concat-tools"
if not exist "%DL%" mkdir "%DL%"

echo.
echo  ================================================================
echo   DETECTED - what is already on this PC
echo  ================================================================

rem ---------------------------------------------------------------------
rem Git
rem ---------------------------------------------------------------------
set "GITEXE="
set "GITVER="
for /f "delims=" %%i in ('where git 2^>nul') do if not defined GITEXE set "GITEXE=%%i"
if defined GITEXE for /f "tokens=3" %%v in ('git --version 2^>nul') do set "GITVER=%%v"
if defined GITEXE (
    echo   Git ............. %GITVER%
    echo                     %GITEXE%
) else (
    echo   Git ............. NOT FOUND
)

rem ---------------------------------------------------------------------
rem CMake. The version is checked in a subprocess so the comparison is
rem not at the mercy of batch's delayed expansion inside a block.
rem ---------------------------------------------------------------------
set "CMAKEEXE="
set "CMAKEVER="
set "CMAKEOK=no"
for /f "delims=" %%i in ('where cmake 2^>nul') do if not defined CMAKEEXE set "CMAKEEXE=%%i"
if defined CMAKEEXE for /f "tokens=3" %%v in ('cmake --version 2^>nul') do set "CMAKEVER=%%v"
if defined CMAKEVER (
    echo %CMAKEVER% | findstr /b /c:"3." >nul 2>&1 && set "CMAKEOK=yes"
)
if defined CMAKEEXE (
    if "%CMAKEOK%"=="yes" (
        echo   CMake ........... %CMAKEVER%   ^(3.x, correct^)
    ) else (
        echo   CMake ........... %CMAKEVER%   ^(WRONG - needs 3.x, 4.x breaks whisper.cpp^)
    )
    echo                     %CMAKEEXE%
) else (
    echo   CMake ........... NOT FOUND
)

rem ---------------------------------------------------------------------
rem LLVM. The one v1 got wrong. Search everywhere it could be.
rem ---------------------------------------------------------------------
rem LIBCLANG is the folder that CONTAINS libclang.dll - which is exactly
rem what LIBCLANG_PATH wants. Earlier versions of these files stored the
rem LLVM root here and appended \bin later, but detector (a) yields the bin
rem folder while (b) and (c) yield the root, so the appended \bin doubled
rem into ...\bin\bin for anyone whose LLVM was found on PATH.
set "LIBCLANG="

rem (a) is libclang.dll on PATH?
for /f "delims=" %%i in ('where libclang.dll 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do set "LIBCLANG=%%~fd"

rem (b) the places installers actually use
if not defined LIBCLANG for %%d in ("C:\Program Files\LLVM" "C:\Program Files (x86)\LLVM" "C:\llvm" "%LOCALAPPDATA%\Programs\LLVM" "%ProgramData%\chocolatey\lib\llvm" "C:\Tools\llvm") do if exist "%%~d\bin\libclang.dll" set "LIBCLANG=%%~d\bin"

rem (c) the registry, which is where LLVM's own installer records itself
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKCU\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"

rem (d) clang itself on PATH; libclang.dll sits beside it
if not defined LIBCLANG for /f "delims=" %%i in ('where clang 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do if exist "%%~fd\libclang.dll" set "LIBCLANG=%%~fd"

if defined LIBCLANG (
    echo   LLVM ............ libclang.dll is in %LIBCLANG%
) else (
    echo   LLVM ............ NOT FOUND   ^(searched PATH, 6 folders, the registry, clang^)
)

rem ---------------------------------------------------------------------
rem VS Build Tools - the C++ compiler
rem ---------------------------------------------------------------------
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "HAVECPP="
if exist "%VSWHERE%" for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2^>nul`) do set "HAVECPP=%%i"
if defined HAVECPP (
    echo   C++ toolchain ... %HAVECPP%
) else (
    echo   C++ toolchain ... NOT FOUND
)

rem ---------------------------------------------------------------------
rem winget
rem ---------------------------------------------------------------------
where winget >nul 2>nul
if %errorlevel%==0 (
    echo   winget .......... installed
) else (
    echo   winget .......... not installed   ^(Concat does not need it^)
)

echo  ================================================================
echo.

rem ---------------------------------------------------------------------
rem What is actually missing
rem ---------------------------------------------------------------------
set "TODO="
if not defined GITEXE   set "TODO=%TODO% git"
if not "%CMAKEOK%"=="yes" set "TODO=%TODO% cmake"
if not defined LIBCLANG set "TODO=%TODO% llvm"
if not defined HAVECPP  set "TODO=%TODO% vsbt"

if not defined TODO (
    echo   Everything Concat needs is already installed.
    echo   Nothing to download. You can skip to STEP-2-AND-3.md.
    echo.
    echo   Press any key to close...
    pause >nul
    goto :eof
)

echo   Still to install:%TODO%
echo.
echo   Approve each UAC prompt when it appears.
echo.
pause

rem =====================================================================
rem Install only what is missing
rem =====================================================================

if defined GITEXE goto :skip_git
echo.
echo  ---- Git -------------------------------------------------------
echo   url: https://github.com/git-for-windows/git/releases/download/v2.56.0.windows.2/Git-2.56.0.2-64-bit.exe
if exist "%DL%\git.exe" (
    echo   using the file already in the download folder
) else (
    curl -L --retry 3 --fail --progress-bar -o "%DL%\git.exe" "https://github.com/git-for-windows/git/releases/download/v2.56.0.windows.2/Git-2.56.0.2-64-bit.exe"
    if errorlevel 1 goto :failed_git
)
echo   installing - click Next through the wizard, the defaults are fine
start /wait "" "%DL%\git.exe"
echo   done
:skip_git

if "%CMAKEOK%"=="yes" goto :skip_cmake
echo.
echo  ---- CMake 3.31.12 ---------------------------------------------
echo   NOT the latest. whisper.cpp declares cmake_minimum_required(3.5)
echo   and CMake 4.0 removed compatibility with anything below 3.5.
echo   url: https://github.com/Kitware/CMake/releases/download/v3.31.12/cmake-3.31.12-windows-x86_64.msi
if exist "%DL%\cmake.msi" (
    echo   using the file already in the download folder
) else (
    curl -L --retry 3 --fail --progress-bar -o "%DL%\cmake.msi" "https://github.com/Kitware/CMake/releases/download/v3.31.12/cmake-3.31.12-windows-x86_64.msi"
    if errorlevel 1 goto :failed_cmake
)
echo   installing silently and adding it to PATH
msiexec /i "%DL%\cmake.msi" /qn /norestart ADD_CMAKE_TO_PATH=System
echo   done
:skip_cmake

if defined HAVECPP goto :skip_vsbt
echo.
echo  ---- VS 2022 Build Tools, C++ workload --------------------------
echo   This is the big one: 2 to 7 GB, 20 to 40 minutes.
echo   A progress window appears and looks frozen. It is not. Leave it.
echo   url: https://aka.ms/vs/17/release/vs_BuildTools.exe
if exist "%DL%\vs_BuildTools.exe" (
    echo   using the file already in the download folder
) else (
    curl -L --retry 3 --fail --progress-bar -o "%DL%\vs_BuildTools.exe" "https://aka.ms/vs/17/release/vs_BuildTools.exe"
    if errorlevel 1 goto :failed_vsbt
)
"%DL%\vs_BuildTools.exe" --wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
echo   the installer finished; the summary below re-checks whether it worked
:skip_vsbt

if defined LIBCLANG goto :skip_llvm
echo.
echo  ---- LLVM 21.1.0 ------------------------------------------------
echo   The version Concat's own scripts\ci\deps.lock pins.
echo   url: https://github.com/llvm/llvm-project/releases/download/llvmorg-21.1.0/LLVM-21.1.0-win64.exe
if exist "%DL%\llvm.exe" (
    echo   using the file already in the download folder
) else (
    curl -L --retry 3 --fail --progress-bar -o "%DL%\llvm.exe" "https://github.com/llvm/llvm-project/releases/download/llvmorg-21.1.0/LLVM-21.1.0-win64.exe"
    if errorlevel 1 goto :failed_llvm
)
echo   installing silently to C:\llvm - approve the UAC prompt
start /wait "" "%DL%\llvm.exe" /S /D=C:\llvm
if exist "C:\llvm\bin\libclang.dll" set "LIBCLANG=C:\llvm\bin"
echo   done
:skip_llvm

rem =====================================================================
rem Re-check everything, freshly, and report
rem =====================================================================
echo.
echo  ================================================================
echo   SUMMARY - checked again after installing
echo  ================================================================

set "OK=yes"

where git >nul 2>nul
if errorlevel 1 (
    echo   [MISSING] Git
    set "OK=no"
) else (
    echo   [ok]      Git
)

set "V2="
for /f "tokens=3" %%v in ('cmake --version 2^>nul') do if not defined V2 set "V2=%%v"
if not defined V2 (
    echo   [MISSING] CMake
    set "OK=no"
) else (
    echo %V2% | findstr /b /c:"3." >nul 2>&1
    if errorlevel 1 (
        echo   [WRONG]   CMake %V2% - must be 3.x
        set "OK=no"
    ) else (
        echo   [ok]      CMake %V2%
    )
)

set "L2="
for /f "delims=" %%i in ('where libclang.dll 2^>nul') do if not defined L2 for %%d in ("%%~dpi.") do set "L2=%%~fd"
if not defined L2 if defined LIBCLANG set "L2=%LIBCLANG%"
if not defined L2 if exist "C:\llvm\bin\libclang.dll" set "L2=C:\llvm"
if not defined L2 if exist "C:\Program Files\LLVM\bin\libclang.dll" set "L2=C:\Program Files\LLVM"
if not defined L2 (
    echo   [MISSING] LLVM - libclang.dll is nowhere this script can see
    set "OK=no"
) else (
    echo   [ok]      LLVM  libclang.dll is in %L2%
)

set "C2="
if exist "%VSWHERE%" for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2^>nul`) do set "C2=%%i"
if not defined C2 (
    echo   [MISSING] C++ toolchain
    set "OK=no"
) else (
    echo   [ok]      C++ toolchain
    echo             %C2%
)

echo  ================================================================
echo.

if "%OK%"=="no" (
    echo   Something is still missing. Double-click this file again: it
    echo   re-detects from scratch and only retries what is absent.
    echo.
    echo   If the same thing is missing twice, copy the DETECTED section
    echo   from the top of the run and the red text, and ask.
) else (
    echo   All four are in place.
    echo.
    echo   NEXT: close VSCode completely and reopen it, so the new PATH is
    echo   visible. Then follow STEP-2-AND-3.md, one line at a time.
    echo.
    echo   Note where LLVM is - the next step asks for that folder:
    if defined L2 echo       %L2%
)

echo.
echo  Press any key to close...
pause >nul
goto :eof

rem =====================================================================
rem One failure handler per download, so the message names the tool and
rem keeps whatever curl said.
rem =====================================================================
:failed_git
set "WHICH=Git"
set "URL=https://github.com/git-for-windows/git/releases/download/v2.56.0.windows.2/Git-2.56.0.2-64-bit.exe"
goto :report_fail
:failed_cmake
set "WHICH=CMake 3.31.12"
set "URL=https://github.com/Kitware/CMake/releases/download/v3.31.12/cmake-3.31.12-windows-x86_64.msi"
goto :report_fail
:failed_vsbt
set "WHICH=VS Build Tools"
set "URL=https://aka.ms/vs/17/release/vs_BuildTools.exe"
goto :report_fail
:failed_llvm
set "WHICH=LLVM 21.1.0"
set "URL=https://github.com/llvm/llvm-project/releases/download/llvmorg-21.1.0/LLVM-21.1.0-win64.exe"
goto :report_fail

:report_fail
echo.
echo  ================================================================
echo   THE %WHICH% DOWNLOAD FAILED
echo  ================================================================
echo   url:    %URL%
echo   saved to: %DL%
echo.
echo   curl's own message is just above this box - that is the useful
echo   part, and it says which of these it was:
echo.
echo     curl: (6)   could not resolve host  - DNS or no internet
echo     curl: (7)   failed to connect       - firewall or proxy
echo     curl: (22)  HTTP error 4xx/5xx      - the link moved
echo     curl: (28)  timed out               - slow connection
echo     curl: (35)  SSL connect error       - antivirus intercepting TLS
echo     curl: (56)  recv failure            - connection dropped
echo.
echo   If antivirus is the cause, the file may also be in quarantine.
echo.
echo   You can download it in a browser instead, from the url above,
echo   save it to %DL% with the exact filename shown, and
echo   double-click this file again - it will find the file there.
echo  ================================================================
echo.
echo  Press any key to close...
pause >nul
exit /b 1
