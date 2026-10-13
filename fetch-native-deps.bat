@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem =====================================================================
rem  Concat for Windows - step 3: the native libraries, in pure batch
rem
rem  DOUBLE-CLICK this file. Nothing to type, nothing to paste.
rem
rem  This replaces setup-windows-deps.ps1. It does the same job with no
rem  PowerShell at all, because four attempts to invoke that script
rem  through a batch wrapper produced an argument-binding error nobody
rem  could explain, and the debugging had stopped being worth it.
rem
rem  Everything it needs is built into Windows 10:
rem    curl.exe    downloads, since build 17063
rem    tar.exe     unpacks .zip .tar.gz .tar.xz .tar.bz2, since 1803
rem    certutil    computes sha256, since Vista
rem    setx        stores environment variables
rem
rem  It reads scripts\ci\deps.lock, the same file the PowerShell script
rem  read and the same file Concat's CI reads, so the versions and the
rem  fingerprints come from the repository rather than from this file.
rem =====================================================================

title Concat - native libraries

set "REPO=C:\dev\concat"
set "DEPS=C:\Concat-deps"

echo.
echo  ================================================================
echo   Concat - native libraries, pure batch, no PowerShell
echo  ================================================================
echo.

rem ---------------------------------------------------------------------
rem Find the Concat source
rem ---------------------------------------------------------------------
if exist "%REPO%\scripts\ci\deps.lock" goto :have_repo

set "REPO="
if exist "%~dp0scripts\ci\deps.lock" set "REPO=%~dp0."
if not defined REPO for /d %%d in ("%~dp0*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not defined REPO for /d %%d in ("C:\dev\*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not defined REPO for /d %%d in ("C:\*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not defined REPO for /d %%d in ("%USERPROFILE%\Downloads\*") do if not defined REPO if exist "%%~fd\scripts\ci\deps.lock" set "REPO=%%~fd"
if not "%~1"=="" if exist "%~1\scripts\ci\deps.lock" set "REPO=%~1"

if not defined REPO (
    echo   Could not find the Concat source folder - the one containing
    echo   scripts\ci\deps.lock, src\ and ARCHITECTURE.md.
    echo.
    set /p "REPO=Full path to your Concat folder: "
    if not defined REPO goto :bail
    if not exist "!REPO!\scripts\ci\deps.lock" (
        echo.
        echo   There is no scripts\ci\deps.lock in !REPO!
        echo   so that is not the Concat source folder.
        goto :bail
    )
)

:have_repo
for %%f in ("%REPO%") do set "REPO=%%~ff"
set "LOCK=%REPO%\scripts\ci\deps.lock"
echo   Concat source: %REPO%
echo   Pinned versions come from: scripts\ci\deps.lock
echo   Unpacking into:            %DEPS%
echo.

rem ---------------------------------------------------------------------
rem The tools this needs, all built into Windows 10
rem ---------------------------------------------------------------------
where curl >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] curl.exe is not on PATH. It ships with Windows 10
    echo   build 17063 and later. If yours is older, this file cannot
    echo   download anything.
    goto :bail
)
where tar >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] tar.exe is not on PATH. It ships with Windows 10
    echo   build 1803 and later, and it is what unpacks sherpa-onnx's
    echo   .tar.bz2 - which is why no 7-Zip is needed.
    goto :bail
)
where certutil >nul 2>nul
if errorlevel 1 (
    echo   [ERROR] certutil.exe is not on PATH. It is used to check the
    echo   sha256 of every download. Without it this file would install
    echo   libraries nobody had verified, which is the one thing
    echo   scripts\ci\deps.lock exists to prevent.
    goto :bail
)
echo   [ok] curl, tar and certutil are all present
echo.

rem ---------------------------------------------------------------------
rem Find LLVM, wherever it is
rem ---------------------------------------------------------------------
set "LIBCLANG="
for /f "delims=" %%i in ('where libclang.dll 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do set "LIBCLANG=%%~fd"
if not defined LIBCLANG for %%d in ("C:\Program Files\LLVM" "C:\Program Files (x86)\LLVM" "C:\llvm" "%LOCALAPPDATA%\Programs\LLVM" "C:\Tools\llvm") do if exist "%%~d\bin\libclang.dll" set "LIBCLANG=%%~d\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKLM\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if not defined LIBCLANG for /f "tokens=2*" %%a in ('reg query "HKCU\SOFTWARE\LLVM\LLVM" /ve 2^>nul ^| findstr REG_SZ') do if exist "%%b\bin\libclang.dll" set "LIBCLANG=%%b\bin"
if not defined LIBCLANG for /f "delims=" %%i in ('where clang 2^>nul') do if not defined LIBCLANG for %%d in ("%%~dpi.") do if exist "%%~fd\libclang.dll" set "LIBCLANG=%%~fd"
rem Last resort: the folder may hold libclang under a version suffix
if not defined LIBCLANG for %%d in ("C:\Program Files\LLVM\bin" "C:\llvm\bin") do if exist "%%~d\libclang.dll" set "LIBCLANG=%%~d"

if not defined LIBCLANG (
    echo   [ERROR] libclang.dll is nowhere this file can see. LLVM is
    echo   needed because bindgen uses libclang to generate the FFmpeg
    echo   bindings. Double-click install-build-tools.bat first, which
    echo   puts LLVM somewhere known, then come back here.
    goto :bail
)
rem LIBCLANG is the folder holding libclang.dll, which is exactly what
rem LIBCLANG_PATH wants - no \bin arithmetic, so nothing can double up.
echo   [ok] libclang.dll is in: %LIBCLANG%
echo.

if not exist "%DEPS%" mkdir "%DEPS%"

rem =====================================================================
rem 1. FFmpeg
rem =====================================================================
echo  ---- FFmpeg ----------------------------------------------------
call :row ffmpeg-win64
if not defined RURL (
    echo   [ERROR] no row named ffmpeg-win64 in deps.lock
    goto :bail
)
set "FF=%DEPS%\ffmpeg"
if exist "%FF%\lib\avcodec.lib" (
    echo   [ok] already unpacked, skipping the download
) else (
    echo   url: !RURL!
    call :get !RURL! %DEPS%\ffmpeg.zip !RSHA! ffmpeg
    if errorlevel 1 goto :bail
    if exist "%DEPS%\ffmpeg-stage" rmdir /s /q "%DEPS%\ffmpeg-stage"
    mkdir "%DEPS%\ffmpeg-stage"
    tar -xf "%DEPS%\ffmpeg.zip" -C "%DEPS%\ffmpeg-stage"
    if errorlevel 1 (
        echo   [ERROR] tar could not unpack the FFmpeg archive
        goto :bail
    )
    rem the archive unpacks to one folder; move it into place
    for /d %%d in ("%DEPS%\ffmpeg-stage\*") do set "INNER=%%~fd"
    if not defined INNER (
        echo   [ERROR] nothing unpacked into ffmpeg-stage
        goto :bail
    )
    if exist "%FF%" rmdir /s /q "%FF%"
    move "!INNER!" "%FF%" >nul
    rmdir /s /q "%DEPS%\ffmpeg-stage"
    del "%DEPS%\ffmpeg.zip"
    if not exist "%FF%\lib\avcodec.lib" (
        echo   [ERROR] avcodec.lib is not in %FF%\lib - the archive's
        echo   layout has changed since deps.lock was written.
        goto :bail
    )
    echo   [ok] unpacked, avcodec.lib found
)
echo.

rem =====================================================================
rem 2. ONNX Runtime
rem =====================================================================
echo  ---- ONNX Runtime ----------------------------------------------
call :row onnxruntime-win-x64
if not defined RURL (
    echo   [ERROR] no row named onnxruntime-win-x64 in deps.lock
    goto :bail
)
set "ORT=%DEPS%\ort"
if exist "%ORT%\lib\onnxruntime.lib" (
    echo   [ok] already unpacked, skipping the download
) else (
    echo   url: !RURL!
    call :get !RURL! %DEPS%\ort.zip !RSHA! onnxruntime
    if errorlevel 1 goto :bail
    if exist "%DEPS%\ort-stage" rmdir /s /q "%DEPS%\ort-stage"
    mkdir "%DEPS%\ort-stage"
    tar -xf "%DEPS%\ort.zip" -C "%DEPS%\ort-stage"
    if errorlevel 1 (
        echo   [ERROR] tar could not unpack the ONNX Runtime archive
        goto :bail
    )
    for /d %%d in ("%DEPS%\ort-stage\*") do set "INNER=%%~fd"
    if not defined INNER (
        echo   [ERROR] nothing unpacked into ort-stage
        goto :bail
    )
    if exist "%ORT%" rmdir /s /q "%ORT%"
    move "!INNER!" "%ORT%" >nul
    rmdir /s /q "%DEPS%\ort-stage"
    del "%DEPS%\ort.zip"
    if not exist "%ORT%\lib\onnxruntime.lib" (
        echo   [ERROR] onnxruntime.lib is not in %ORT%\lib
        goto :bail
    )
    echo   [ok] unpacked, onnxruntime.lib found
)
echo.

rem =====================================================================
rem 3. sherpa-onnx, with the .lib and .dll split
rem =====================================================================
echo  ---- sherpa-onnx -----------------------------------------------

rem The version has to be the one src\Cargo.lock resolves for
rem sherpa-onnx-sys. CI fails the build when the two disagree, and so
rem does this, because linking against the wrong one fails obscurely.
set "SVER="
for /f "usebackq tokens=3" %%v in (`findstr /c:"sherpa-onnx-sys" "%REPO%\src\Cargo.lock"`) do set "SVER=%%v"
if not defined SVER (
    rem the name and the version are on adjacent lines, so read the line after
    for /f "tokens=1 delims=:" %%L in ('findstr /n /c:"sherpa-onnx-sys" "%REPO%\src\Cargo.lock"') do set "SLINE=%%L"
    if defined SLINE for /f "tokens=3 skipasempty delims=" %%v in ('more +%SLINE% "%REPO%\src\Cargo.lock" ^| findstr /b /c:"version"') do if not defined SVER set "SVER=%%v"
)
if defined SVER set "SVER=!SVER:"=!"
if not defined SVER (
    echo   [WARN] could not read the sherpa-onnx-sys version from
    echo   src\Cargo.lock. Falling back to the version deps.lock pins,
    echo   without the cross-check CI does.
)

call :row sherpa-onnx-win-x64-shared-MT-Release-lib
if not defined RURL (
    echo   [ERROR] no row named sherpa-onnx-win-x64-shared-MT-Release-lib
    echo   in deps.lock
    goto :bail
)
if defined SVER (
    echo !RURL! | findstr /c:"/v%SVER%/" >nul
    if errorlevel 1 (
        echo   [ERROR] deps.lock pins sherpa-onnx for a different version
        echo   than src\Cargo.lock's %SVER%. CI fails the build on this
        echo   mismatch, because the two have to agree.
        echo   url in deps.lock: !RURL!
        goto :bail
    )
    echo   [ok] Cargo.lock wants %SVER% and deps.lock pins it
)

set "SLIBS=%DEPS%\sherpa-libs"
set "SDLLS=%DEPS%\sherpa-dlls"
if exist "%SLIBS%\*.lib" if exist "%SDLLS%\*.dll" (
    echo   [ok] already unpacked and split, skipping the download
) else (
    echo   url: !RURL!
    call :get !RURL! %DEPS%\sherpa.tar.bz2 !RSHA! sherpa-onnx
    if errorlevel 1 goto :bail
    if exist "%DEPS%\sherpa-stage" rmdir /s /q "%DEPS%\sherpa-stage"
    mkdir "%DEPS%\sherpa-stage"
    tar -xf "%DEPS%\sherpa.tar.bz2" -C "%DEPS%\sherpa-stage"
    if errorlevel 1 (
        echo   [ERROR] tar could not unpack the sherpa-onnx archive.
        echo   Windows tar handles .tar.bz2, so if this failed the file
        echo   is probably truncated - delete it and re-run.
        goto :bail
    )
    set "SINNER="
    for /d %%d in ("%DEPS%\sherpa-stage\*") do set "SINNER=%%~fd"
    if not defined SINNER (
        echo   [ERROR] nothing unpacked into sherpa-stage
        goto :bail
    )
    if not exist "!SINNER!\lib" (
        echo   [ERROR] !SINNER! has no lib folder inside it
        goto :bail
    )

    rem THE SPLIT. concat-speech\Cargo.toml explains both halves:
    rem  * the shared build, because k2-fsa's only static archives are
    rem    /MT and MSVC refuses to link those into a binary whose every
    rem    other object is /MD - LNK2038 on every object pair;
    rem  * SHERPA_ONNX_LIB_DIR must hold ONLY the import .lib files,
    rem    because left to its own download the sys crate finds the DLLs
    rem    beside them and tries to copy them into a "profile directory"
    rem    it locates by walking up for an ancestor named after $PROFILE
    rem    - a walk that panics under Concat's custom `app` profile.
    if exist "%SLIBS%" rmdir /s /q "%SLIBS%"
    if exist "%SDLLS%" rmdir /s /q "%SDLLS%"
    mkdir "%SLIBS%"
    mkdir "%SDLLS%"
    copy /y "!SINNER!\lib\sherpa-onnx-*.lib" "%SLIBS%\" >nul
    copy /y "!SINNER!\lib\sherpa-onnx-*.dll" "%SDLLS%\" >nul
    rmdir /s /q "%DEPS%\sherpa-stage"
    del "%DEPS%\sherpa.tar.bz2"

    dir /b "%SLIBS%\*.lib" >nul 2>nul
    if errorlevel 1 (
        echo   [ERROR] no .lib files reached %SLIBS%
        goto :bail
    )
    dir /b "%SLIBS%\*.dll" >nul 2>nul
    if not errorlevel 1 (
        echo   [ERROR] a .dll reached %SLIBS%. The split failed, and a
        echo   .dll in that folder causes a panic mentioning $PROFILE
        echo   during the build. Delete the folder and re-run.
        goto :bail
    )
    echo   [ok] unpacked, .lib and .dll kept apart
)
echo.

rem =====================================================================
rem Environment variables
rem =====================================================================
echo  ---- environment -----------------------------------------------
setx FFMPEG_DIR "%FF%" >nul
setx ORT_LIB_LOCATION "%ORT%\lib" >nul
setx ORT_PREFER_DYNAMIC_LINK "1" >nul
setx SHERPA_ONNX_LIB_DIR "%SLIBS%" >nul
setx LIBCLANG_PATH "%LIBCLANG%" >nul
echo   FFMPEG_DIR              = %FF%
echo   ORT_LIB_LOCATION        = %ORT%\lib
echo   ORT_PREFER_DYNAMIC_LINK = 1
echo   SHERPA_ONNX_LIB_DIR     = %SLIBS%
echo   LIBCLANG_PATH           = %LIBCLANG%

rem PATH. setx truncates its value at 1024 characters, so the current
rem user PATH is measured first and left alone if it is already long -
rem losing the tail of somebody's PATH to save four entries is a bad
rem trade. open-a-build-terminal.bat covers the case where this skips,
rem by putting the folders on PATH for one session and touching nothing.
set "NEEDPATH=%FF%\bin;%ORT%\lib;%SDLLS%;%LIBCLANG%"
set "USERPATH="
for /f "tokens=2*" %%a in ('reg query "HKCU\Environment" /v Path 2^>nul ^| findstr /i "Path"') do set "USERPATH=%%b"

call :len "!USERPATH!"
if !NLEN! GTR 900 (
    echo   [SKIP] your user PATH is already !NLEN! characters. setx would
    echo   truncate it at 1024 and lose the tail, so this is leaving it
    echo   alone. Use open-a-build-terminal.bat instead, which puts the
    echo   four folders on PATH for one session and changes nothing
    echo   permanently. Or add them by hand:
    echo       Win+R, sysdm.cpl, Advanced, Environment Variables
    for %%p in ("%NEEDPATH:;=" "%") do echo         %%~p
) else (
    set "NEWPATH=!USERPATH!"
    for %%p in ("%NEEDPATH:;=" "%") do (
        echo ;!NEWPATH!; | findstr /i /c:";%%~p;" >nul
        if errorlevel 1 (
            if not defined NEWPATH (set "NEWPATH=%%~p") else (set "NEWPATH=!NEWPATH!;%%~p")
            echo   adding to PATH: %%~p
        ) else (
            echo   already on PATH: %%~p
        )
    )
    call :len "!NEWPATH!"
    if !NLEN! GTR 1020 (
        echo   [SKIP] that would make PATH !NLEN! characters, over setx's
        echo   1024 limit. Leaving it alone; use open-a-build-terminal.bat.
    ) else (
        setx Path "!NEWPATH!" >nul
        echo   [ok] user PATH updated, now !NLEN! characters
    )
)
echo.

rem =====================================================================
echo  ================================================================
echo   DONE
echo  ================================================================
echo.
echo   NEXT, and this matters: close VSCode COMPLETELY and reopen it.
echo   setx writes to the registry, and a process that is already
echo   running cannot see it. Closing the terminal panel is not
echo   enough - close VSCode itself.
echo.
echo   Then in a fresh terminal, ONE line at a time:
echo.
echo       echo %%FFMPEG_DIR%%
echo       dir %%SHERPA_ONNX_LIB_DIR%%\*.dll
echo.
echo   The first prints %FF%
echo   The second must print "File Not Found". That folder holds .lib
echo   files only, and a .dll in it breaks the build later.
echo.
echo   Then the first build - needs no FFmpeg and no GPU:
echo.
echo       cd %REPO%\src
echo       cargo test -p concat-project
echo.
echo   Expect "114 passed".
echo  ================================================================
echo.
echo   Press any key to close...
pause >nul
exit /b 0

rem =====================================================================
rem Subroutines
rem =====================================================================

rem :row NAME - read NAME's url and sha256 out of deps.lock into RURL/RSHA.
rem The lock file's format is three whitespace-separated columns, name,
rem url, sha256, with # comments.
:row
set "RURL="
set "RSHA="
for /f "usebackq tokens=1,2,3 eol=#" %%a in (`findstr /r /v "^[ 	]*$" "%LOCK%"`) do (
    if /i "%%a"=="%~1" (
        set "RURL=%%b"
        set "RSHA=%%c"
    )
)
exit /b 0

rem :get URL OUT WANTSHA LABEL - download unless OUT is already correct.
rem A file whose fingerprint does not match is DELETED, not built against:
rem that is the whole point of deps.lock, and the reason a release that was
rem replaced upstream stops the build instead of ending up in a bundle.
:get
set "GURL=%~1"
set "GOUT=%~2"
set "GWANT=%~3"
set "GLABEL=%~4"
if exist "%GOUT%" (
    call :sha "%GOUT%"
    if /i "!GOTSHA!"=="%GWANT%" (
        echo   [ok] %GLABEL% is already downloaded and verified
        exit /b 0
    )
    echo   the file already there does not match - deleting and refetching
    del "%GOUT%"
)
echo   downloading %GLABEL% ...
curl -L --retry 3 --fail --progress-bar -o "%GOUT%" "%GURL%"
if errorlevel 1 (
    echo   [ERROR] the %GLABEL% download failed. curl's own message is
    echo   just above: ^(6^) is DNS, ^(7^) firewall or proxy, ^(22^) the link
    echo   moved, ^(28^) timed out, ^(35^) antivirus intercepting TLS,
    echo   ^(56^) the connection dropped.
    echo   url: %GURL%
    exit /b 1
)
call :sha "%GOUT%"
if /i not "!GOTSHA!"=="%GWANT%" (
    echo   [ERROR] %GLABEL% does not match deps.lock
    echo     expected: %GWANT%
    echo     got:      !GOTSHA!
    echo   The file has been deleted. Either the download was corrupt or
    echo   the release was replaced upstream. Re-run to try again.
    del "%GOUT%"
    exit /b 1
)
echo   [ok] %GLABEL% verified, sha256 !GOTSHA!
exit /b 0

rem :sha FILE - the file's sha256, lowercase, into GOTSHA.
rem certutil prints it as "aa bb cc ..." with spaces, so the spaces come
rem out and the result is folded to lowercase to match deps.lock.
:sha
set "GOTSHA="
for /f "skip=1 tokens=*" %%h in ('certutil -hashfile "%~1" SHA256 2^>nul') do (
    if not defined GOTSHA (
        set "GOTSHA=%%h"
        set "GOTSHA=!GOTSHA: =!"
    )
)
exit /b 0

rem :len STRING - the length of STRING, into NLEN. Batch has no length
rem operator, so this walks the string one character at a time.
:len
set "LSTR=%~1"
set /a NLEN=0
:len_loop
if not defined LSTR exit /b 0
set "LSTR=!LSTR:~1!"
set /a NLEN+=1
goto :len_loop

:bail
echo.
echo  ================================================================
echo   STOPPED. Fix the line above and double-click this file again -
echo   anything already downloaded and verified is kept, so it resumes
echo   rather than starting over.
echo  ================================================================
echo.
echo   Press any key to close...
pause >nul
exit /b 1
