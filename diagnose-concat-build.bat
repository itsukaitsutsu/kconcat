@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem =====================================================================
rem  Concat for Windows - why did the build fail?
rem
rem  DOUBLE-CLICK this file. It changes nothing. It only looks, and prints
rem  what it finds. Copy the whole output and send it back.
rem
rem  Written for this specific failure:
rem
rem      FFmpeg 5.1 or higher is required, but found avcodec version (0, 0)
rem
rem  ffmpeg-sys-the-third's build.rs starts every library's version at
rem  (0, 0) and then uses libclang to preprocess a generated check.c that
rem  says `#include <libavcodec/avcodec.h>`. It reads LIBAVCODEC_VERSION_MAJOR
rem  and _MINOR out of that header. If the header is never found, the
rem  version stays at its initial (0, 0) and this assertion fires.
rem
rem  So (0, 0) means one thing: clang ran, but it could not find FFmpeg's
rem  headers. The include path it was given is FFMPEG_DIR\include. This
rem  file checks whether that folder exists and has the headers in it.
rem =====================================================================

title Concat - build diagnosis

echo.
echo  ================================================================
echo   CONCAT BUILD DIAGNOSIS - nothing here is changed, only read
echo  ================================================================
echo.

echo  ---- 1. the five environment variables -------------------------
echo.
for %%v in (FFMPEG_DIR ORT_LIB_LOCATION ORT_PREFER_DYNAMIC_LINK SHERPA_ONNX_LIB_DIR LIBCLANG_PATH) do (
    if defined %%v (
        echo   %%v = !%%v!
    ) else (
        echo   %%v = *** NOT SET IN THIS WINDOW ***
    )
)
echo.
echo   A variable can be set permanently and still be missing here, if
echo   this window was opened before it was set. That is normal - close
echo   and reopen, or use open-a-build-terminal.bat which sets them
echo   itself.
echo.

echo  ---- 2. FFmpeg, which is what the (0,0) error is about ---------
echo.
if not defined FFMPEG_DIR (
    echo   FFMPEG_DIR is not set in this window, so the build script fell
    echo   through to vcpkg and then pkg-config. Run this file from the
    echo   window that open-a-build-terminal.bat opens.
    echo.
    echo   Checking the folder fetch-native-deps.bat would have used anyway:
    set "FFGUESS=C:\Concat-deps\ffmpeg"
) else (
    set "FFGUESS=%FFMPEG_DIR%"
)

echo   Looking in: !FFGUESS!
echo.
if not exist "!FFGUESS!" (
    echo   [MISSING] that folder does not exist at all.
    echo             fetch-native-deps.bat has not run, or it failed.
) else (
    echo   [ok] the folder exists

    if exist "!FFGUESS!\lib\avcodec.lib" (
        echo   [ok] lib\avcodec.lib        - the import library, for linking
    ) else (
        echo   [MISSING] lib\avcodec.lib
    )

    if exist "!FFGUESS!\include\libavcodec\avcodec.h" (
        echo   [ok] include\libavcodec\avcodec.h   - THIS is the file the
        echo                                      build script reads the
        echo                                      version out of
    ) else (
        echo   [MISSING] include\libavcodec\avcodec.h
        echo             *** This is almost certainly the cause of the
        echo                 "avcodec version (0, 0)" panic. ***
    )

    if exist "!FFGUESS!\bin\avcodec-*.dll" (
        echo   [ok] bin\avcodec-*.dll      - the runtime DLL
    ) else (
        echo   [MISSING] bin\avcodec-*.dll
    )

    echo.
    echo   What is actually inside !FFGUESS! :
    dir /b "!FFGUESS!" 2>nul | findstr /v "^$"
    echo.
    echo   And inside its include folder, if there is one:
    if exist "!FFGUESS!\include" (
        dir /b "!FFGUESS!\include" 2>nul
        echo.
        echo   And inside include\libavcodec:
        if exist "!FFGUESS!\include\libavcodec" (
            dir /b "!FFGUESS!\include\libavcodec" 2>nul | findstr /i "avcodec.h version" 
        ) else (
            echo     there is no include\libavcodec folder
        )
    ) else (
        echo     there is no include folder at all - so this is not an
        echo     FFmpeg development tree, it may be only the binaries.
    )
)
echo.

echo  ---- 3. is there another FFmpeg somewhere obvious? -------------
echo.
for %%d in ("C:\Concat-deps" "C:\ffmpeg" "%USERPROFILE%\Downloads") do (
    if exist "%%~d" (
        echo   searching %%~d for avcodec.h ...
        dir /s /b "%%~d\avcodec.h" 2>nul
    )
)
echo   (blank above means none found in those three places)
echo.

echo  ---- 4. libclang, which the build script uses to read headers --
echo.
if defined LIBCLANG_PATH (
    echo   LIBCLANG_PATH = %LIBCLANG_PATH%
    if exist "%LIBCLANG_PATH%\libclang.dll" (
        echo   [ok] libclang.dll is in it
    ) else (
        echo   [MISSING] there is no libclang.dll in that folder.
        echo             *** A path ending in \bin\bin means an older
        echo                 version of fetch-native-deps.bat set it.
        echo                 Re-run the current one, which fixes it. ***
    )
) else (
    echo   LIBCLANG_PATH is not set in this window.
)
echo.
echo   Every libclang.dll on PATH:
where libclang.dll 2>nul
echo   (blank means it is not on PATH, which is fine if LIBCLANG_PATH is right)
echo.

echo  ---- 5. the rest of the native libraries -----------------------
echo.
if exist "C:\Concat-deps\ort\lib\onnxruntime.lib" (
    echo   [ok] ONNX Runtime   C:\Concat-deps\ort\lib\onnxruntime.lib
) else (
    echo   [MISSING] ONNX Runtime onnxruntime.lib
)
if exist "C:\Concat-deps\sherpa-libs" (
    echo   [ok] sherpa-onnx libs folder exists
    echo        .lib files: 
    dir /b "C:\Concat-deps\sherpa-libs\*.lib" 2>nul | find /c ".lib"
    echo        .dll files that must NOT be in there:
    dir /b "C:\Concat-deps\sherpa-libs\*.dll" 2>nul | find /c ".dll"
    echo        (that second number must be 0)
) else (
    echo   [MISSING] sherpa-onnx libs folder
)
if exist "C:\Concat-deps\sherpa-dlls" (
    echo   [ok] sherpa-onnx dlls folder exists
) else (
    echo   [MISSING] sherpa-onnx dlls folder
)
echo.

echo  ---- 6. the Rust side ------------------------------------------
echo.
where cargo >nul 2>nul
if errorlevel 1 (
    echo   [MISSING] cargo is not on PATH in this window.
) else (
    echo   [ok] cargo:
    cargo --version
    echo.
    echo   active toolchain:
    cargo +stable --version >nul 2>nul
    rustup show active-toolchain 2>nul
    echo.
    echo   rust-analyzer component, for the VSCode warning:
    rustup component list --toolchain 1.93.1-x86_64-pc-windows-msvc 2>nul | findstr /i "rust-analyzer"
)
echo.

echo  ---- 7. the source folder --------------------------------------
echo.
for %%d in ("C:\dev\concat" "C:\dev\Concat") do (
    if exist "%%~d\src\Cargo.toml" (
        echo   [ok] %%~d is the Concat source
        echo        deps.lock: 
        if exist "%%~d\scripts\ci\deps.lock" (echo          present) else (echo          MISSING)
        echo        Cargo.lock:
        if exist "%%~d\src\Cargo.lock" (echo          present) else (echo          MISSING)
        echo        the Tranche A patch:
        if exist "%%~d\src\crates\concat-project\src\captionmap.rs" (echo          applied) else (echo          not applied)
    )
)
echo.

echo  ================================================================
echo   HOW TO READ THIS
echo  ================================================================
echo.
echo   If section 2 says [MISSING] include\libavcodec\avcodec.h, that is
echo   the (0,0) panic. FFMPEG_DIR points somewhere that has the binaries
echo   but not the headers, or somewhere that does not exist. Fix: run
echo   fetch-native-deps.bat, which unpacks BtbN's "shared" build - the
echo   one that carries both.
echo.
echo   If section 4 says libclang.dll is missing from LIBCLANG_PATH and
echo   the path ends in \bin\bin, an older version of the fetch script set
echo   it. Re-run the current fetch-native-deps.bat; it computes that path
echo   without the doubling.
echo.
echo   If section 1 shows variables NOT SET but section 2 finds everything,
echo   the window is stale. Use open-a-build-terminal.bat and build in
echo   THAT window.
echo.
echo   Copy all of the above and send it back.
echo  ================================================================
echo.
echo  Press any key to close...
pause >nul
exit /b 0
