# NO-WINGET.md — install the four tools without winget

`winget` is not a separate program. It ships inside **App Installer**
(`Microsoft.DesktopAppInstaller`), which Windows 10 only carries on 21H2+ images —
and LTSC / IoT / older installs skip it entirely. On an i7-10700F-era build that
is the expected situation, not a broken one.

**You do not need it.** Every tool step 1 wanted is a plain `.exe` or `.msi`.
That is the shorter path, and it is below.

All four URLs were checked reachable on 2026-10-13.

---

## The CMake trap — read this one

**Install CMake 3.31.12. Do not install the latest CMake.**

`whisper-rs-sys` builds whisper.cpp with cmake, and the copy it vendors
(v1.8.3) opens with:

```cmake
cmake_minimum_required(VERSION 3.5)
```

**CMake 4.0 removed compatibility with anything below 3.5** and errors out with
*"Compatibility with CMake < 3.5 has been removed"*. The escape hatch is
`-DCMAKE_POLICY_VERSION_MINIMUM=3.5`, and `whisper-rs-sys`'s `build.rs` does not
pass it — I checked; it defines `BUILD_SHARED_LIBS`, `WHISPER_BUILD_TESTS`,
`WHISPER_COREML`, `GGML_CUDA` and nothing else. So CMake 4.x fails the build at
the first crate that needs it, with an error that points at whisper.cpp and not
at your cmake version.

3.31.12 is the last 3.x release. It is also what the ecosystem still targets.

---

## Install, in order

Download all four, then run them. Each needs administrator rights.

### 1. Git for Windows — 67 MB

```
https://github.com/git-for-windows/git/releases/download/v2.56.0.windows.2/Git-2.56.0.2-64-bit.exe
```

v2.56.0.windows.2 is current. Accept the defaults. **This also gives you Git
Bash**, which `scripts/ci/fetch.sh` needs if you ever run it — and which is the
shell the setup script's tar calls behave best in.

### 2. Visual Studio 2022 Build Tools — the large one, 2–7 GB

```
https://aka.ms/vs/17/release/vs_BuildTools.exe
```

That is Microsoft's own permanent link to the current VS 2022 bootstrapper, so
it does not go stale. Run it, and in the installer tick exactly one workload:

> **Desktop development with C++**

Leave the optional components on the right at their defaults — that selection
brings MSVC v143 (`cl.exe`, `link.exe`) and the Windows 11 SDK, which is all
Concat's `x86_64-pc-windows-msvc` target needs. There is no alternative to MSVC
for that target, so this step is not skippable.

Prefer a command line? The bootstrapper takes the same switch the CI would:

```powershell
.\vs_BuildTools.exe --wait --passive `
  --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended
```

**Verify when it finishes:**

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  -property installationPath
```

Expected: a path ending in `...\2022\BuildTools`. Empty output means the C++
workload did not install.

### 3. CMake 3.31.12 — 34 MB

```
https://github.com/Kitware/CMake/releases/download/v3.31.12/cmake-3.31.12-windows-x86_64.msi
```

**3.31.12, not 4.x** — see the trap above. In the installer, choose *"Add CMake
to the system PATH for all users"*.

If you already installed CMake 4.x, check and downgrade:

```powershell
cmake --version
# first line must read 3.x. If it reads 4.x:
#   Settings > Apps > Installed apps > CMake > Uninstall, then install 3.31.12.
```

There is a workaround if you must keep 4.x — `setx CMAKE_POLICY_VERSION_MINIMUM 3.5`
— but I have not verified that `whisper-rs-sys` honours it, so treat it as a last
resort rather than a plan.

### 4. LLVM 21.1.0 — ~350 MB

```
https://github.com/llvm/llvm-project/releases/download/llvmorg-21.1.0/LLVM-21.1.0-win64.exe
```

**21.1.0 specifically** — that is the version `scripts/ci/deps.lock` pins, so you
match CI rather than trusting whatever is current. Install it, and note where it
went. The default is `C:\Program Files\LLVM`.

`bindgen` needs one file from it, `libclang.dll`. Confirm it landed:

```powershell
Get-ChildItem "C:\Program Files\LLVM\bin\libclang.dll"
```

If that prints nothing, find it and use the folder that contains it:

```powershell
Get-ChildItem C:\ -Recurse -Filter libclang.dll -ErrorAction SilentlyContinue |
  Select-Object -First 3 FullName
```

**Skip this download entirely** if you would rather `setup-windows-deps.ps1`
fetched and installed it — that is option B in step 3 of `SETUP-STEPS.md`, and it
puts LLVM at `C:\llvm` exactly as CI does.

---

## Then continue at step 2 of SETUP-STEPS.md

Clone Concat, copy `setup-windows-deps.ps1` to the **root of the clone**, and run
it. Because you installed LLVM yourself, tell the script to skip that one and
where yours is:

```powershell
cd C:\dev\Concat
powershell -ExecutionPolicy Bypass -File .\setup-windows-deps.ps1 `
  -Skip llvm -LlvmDir "C:\Program Files\LLVM"
```

`-LlvmDir` is the LLVM **root**, not the `bin` folder — the script appends `\bin`
when it sets `LIBCLANG_PATH`.

---

## If you would rather have winget anyway

Worth having for later, not worth blocking on. Latest is **v1.29.380**.

```powershell
# As Administrator. The two dependencies first — skipping them is what causes
# the 0x80073CF3 "depends on a framework that could not be found" failure.
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest "https://aka.ms/Microsoft.VCLibs.x64.14.00.Desktop.appx" -OutFile "$env:TEMP\vclibs.appx" -UseBasicParsing
Invoke-WebRequest "https://github.com/microsoft/microsoft-ui-xaml/releases/download/v2.8.6/Microsoft.UI.Xaml.2.8.x64.appx" -OutFile "$env:TEMP\uixaml.appx" -UseBasicParsing
Invoke-WebRequest "https://github.com/microsoft/winget-cli/releases/download/v1.29.380/Microsoft.DesktopAppInstaller_8wekyb3d8bbwe.msixbundle" -OutFile "$env:TEMP\winget.msixbundle" -UseBasicParsing

Add-AppxPackage "$env:TEMP\vclibs.appx"
Add-AppxPackage "$env:TEMP\uixaml.appx"
Add-AppxPackage "$env:TEMP\winget.msixbundle"
```

Then **close every terminal and open a new one** — the PATH entry
(`%LOCALAPPDATA%\Microsoft\WindowsApps`) only reaches new processes.

```powershell
winget --version
```

Or let the release's own dependency bundle do it, which is 98 MB and removes the
guesswork about which frameworks your image lacks:

```
https://github.com/microsoft/winget-cli/releases/download/v1.29.380/DesktopAppInstaller_Dependencies.zip
```

**Still not recognised after installing?** In order:

1. New terminal. This is the most common cause by far.
2. `Get-AppxPackage Microsoft.DesktopAppInstaller` — empty means it did not install.
3. Settings → Apps → Advanced app settings → **App execution aliases** → toggle
   *Windows Package Manager Client* off and on.
4. The direct path works but the bare command does not → `%LOCALAPPDATA%\Microsoft\WindowsApps`
   is missing from your user PATH. Add it.
5. `Get-AppxPackage Microsoft.DesktopAppInstaller | Reset-AppxPackage`

Note that Windows 10 needs build 17763 (1809) or newer for any of this:

```powershell
[System.Environment]::OSVersion.Version    # Major=10, Build must be >= 17763
```

---

## Summary of what changed from the original step 1

| Tool | Original (winget) | Now (direct) |
|---|---|---|
| Git | `winget install Git.Git` | `Git-2.56.0.2-64-bit.exe` |
| VS Build Tools | `winget install ...BuildTools` | `aka.ms/vs/17/release/vs_BuildTools.exe` |
| CMake | `winget install Kitware.CMake` → **would have got 4.4.4 and broken** | **`cmake-3.31.12-windows-x86_64.msi`** |
| LLVM | `winget install LLVM.LLVM` → latest, unpinned | **`LLVM-21.1.0-win64.exe`**, matching `deps.lock` |

Two of the four are better this way regardless of winget: CMake is pinned below
the version that breaks whisper.cpp, and LLVM is pinned to the version CI uses.

---

## Addendum — you downloaded the repo as a ZIP

Recognisable by a folder named `concat-main` in `Downloads`: that is GitHub's
"Download ZIP" of the `main` branch. It works fine for building, with two
consequences worth knowing.

**1. It is not a git repository.** There is no `.git` folder, so `git am` will
not apply the Tranche A patch and `git checkout -b` will not work. Either:

```powershell
# turn it into one, at the commit it came from:
cd C:\Users\E19\Downloads\concat-main
git init
git add -A
git commit -m "import: jub0t/Concat main"
# now git am works
git am C:\path\to\patch\0001-tranche-a.patch
```

or skip git entirely and **copy the three new `.rs` files** into
`src\crates\concat-project\src\`, then add the three `pub mod` lines to `lib.rs`.
The README inside `vts-concat-tranche-a.zip` gives both routes.

**2. `setup-windows-deps.ps1` does not have to be copied anywhere.** Drop it
wherever is convenient — inside `concat-main`, beside it in `Downloads`, or in a
subfolder — and it finds the checkout itself. It looks for the folder holding
`scripts\ci\deps.lock`, checking in this order: its own folder, up to two levels
above that, the directory you launched from, and one level down from there. So
this works from `Downloads` with the script anywhere in or beside the checkout:

```powershell
cd C:\Users\E19\Downloads\concat-main
powershell -ExecutionPolicy Bypass -File .\setup-windows-deps.ps1 `
  -Skip llvm -LlvmDir "C:\Program Files\LLVM"
```

If it cannot find the checkout it says so and **lists every folder it looked
in**, then tells you to pass `-Repo C:\path\to\concat-main`.

### Run it with `-File`, not from the ISE

```powershell
powershell -ExecutionPolicy Bypass -File .\setup-windows-deps.ps1 ...
```

The PowerShell **ISE** leaves `$PSScriptRoot` empty, as does pasting the script's
contents into a prompt or piping it through `Invoke-Expression`. The script now
detects that and warns instead of failing, falling back to the current directory
— but `-File` from a normal PowerShell window is the reliable way, and it is also
what makes the UAC prompt for LLVM behave if you let the script install it.

Windows PowerShell 5.1 is fine. PowerShell 7 (`pwsh`) is not required.
