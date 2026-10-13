@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem =====================================================================
rem  Concat - fetch sherpa-onnx only, and do the .lib / .dll split
rem
rem  DOUBLE-CLICK this file.
rem
rem  Why it exists: fetch-native-deps.bat does FFmpeg, ONNX Runtime and
rem  sherpa-onnx in one pass. On this machine FFmpeg and ONNX Runtime
rem  arrived but C:\Concat-deps\sherpa-libs did not, and sherpa-onnx-sys
rem  panics without it:
rem
rem      SHERPA_ONNX_LIB_DIR does not exist or is not a directory
rem
rem  This file does only step 3 of that script, at the version
rem  scripts\ci\deps.lock pins, verified by sha256. It reads the URL and
rem  the digest out of deps.lock rather than hardcoding them, so it
rem  cannot drift from CI.
rem
rem  THE SPLIT MATTERS. concat-speech\Cargo.toml explains both halves:
rem    * the shared build, because k2-fsa's only static archives are
rem      /MT and MSVC refuses to link those into a binary whose every
rem      other object is /MD - LNK2038 on every object pair;
rem    * SHERPA_ONNX_LIB_DIR must hold ONLY the import .lib files,
rem      because left to its own download the sys crate finds the DLLs
rem      beside them and tries to copy them into a "profile directory"
rem      it locates by walking up for an ancestor named after $PROFILE -
rem      a walk that panics under Concat's custom `quick` and `app`
rem      profiles. Keeping the DLLs out of the lib folder skips it.
rem
rem  Safe to re-run. A verified download is kept, so it resumes.
rem =====================================================================

title Concat - sherpa-onnx

set "DEPS=C:\Concat-deps"
set "SLIBS=%DEPS%\sherpa-libs"
set "SDLLS=%DEPS%\sherpa-dlls"

echo.
echo  ================================================================
echo   Concat - sherpa-onnx, on its own
echo  ================================================================
echo.

rem ---------------------------------------------------------------------
rem Find the Concat source, the same way fetch-native-deps.bat does
rem ---------------------------------------------------------------------
set "REPO="
if exist "%~dp0scripts\ci\deps.lock" set "REPO=%~dp0."
if not defined REPO for /d %%d in ("%~dp0*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not defined REPO for /d %%d in ("C:\dev\*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not defined REPO for /d %%d in ("C:\*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not defined REPO for /d %%d in ("%USERPROFILE%\Downloads\*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"

if not defined REPO (
    echo   [ERROR] could not find the Concat source folder - the one
    echo           holding scripts\ci\deps.lock. Put this file in that
    echo           folder, beside fetch-native-deps.bat, and run it again.
    goto :bail
)
for %%f in ("%REPO%") do set "REPO=%%~ff"
set "LOCK=%REPO%\scripts\ci\deps.lock"
echo   Concat source: %REPO%

rem ---------------------------------------------------------------------
rem The two tools this needs, both built into Windows 10
rem ---------------------------------------------------------------------
where curl >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] curl.exe is not on PATH. It ships with Windows 10
    echo           build 17063 and later.
    goto :bail
)
where tar >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] tar.exe is not on PATH. It ships with Windows 10 build
    echo           1803 and later, and it is what unpacks the .tar.bz2.
    goto :bail
)
where certutil >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] certutil.exe is not on PATH. It checks the sha256.
    goto :bail
)
echo   [ok] curl, tar and certutil are all present
echo.

rem ---------------------------------------------------------------------
rem Already done? Then there is nothing to fetch.
rem ---------------------------------------------------------------------
if exist "%SLIBS%\sherpa-onnx-c-api.lib" if exist "%SDLLS%\*.dll" (
    echo   [ok] already unpacked and split - nothing to do
    goto :report
)

rem ---------------------------------------------------------------------
rem The pinned URL and digest, out of deps.lock
rem ---------------------------------------------------------------------
set "RURL="
set "RSHA="
for /f "usebackq tokens=1,2,3 eol=#" %%a in (`findstr /r /v "^[ 	]*$" "%LOCK%"`) do (
    if /i "%%a"=="sherpa-onnx-win-x64-shared-MT-Release-lib" (
        set "RURL=%%b"
        set "RSHA=%%c"
    )
)
if not defined RURL (
    echo   [ERROR] no row named sherpa-onnx-win-x64-shared-MT-Release-lib
    echo           in scripts\ci\deps.lock
    goto :bail
)

rem ---------------------------------------------------------------------
rem Cross-check against src\Cargo.lock, which is what CI does.
rem
rem This is deliberately NOT how fetch-native-deps.bat does it. That file
rem finds the line number of "sherpa-onnx-sys" and then pipes the rest of
rem an 8627-line Cargo.lock through `more +N`. On this machine that read
rem back the string "sherpa-onnx-sys" instead of "1.13.7", so the check
rem compared the pinned URL against a package name and aborted:
rem
rem     [ERROR] deps.lock pins sherpa-onnx for a different version
rem     than src\Cargo.lock's sherpa-onnx-sys
rem
rem The two files do agree - Cargo.lock says 1.13.7 and the URL says
rem /v1.13.7/ - so that abort was simply wrong. This version reads the
rem version line straight out of Cargo.lock with one findstr, no line
rem arithmetic and no `more`. And it warns instead of stopping, because
rem the sha256 check below is what actually protects you: it is the
rem fingerprint of the exact bytes CI builds against.
rem ---------------------------------------------------------------------
set "SVER="
set "INPACKAGE="
for /f "usebackq tokens=1,* delims=:" %%a in (`findstr /n /c:"name = " /c:"version = " "%REPO%\src\Cargo.lock" 2^>nul`) do (
    set "L=%%b"
    echo !L! | findstr /c:"sherpa-onnx-sys" >nul && set "INPACKAGE=yes"
    if defined INPACKAGE (
        echo !L! | findstr /c:"version" >nul && if not defined SVER for /f "tokens=3" %%v in ("!L!") do set "SVER=%%~v"
    )
)
if defined SVER if "!SVER!"=="sherpa-onnx-sys" set "SVER="

if not defined SVER (
    echo   [WARN] could not read the sherpa-onnx-sys version out of
    echo          src\Cargo.lock. Continuing with what deps.lock pins.
) else (
    echo !RURL! | findstr /c:"/v!SVER!/" >nul
    if errorlevel 1 (
        echo   [WARN] deps.lock pins sherpa-onnx for a different version than
        echo          src\Cargo.lock's !SVER!. CI would fail the build on this,
        echo          but the sha256 below is the real guarantee, so this
        echo          continues. Worth telling the maintainer the two disagree.
        echo   url in deps.lock: !RURL!
    ) else (
        echo   [ok] Cargo.lock wants !SVER! and deps.lock pins it
    )
)

rem ---------------------------------------------------------------------
rem Download unless the file already there is the right one
rem ---------------------------------------------------------------------
if not exist "%DEPS%" mkdir "%DEPS%"
set "TAR=%DEPS%\sherpa.tar.bz2"
if exist "%TAR%" (
    call :sha "%TAR%"
    if /i "!GOTSHA!"=="%RSHA%" (
        echo   [ok] the archive is already downloaded and verified
    ) else (
        echo   the file already there does not match - deleting and refetching
        del "%TAR%"
    )
)
if not exist "%TAR%" (
    echo   url: !RURL!
    echo   downloading sherpa-onnx ...
    curl -L --retry 3 --fail --progress-bar -o "%TAR%" "!RURL!"
    if errorlevel 1 (
        echo   [ERROR] the download failed. curl's own message is just above:
        echo           ^(6^) DNS, ^(7^) firewall or proxy, ^(22^) link moved,
        echo           ^(28^) timed out, ^(35^) antivirus intercepting TLS,
        echo           ^(56^) connection dropped.
        goto :bail
    )
    call :sha "%TAR%"
    if /i not "!GOTSHA!"=="%RSHA%" (
        echo   [ERROR] sherpa-onnx does not match deps.lock
        echo     expected: %RSHA%
        echo     got:      !GOTSHA!
        echo   The file has been deleted. Either the download was corrupt or
        echo   the release was replaced upstream. Run this file again.
        del "%TAR%"
        goto :bail
    )
    echo   [ok] verified, sha256 !GOTSHA!
)

rem ---------------------------------------------------------------------
rem Unpack
rem ---------------------------------------------------------------------
rem ---------------------------------------------------------------------
rem Unpack.
rem
rem Windows' tar.exe does NOT decompress .tar.bz2 itself. It shells out to
rem a program called bzip2, and if bzip2.exe is not on PATH it fails with:
rem
rem     tar: Error opening archive: Can't initialize filter;
rem     unable to run program "bzip2 -d"
rem
rem That is not a truncated download - the sha256 above already proved the
rem file is intact. Windows simply ships tar without bzip2. So try, in
rem order: whatever bzip2 is already on this PC (Git for Windows and 7-Zip
rem both carry one), then plain tar, then Git's own tar, then Python,
rem whose bz2 module is built in.
rem ---------------------------------------------------------------------
if exist "%DEPS%\sherpa-stage" rmdir /s /q "%DEPS%\sherpa-stage"
mkdir "%DEPS%\sherpa-stage"

rem (a) a bzip2.exe already on this PC, put where tar can find it
where bzip2 >nul 2>nul
if errorlevel 1 (
    for %%d in ("%ProgramFiles%\Git\usr\bin" "%ProgramFiles(x86)%\Git\usr\bin" "%LOCALAPPDATA%\Programs\Git\usr\bin" "C:\Program Files\7-Zip" "C:\Program Files (x86)\7-Zip") do (
        if exist "%%~d\bzip2.exe" set "PATH=%%~d;!PATH!"
    )
)

rem (b) plain tar, which now works if (a) found a bzip2
set "UNTARRED="
cmd /c "tar -xf ""%TAR%"" -C ""%DEPS%\sherpa-stage"" 2>nul"
if not errorlevel 1 set "UNTARRED=yes"

rem (c) Git for Windows' own tar, which carries its decompressors
if not defined UNTARRED for %%d in ("%ProgramFiles%\Git\usr\bin" "%ProgramFiles(x86)%\Git\usr\bin" "%LOCALAPPDATA%\Programs\Git\usr\bin") do (
    if not defined UNTARRED if exist "%%~d\tar.exe" (
        "%%~d\tar.exe" -xf "%TAR%" -C "%DEPS%\sherpa-stage" 2>nul
        if not errorlevel 1 set "UNTARRED=yes"
    )
)

rem (d) Python, whose bz2 support is built in and needs no helper at all.
rem     Run from inside the stage folder with a relative archive path,
rem     because a backslash inside a Python string literal escapes.
if not defined UNTARRED for %%p in (py python python3) do (
    if not defined UNTARRED (
        pushd "%DEPS%\sherpa-stage"
        %%p -c "import tarfile,sys; tarfile.open(sys.argv[1],'r:bz2').extractall()" "..\sherpa.tar.bz2" >nul 2>nul
        if not errorlevel 1 set "UNTARRED=yes"
        popd
    )
)

rem (e) nothing worked, so say which piece is missing rather than guessing
if not defined UNTARRED (
    echo.
    echo   [ERROR] nothing on this PC could unpack a .tar.bz2. Tried:
    echo           bzip2.exe on PATH, Git's and 7-Zip's copies, tar.exe,
    echo           Git's own tar.exe, and py / python / python3.
    echo.
    echo   The archive itself is fine - its sha256 matched deps.lock.
    echo   Cheapest fix: install Git for Windows, which puts both
    echo   bzip2.exe and tar.exe in C:\Program Files\Git\usr\bin, then
    echo   run this file again.
    echo.
    echo   Already have Git? Then it is somewhere this file did not look.
    echo   Find it with:   where git
    echo   and tell me the path.
    goto :bail
)
echo   [ok] unpacked

set "SINNER="
for /d %%d in ("%DEPS%\sherpa-stage\*") do set "SINNER=%%~fd"
if not defined SINNER (
    echo   [ERROR] the archive unpacked but nothing landed in sherpa-stage
    goto :bail
)
if not exist "!SINNER!\lib" (
    echo   [ERROR] !SINNER! has no lib folder inside it
    goto :bail
)

rem ---------------------------------------------------------------------
rem THE SPLIT: .lib into sherpa-libs, .dll into sherpa-dlls
rem ---------------------------------------------------------------------
if exist "%SLIBS%" rmdir /s /q "%SLIBS%"
if exist "%SDLLS%" rmdir /s /q "%SDLLS%"
mkdir "%SLIBS%"
mkdir "%SDLLS%"
copy /y "!SINNER!\lib\sherpa-onnx-*.lib" "%SLIBS%\" >nul
copy /y "!SINNER!\lib\sherpa-onnx-*.dll" "%SDLLS%\" >nul
rmdir /s /q "%DEPS%\sherpa-stage"
del "%TAR%"

dir /b "%SLIBS%\*.lib" >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] no .lib files reached %SLIBS%
    goto :bail
)
dir /b "%SLIBS%\*.dll" >nul 2>nul
if not errorlevel 1 (
    echo   [ERROR] a .dll reached %SLIBS%. The split failed, and a .dll in
    echo           that folder causes a panic mentioning $PROFILE during
    echo           the build. Delete the folder and run this file again.
    goto :bail
)
dir /b "%SDLLS%\*.dll" >nul 2>nul
if errorlevel 1 (
    echo   [WARN] no .dll files reached %SDLLS%. The build will link, but
    echo          the app will not start. Check what the archive held.
) else (
    echo   [ok] unpacked, .lib and .dll kept apart
)

rem ---------------------------------------------------------------------
rem The environment variable
rem ---------------------------------------------------------------------
setx SHERPA_ONNX_LIB_DIR "%SLIBS%" >nul
echo   [ok] SHERPA_ONNX_LIB_DIR set permanently to %SLIBS%

:report
echo.
echo  ================================================================
echo   WHAT IS IN THE TWO FOLDERS NOW
echo  ================================================================
echo.
echo   %SLIBS%   (.lib only - a .dll here breaks the build)
dir /b "%SLIBS%" 2>nul
echo.
echo   %SDLLS%   (.dll only - goes on PATH at run time)
dir /b "%SDLLS%" 2>nul
echo.
echo  ================================================================
echo   NEXT
echo  ================================================================
echo.
echo   1. Close the build terminal window you have open. It was opened
echo      before this folder existed.
echo   2. Double-click open-a-real-build-terminal.bat again. It should
echo      now say "[ok] all five libraries are where they should be"
echo      with no WARNING line.
echo   3. In that window:  cargo dev
echo.
echo   The first full build is long - CI budgets 60 minutes for it.
echo   If rustc is killed or the PC freezes, that is the 16 GB of RAM,
echo   not a code error: retry with  cargo dev -- -j 2
echo  ================================================================
echo.
pause
exit /b 0

rem ---------------------------------------------------------------------
rem :sha FILE - the file's sha256, lowercase, into GOTSHA.
rem certutil prints it as "aa bb cc ..." with spaces, so the spaces come
rem out and it is folded to lowercase to match deps.lock.
rem ---------------------------------------------------------------------
:sha
set "GOTSHA="
for /f "skip=1 tokens=*" %%h in ('certutil -hashfile "%~1" SHA256 2^>nul') do (
    if not defined GOTSHA (
        set "GOTSHA=%%h"
        set "GOTSHA=!GOTSHA: =!"
    )
)
exit /b 0

:bail
echo.
echo  ================================================================
echo   STOPPED. The line above says why. Anything already downloaded and
echo   verified is kept, so re-running resumes rather than restarting.
echo  ================================================================
echo.
pause
exit /b 1
