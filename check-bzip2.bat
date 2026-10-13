@echo off
setlocal EnableExtensions EnableDelayedExpansion

rem  What can this PC unpack a .tar.bz2 with? Changes nothing, only looks.
rem  DOUBLE-CLICK, then copy the whole output back.

title Concat - bzip2 check

echo.
echo  ---- 1. bzip2 on PATH? ----------------------------------------
where bzip2 2>nul
if errorlevel 1 echo   NOT on PATH
echo.

echo  ---- 2. where is git? -----------------------------------------
where git 2>nul
if errorlevel 1 echo   git NOT on PATH
echo.

echo  ---- 3. bzip2.exe in the usual places -------------------------
for %%d in ("%ProgramFiles%\Git\usr\bin" "%ProgramFiles(x86)%\Git\usr\bin" "%LOCALAPPDATA%\Programs\Git\usr\bin" "C:\Program Files\Git\usr\bin" "C:\Program Files\7-Zip" "C:\Program Files (x86)\7-Zip" "C:\msys64\usr\bin" "C:\Strawberry\c\bin") do (
    if exist "%%~d\bzip2.exe" (echo   FOUND  %%~d\bzip2.exe) else (echo   none   %%~d)
)
echo.

echo  ---- 4. a second tar.exe (Git ships one that carries its own) --
for %%d in ("%ProgramFiles%\Git\usr\bin" "%ProgramFiles(x86)%\Git\usr\bin" "%LOCALAPPDATA%\Programs\Git\usr\bin" "C:\Program Files\Git\usr\bin") do (
    if exist "%%~d\tar.exe" (echo   FOUND  %%~d\tar.exe) else (echo   none   %%~d)
)
echo   which tar is used now:
where tar 2>nul
echo.

echo  ---- 5. python? (its bz2 support is built in) -----------------
for %%p in (py python python3) do (
    set "V="
    for /f "delims=" %%v in ('%%p --version 2^>nul') do set "V=%%v"
    if defined V (echo   %%p -> !V!) else (echo   %%p -> not usable)
)
echo.

echo  ---- 6. the archive itself, and the target folder --------------
if exist "C:\Concat-deps\sherpa.tar.bz2" (
    for %%f in ("C:\Concat-deps\sherpa.tar.bz2") do echo   size %%~zf bytes   %%~ff
) else (
    echo   C:\Concat-deps\sherpa.tar.bz2 is NOT there
)
echo   contents of C:\Concat-deps:
dir /b "C:\Concat-deps" 2>nul
echo.
echo  ================================================================
echo   Copy all of the above back.
echo  ================================================================
pause
