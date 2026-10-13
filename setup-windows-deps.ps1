<#
.SYNOPSIS
    Fetches the prebuilt native libraries Concat builds against on Windows, at
    the exact versions and digests the repository's own CI pins.

.DESCRIPTION
    Concat spawns nothing at run time - FFmpeg is linked, whisper.cpp and
    sherpa-onnx are compiled in - so the build needs their development files.
    None are pip or winget packages at the right version, and "latest" is the
    wrong answer for all of them: scripts/ci/deps.lock exists precisely because
    BtbN's daily FFmpeg builds are deleted after a fortnight and `latest`
    changes under a build every day.

    This script reads that lock file rather than hardcoding anything, so it
    cannot drift from CI. For each dependency it:

      1. downloads the pinned file,
      2. checks its sha256 against the lock and DELETES it if they differ,
      3. unpacks it under -Dest,
      4. sets the environment variable the build looks for,
      5. puts the DLL folder on the user PATH, because the test binaries load
         those at run time and not only at link time.

    It is the PowerShell equivalent of the "LLVM (Windows)", "FFmpeg (Windows)",
    "ONNX Runtime (Windows)" and "sherpa-onnx libraries" steps of the desktop
    job in .github/workflows/ci.yml.

.PARAMETER Repo
    Path to the Concat checkout. Defaults to the parent of this script's folder.

.PARAMETER Dest
    Where to unpack. Defaults to C:\Concat-deps.

.PARAMETER LlvmDir
    Where LLVM installs. Defaults to C:\llvm, matching CI.

.PARAMETER Skip
    Comma-separated names to skip: ffmpeg, ort, sherpa, llvm.

.PARAMETER NoUserEnv
    Print the values instead of setting them with setx / SetEnvironmentVariable.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File setup-windows-deps.ps1

.EXAMPLE
    # Only FFmpeg, into a folder of its own, changing no environment:
    powershell -ExecutionPolicy Bypass -File setup-windows-deps.ps1 `
        -Skip ort,sherpa,llvm -Dest D:\ffmpeg-dev -NoUserEnv

.NOTES
    Needs Windows 10 1803+ for tar.exe (which also unpacks .tar.bz2 and .tar.xz)
    and PowerShell 5.1+. LLVM's installer is run silently with /S.
#>
#Requires -Version 5.1
[CmdletBinding()]
param(
    # Left empty on purpose and resolved below the param block. $PSScriptRoot is
    # blank when a script is run from the ISE, pasted in, or invoked through
    # Invoke-Expression rather than -File, and Join-Path refuses an empty -Path
    # with an error that names the parameter rather than the cause.
    [string]$Repo    = "",
    [string]$Dest    = "C:\Concat-deps",
    [string]$LlvmDir = "C:\llvm",
    [string[]]$Skip  = @(),
    [switch]$NoUserEnv
)

$ErrorActionPreference = "Stop"
$ProgressPreference    = "SilentlyContinue"   # Invoke-WebRequest is far slower with a progress bar

# ── Where am I, and where is the repository ──────────────────────────────────
# The folder holding this script, found without $PSScriptRoot, which is empty
# under the ISE and when a script is pasted rather than run with -File. The
# fallbacks are the command's own path and then the caller's directory.
$self = if ($PSScriptRoot) { $PSScriptRoot }
        elseif ($MyInvocation.MyCommand.Path) { Split-Path -Parent $MyInvocation.MyCommand.Path }
        else { (Get-Location).Path }

if ($host.Name -match 'ISE') {
    Write-Warning "Running from the PowerShell ISE. `$PSScriptRoot is empty there, so this script fell back to the current directory. It works, but run it from a normal PowerShell window instead:"
    Write-Warning "    powershell -ExecutionPolicy Bypass -File `"$self\$(Split-Path -Leaf $MyInvocation.MyCommand.Path)`""
}

$marker = Join-Path "scripts" "ci" "deps.lock"

# True when a folder is a Concat checkout, judged by the one file this script
# cannot work without.
function Test-Repo([string]$dir) {
    return $dir -and (Test-Path (Join-Path $dir $marker))
}

# Candidates in the order they should be believed: this script's own folder,
# then up to two levels (so it works from the repo root, from a subfolder of it,
# and from a folder holding the checkout), then wherever the caller is standing,
# then one level down from there - which is what finds a checkout that was
# unzipped into a folder of its own, as `concat-main` is.
$candidates = New-Object System.Collections.Generic.List[string]
$candidates.Add($self)
$up = $self
for ($i = 0; $i -lt 2; $i++) {
    $parent = Split-Path -Parent $up
    if (-not $parent -or $parent -eq $up) { break }
    $candidates.Add($parent)
    $up = $parent
}
$cwd = (Get-Location).Path
if ($cwd -ne $self) { $candidates.Add($cwd) }
foreach ($sub in (Get-ChildItem $cwd -Directory -ErrorAction SilentlyContinue)) {
    $candidates.Add($sub.FullName)
}

if (-not $Repo) {
    foreach ($c in $candidates) { if (Test-Repo $c) { $Repo = $c; break } }
}
if (-not (Test-Repo $Repo)) {
    $tried = ($candidates | Select-Object -Unique) -join "`n    "
    throw @"
Could not find the Concat checkout - no folder held scripts\ci\deps.lock.
Looked in:
    $tried
Pass it explicitly:
    -Repo C:\path\to\Concat
"@
}
$Repo = (Resolve-Path $Repo).Path
Write-Host "repository: $Repo" -ForegroundColor DarkGray

$lock = Join-Path $Repo "scripts\ci\deps.lock"

# ONNX Runtime's version has to agree with the one the `ort` crate was built
# against. CI sets it as a workflow variable and deps.lock pins the file; this
# reads the lock's own URL so the two cannot disagree here.
New-Item -ItemType Directory -Force -Path $Dest | Out-Null

function Say([string]$text) { Write-Host ""; Write-Host "== $text" -ForegroundColor Cyan }
function Note([string]$text) { Write-Host "   $text" }
function Skip([string]$name) { return $Skip -contains $name }

# ── The lock file ────────────────────────────────────────────────────────────
#   name  url  sha256
# Comments and blank lines aside, one dependency per row.
function Get-Row([string]$name) {
    $row = Get-Content $lock |
        Where-Object { $_ -notmatch '^\s*(#|$)' } |
        Where-Object { ($_ -split '\s+')[0] -eq $name } |
        Select-Object -First 1
    if (-not $row) { throw "no row named '$name' in deps.lock" }
    $parts = $row -split '\s+'
    [pscustomobject]@{ Url = $parts[1]; Sha256 = $parts[2] }
}

function Get-Sha256([string]$path) {
    (Get-FileHash -Algorithm SHA256 -Path $path).Hash.ToLowerInvariant()
}

# Downloads and verifies, the way scripts/ci/fetch.sh does: a file already there
# with the right digest is kept, and one whose digest differs is deleted rather
# than built against.
function Fetch([string]$name, [string]$out) {
    $row = Get-Row $name
    if ((Test-Path $out) -and ((Get-Sha256 $out) -eq $row.Sha256)) {
        Note "$name is already at $out"
        return
    }
    Note "downloading $name"
    Note "  $($row.Url)"
    # --retry 5 --retry-all-errors, as fetch.sh does.
    for ($attempt = 1; ; $attempt++) {
        try {
            Invoke-WebRequest -Uri $row.Url -OutFile $out -UseBasicParsing
            break
        } catch {
            if ($attempt -ge 5) { throw "the $name download failed after $attempt attempts: $_" }
            Note "  attempt $attempt failed, retrying: $_"
            Start-Sleep -Seconds (2 * $attempt)
        }
    }
    $got = Get-Sha256 $out
    if ($got -ne $row.Sha256) {
        Remove-Item -Force $out
        throw @"
$name does not match deps.lock
  url:      $($row.Url)
  expected: $($row.Sha256)
  got:      $got
"@
    }
    Note "verified (sha256 $got)"
}

# tar.exe ships with Windows 10 1803+ and unpacks .zip, .tar.gz, .tar.xz and
# .tar.bz2 - which is why no 7-Zip is needed for sherpa's .tar.bz2.
function Unpack([string]$archive, [string]$into) {
    New-Item -ItemType Directory -Force -Path $into | Out-Null
    Note "unpacking into $into"
    & tar -xf $archive -C $into
    if ($LASTEXITCODE -ne 0) { throw "tar could not unpack $archive (exit $LASTEXITCODE)" }
}

# The one folder inside a parent, which is what both the FFmpeg and the ONNX
# archives unpack to.
function Inner([string]$parent) {
    $dirs = Get-ChildItem $parent -Directory
    if ($dirs.Count -lt 1) { throw "nothing unpacked into $parent" }
    $dirs[0].FullName
}

$envVars  = @{}     # name -> value, for the summary and for -NoUserEnv
$pathAdds = @()     # folders the test binaries need on PATH

# ── 1. LLVM: libclang for bindgen ────────────────────────────────────────────
# CI installs LLVM's own build into a folder of its own rather than trusting
# whatever the machine carries, and this does the same.
if (Skip "llvm") {
    Say "LLVM (skipped)"
} elseif (Test-Path (Join-Path $LlvmDir "bin\libclang.dll")) {
    Say "LLVM"
    Note "already installed at $LlvmDir"
} else {
    Say "LLVM"
    $exe = Join-Path $env:TEMP "LLVM-win64.exe"
    Fetch "llvm-win64" $exe
    Note "installing silently to $LlvmDir (a few minutes)"
    Start-Process -Wait -FilePath $exe -ArgumentList "/S", "/D=$LlvmDir"
    if (-not (Test-Path (Join-Path $LlvmDir "bin\libclang.dll"))) {
        throw "LLVM did not install to $LlvmDir"
    }
    Remove-Item -Force $exe
}
$envVars["LIBCLANG_PATH"] = Join-Path $LlvmDir "bin"
$pathAdds += Join-Path $LlvmDir "bin"

# ── 2. FFmpeg 7+ development libraries ───────────────────────────────────────
# BtbN's GPL *shared* build: headers and import libraries to link against, DLLs
# to run the tests with. FFMPEG_DIR is what ffmpeg-sys-the-third reads when
# pkg-config finds nothing.
if (Skip "ffmpeg") {
    Say "FFmpeg (skipped)"
} else {
    Say "FFmpeg"
    $zip = Join-Path $env:TEMP "ffmpeg-win64.zip"
    Fetch "ffmpeg-win64" $zip
    $stage = Join-Path $Dest "ffmpeg-stage"
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    Unpack $zip $stage
    $inner = Inner $stage
    $ffmpeg = Join-Path $Dest "ffmpeg"
    if (Test-Path $ffmpeg) { Remove-Item -Recurse -Force $ffmpeg }
    Move-Item $inner $ffmpeg
    Remove-Item -Recurse -Force $stage
    Remove-Item -Force $zip
    if (-not (Test-Path (Join-Path $ffmpeg "lib\avcodec.lib"))) {
        throw "avcodec.lib is not in $ffmpeg\lib - the archive's layout changed"
    }
    $envVars["FFMPEG_DIR"] = $ffmpeg
    $pathAdds += Join-Path $ffmpeg "bin"
    Note "avcodec.lib found"
}

# ── 3. ONNX Runtime ──────────────────────────────────────────────────────────
# Microsoft's shared build, linked dynamically. ci.yml gives the reason: a
# static one beside sherpa-onnx's is the same protobuf twice, and the test
# binary does not link.
if (Skip "ort") {
    Say "ONNX Runtime (skipped)"
} else {
    Say "ONNX Runtime"
    $zip = Join-Path $env:TEMP "onnxruntime-win-x64.zip"
    Fetch "onnxruntime-win-x64" $zip
    $stage = Join-Path $Dest "ort-stage"
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    Unpack $zip $stage
    $inner = Inner $stage
    $ort = Join-Path $Dest "ort"
    if (Test-Path $ort) { Remove-Item -Recurse -Force $ort }
    Move-Item $inner $ort
    Remove-Item -Recurse -Force $stage
    Remove-Item -Force $zip
    $lib = Join-Path $ort "lib"
    if (-not (Test-Path (Join-Path $lib "onnxruntime.lib"))) {
        throw "onnxruntime.lib is not in $lib"
    }
    $envVars["ORT_LIB_LOCATION"]        = $lib
    $envVars["ORT_PREFER_DYNAMIC_LINK"] = "1"
    $pathAdds += $lib
    Note "onnxruntime.lib found"
}

# ── 4. sherpa-onnx, with the .lib and .dll split CI does ─────────────────────
# concat-speech/Cargo.toml explains both halves of this at length:
#
#   * the shared build, because k2-fsa's only static archives are /MT and MSVC
#     refuses to link those into a binary whose every other object is /MD
#     (LNK2038, on every object pair);
#   * SHERPA_ONNX_LIB_DIR must hold ONLY the import .lib files, because left to
#     its own download the sys crate finds the DLLs beside them and tries to
#     copy them into a "profile directory" it locates by walking up for an
#     ancestor named after $PROFILE - a walk that panics under the custom `app`
#     profile.
#
# So the .dll files go to their own folder, which is on PATH and is not the lib
# directory.
if (Skip "sherpa") {
    Say "sherpa-onnx (skipped)"
} else {
    Say "sherpa-onnx"

    # The version has to be the one src/Cargo.lock resolves for sherpa-onnx-sys;
    # CI fails the build when the two disagree, and so does this.
    $lockFile = Join-Path $Repo "src\Cargo.lock"
    $version = $null
    $lines = Get-Content $lockFile
    for ($i = 0; $i -lt $lines.Count - 1; $i++) {
        if ($lines[$i] -match 'name = "sherpa-onnx-sys"') {
            if ($lines[$i + 1] -match 'version = "([^"]+)"') { $version = $Matches[1] }
            break
        }
    }
    if (-not $version) { throw "sherpa-onnx-sys is not in $lockFile" }

    $name = "sherpa-onnx-win-x64-shared-MT-Release-lib"
    $row  = Get-Row $name
    if (-not $row.Url.Contains("/v$version/")) {
        throw "deps.lock pins $name for another version than Cargo.lock's $version"
    }
    Note "Cargo.lock wants $version and deps.lock pins it"

    $bz2 = Join-Path $env:TEMP "sherpa.tar.bz2"
    Fetch $name $bz2
    $stage = Join-Path $Dest "sherpa-stage"
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    Unpack $bz2 $stage

    $stem = "sherpa-onnx-v$version-win-x64-shared-MT-Release-lib"
    $srcLib = Join-Path $stage "$stem\lib"
    if (-not (Test-Path $srcLib)) { throw "$srcLib is not in the archive" }

    $libs  = Join-Path $Dest "sherpa-libs"
    $dlls  = Join-Path $Dest "sherpa-dlls"
    foreach ($d in @($libs, $dlls)) {
        if (Test-Path $d) { Remove-Item -Recurse -Force $d }
        New-Item -ItemType Directory -Force -Path $d | Out-Null
    }
    Copy-Item (Join-Path $srcLib "sherpa-onnx-*.lib") $libs
    Copy-Item (Join-Path $srcLib "sherpa-onnx-*.dll") $dlls

    Remove-Item -Recurse -Force $stage
    Remove-Item -Force $bz2

    if (-not (Get-ChildItem $libs -Filter *.lib)) { throw "no .lib files reached $libs" }
    if (Get-ChildItem $libs -Filter *.dll) {
        throw "a .dll reached $libs - the split that stops the `$PROFILE panic failed"
    }
    $envVars["SHERPA_ONNX_LIB_DIR"] = $libs
    $pathAdds += $dlls
    Note "$((Get-ChildItem $libs -Filter *.lib).Count) .lib files, $((Get-ChildItem $dlls -Filter *.dll).Count) .dll files, kept apart"
}

# ── 5. Write it down ─────────────────────────────────────────────────────────
Say "Environment"

if ($NoUserEnv) {
    Note "not set, because -NoUserEnv was given. Set these yourself:"
    Note ""
    foreach ($k in ($envVars.Keys | Sort-Object)) {
        Note "  [Environment]::SetEnvironmentVariable(`"$k`", `"$($envVars[$k])`", `"User`")"
    }
} else {
    foreach ($k in ($envVars.Keys | Sort-Object)) {
        [Environment]::SetEnvironmentVariable($k, $envVars[$k], "User")
        # And in this session, so a build in the same window works at once.
        Set-Item -Path "Env:$k" -Value $envVars[$k]
        Note "  $k = $($envVars[$k])"
    }
}

# PATH is one variable, so it is read, merged and written once: appending with
# setx would truncate it at 1024 characters and lose whatever was already there.
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($null -eq $userPath) { $userPath = "" }
$have  = $userPath -split ';' | Where-Object { $_ }
$added = @()
foreach ($dir in ($pathAdds | Select-Object -Unique)) {
    if ($have -notcontains $dir) { $added += $dir; $have += $dir }
    if (-not $NoUserEnv) {
        # This session too, so the test binaries find the DLLs at once.
        $env:Path = "$dir;$env:Path"
    }
}
if ($added -and -not $NoUserEnv) {
    [Environment]::SetEnvironmentVariable("Path", ($have -join ';'), "User")
}

Note ""
if ($added) {
    Note "added to the user PATH:"
    foreach ($dir in $added) { Note "  $dir" }
} else {
    Note "the user PATH already carried every folder needed"
}

Say "Done"
Write-Host @"

  Open a NEW terminal - environment changes do not reach the one that made
  them - then:

      cd $Repo\src
      cargo test -p concat-project      # pure logic; needs no native library
      cargo build -p concat-media       # proves FFmpeg linked
      cargo dev                         # the editor window

  The first full build is long: CI runs it as two parallel jobs with a
  60-minute timeout, and a comment in ci.yml says one took most of an hour.
  If rustc or the linker is killed, that is memory, not a code error - drop
  the parallelism with -j 2.

  See RUN-CONCAT-LOCAL.md for the failures worth checking first.
"@ -ForegroundColor Green
