# SETUP-STEPS.md — run these in order

**Read this first: `requirements.txt` has nothing to do with building Concat.**

Concat is Rust. It needs no Python to build, run or edit. `requirements.txt` is
only for two *optional* side quests — running Video Trim Studio itself, and
verifying the port against the Python original. If your goal is "run Concat on my
PC", **skip step 6 entirely.**

Order below: toolchain → repo → native libraries → verify → the patch → Python
only if you want it.

| Step | What | Time | Needed to run Concat? |
|---:|---|---|---|
| 1 | Git + VS Build Tools + cmake + LLVM | 15–40 min | **yes** |
| 2 | Clone Concat | 2 min | **yes** |
| 3 | LLVM / libclang | 5 min | **yes** |
| 4 | `setup-windows-deps.ps1` | 10 min | **yes** |
| 5 | New terminal + verify | 1 min | **yes** |
| 6 | `requirements.txt` | 5 min | **no — optional** |
| 7 | Apply the Tranche A patch | 2 min | only for the port |
| 8 | First build + run | 10 min – 1 hr | **yes** |

**Total to a running editor: roughly 45 minutes of installing, then one long
first build.**

---

## Step 1 — Git, the C++ toolchain, cmake

Two ways. **Use the direct downloads unless you already have `winget`** — two of
the four come out better pinned, and `winget` is missing on many Windows 10
installs anyway (it ships inside App Installer, which Windows 10 only carries on
21H2+ images).

Open **PowerShell as Administrator** for either route. Keep this window open for
steps 1–4.

### 1A — Direct downloads (recommended)

See **`NO-WINGET.md`** for this route in full, including verify commands. The
short version — four files, all URLs checked reachable:

| Tool | Download | Size |
|---|---|---|
| Git | `github.com/git-for-windows/git/releases/download/v2.56.0.windows.2/Git-2.56.0.2-64-bit.exe` | 67 MB |
| **VS 2022 Build Tools** | `aka.ms/vs/17/release/vs_BuildTools.exe` | 2–7 GB |
| **CMake 3.31.12** | `github.com/Kitware/CMake/releases/download/v3.31.12/cmake-3.31.12-windows-x86_64.msi` | 34 MB |
| LLVM 21.1.0 | `github.com/llvm/llvm-project/releases/download/llvmorg-21.1.0/LLVM-21.1.0-win64.exe` | ~350 MB |

In the VS installer tick exactly one workload: **Desktop development with C++**.
That brings MSVC v143 (`cl.exe`, `link.exe`) and the Windows 11 SDK — the
`x86_64-pc-windows-msvc` target has no alternative, so this is not skippable.

In the CMake installer choose *"Add CMake to the system PATH for all users"*.

> ⚠️ **Do not install the latest CMake.** `whisper-rs-sys` vendors whisper.cpp
> 1.8.3, whose `CMakeLists.txt` opens with `cmake_minimum_required(VERSION 3.5)`,
> and **CMake 4.0 removed compatibility with anything below 3.5**. The escape
> hatch is `-DCMAKE_POLICY_VERSION_MINIMUM=3.5`, which `whisper-rs-sys`'s
> `build.rs` does not pass (verified — it defines `BUILD_SHARED_LIBS`,
> `WHISPER_BUILD_TESTS`, `WHISPER_COREML`, `GGML_CUDA` and nothing else). CMake
> 4.x therefore fails at the first crate that needs it, with an error pointing at
> whisper.cpp rather than at your cmake version. **3.31.12 is the last 3.x.**
>
> Check with `cmake --version` — the first line must read 3.x.

LLVM is pinned to **21.1.0** because that is what `scripts/ci/deps.lock` names,
so you match CI. You can skip this download and let
`setup-windows-deps.ps1` fetch and install it to `C:\llvm` instead (step 4).

**Verify the C++ toolchain:**

```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
  -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
  -property installationPath
```

Expected: a path ending in `...\2022\BuildTools`. Empty output means the C++
workload did not install — rerun the bootstrapper and watch for an error.

### 1B — winget, if you have it

```powershell
winget --version    # if this fails, use route 1A, or see NO-WINGET.md
```

```powershell
winget install --id Git.Git -e
winget install --id Microsoft.VisualStudio.2022.BuildTools --override `
  "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Then **CMake and LLVM by direct download anyway**, because winget gives you
CMake 4.x (breaks whisper.cpp) and an unpinned LLVM (CI uses 21.1.0). Only Git
and Build Tools are genuinely better through winget.

If you want `winget` itself, `NO-WINGET.md` has the install — App Installer
v1.29.380 plus its two framework dependencies, which are what the
`0x80073CF3` failure is about.

---

## Step 2 — Clone Concat

The setup script reads the repository's own `scripts/ci/deps.lock`, so the repo
must exist before step 4. Anywhere you like; this guide assumes `C:\dev`.

```powershell
cd C:\dev
git clone https://github.com/jub0t/Concat.git
cd Concat
git log --oneline -1
```

Expected: `2d3a43e fix(release): the Windows smoke test and its emulated Ivy...`
(or newer — that is the commit the Tranche A patch was built against).

---

## Step 3 — LLVM / libclang

`bindgen` needs `libclang.dll` to generate the FFmpeg FFI. Two ways; pick one.

**Option A — winget (simpler, recommended):**

```powershell
winget install --id LLVM.LLVM -e
```

Installs to `C:\Program Files\LLVM` by default, but confirm where
`libclang.dll` actually landed rather than assuming — that one file is the whole
reason LLVM is here:

```powershell
Get-ChildItem "C:\Program Files\LLVM\bin\libclang.dll" -ErrorAction SilentlyContinue
# if that prints nothing, find it:
Get-ChildItem C:\ -Recurse -Filter libclang.dll -ErrorAction SilentlyContinue |
  Select-Object -First 3 FullName
```

**Note the folder that contains it**, because in step 4 you point the script at
that folder's *parent* (the one containing `bin\`):

```powershell
# add this to the step 4 command:
#   -Skip llvm -LlvmDir "C:\Program Files\LLVM"
```

The script sets `LIBCLANG_PATH` to `<LlvmDir>\bin`, so `-LlvmDir` is the LLVM
root, not the `bin` folder.

**Option B — let the script do it**, exactly as CI does: downloads LLVM 21.1.0
from `deps.lock` (~350 MB), checks its sha256, and installs it silently to
`C:\llvm`. It will raise a **UAC prompt** — click Yes. Then in step 4 pass
nothing extra.

Option A is one less large download and one less UAC prompt. Option B matches CI
byte for byte.

---

## Step 4 — Where does `setup-windows-deps.ps1` go?

**Answer: at the root of your Concat clone — `C:\dev\Concat\setup-windows-deps.ps1`,
next to `ARCHITECTURE.md` and `README.md`.** That is the folder containing
`scripts\ci\deps.lock`.

The script works out the repo by itself, in this order:

1. Is `scripts\ci\deps.lock` in the **script's own folder**? → use that folder.
2. Otherwise, is it in the **parent** of the script's folder? → use the parent.
3. Otherwise it throws and tells you to pass `-Repo`.

So all three of these work:

| You put it at | Repo resolves to | Works? |
|---|---|---|
| `C:\dev\Concat\setup-windows-deps.ps1` | `C:\dev\Concat` (rule 1) | ✅ **recommended** |
| `C:\dev\Concat\tools\setup-windows-deps.ps1` | `C:\dev\Concat` (rule 2) | ✅ any subfolder of the clone |
| `C:\dev\setup-windows-deps.ps1` | `C:\dev` | ❌ needs `-Repo C:\dev\Concat` |
| `C:\dev\Concat\scripts\ci\...ps1` | `C:\dev\Concat\scripts\ci` | ❌ two levels is one too many |
| `C:\Users\me\Desktop\...ps1` | `C:\Users\me\Desktop` | ❌ needs `-Repo C:\dev\Concat` |

Two of those are worth spelling out. Putting it in `C:\dev` *beside* the clone
does **not** work, because rule 2 looks at `C:\dev`, not inside it. And
`scripts\ci\` — the folder the lock file is actually in — also fails, because
rule 2 only ever goes up **one** level. Root of the clone, any single subfolder
of it, or pass `-Repo`.

Copy it there and run it **in the same Administrator PowerShell**:

Extract `concat-local-setup.zip` anywhere, then copy just the `.ps1` into the
clone's root — adjust the source path to wherever you unzipped it:

```powershell
cd C:\dev\Concat
Copy-Item "<wherever you unzipped>\concat-local-setup\setup-windows-deps.ps1" .

# confirm it landed beside the lock file's repo, not inside a subfolder:
Test-Path .\setup-windows-deps.ps1      # expected: True
Test-Path .\scripts\ci\deps.lock       # expected: True
```

# If you used Option A for LLVM in step 3:
powershell -ExecutionPolicy Bypass -File .\setup-windows-deps.ps1 `
  -Skip llvm -LlvmDir "C:\Program Files\LLVM"

# If you used Option B (let the script install LLVM):
powershell -ExecutionPolicy Bypass -File .\setup-windows-deps.ps1
```

`-ExecutionPolicy Bypass` is required — Windows blocks unsigned `.ps1` files by
default. The flag applies to that one process and changes no system setting.

**What it does**, in order, printing as it goes:

1. Reads `scripts\ci\deps.lock` — it does not hardcode a single URL or version.
2. Downloads FFmpeg 8.1 (BtbN GPL shared), ONNX Runtime 1.28.0, sherpa-onnx
   1.13.7. Each is **sha256-verified against the lock file and deleted if the
   digest differs**, so a tampered or truncated download fails loudly instead of
   producing a broken build.
3. Cross-checks that `deps.lock`'s sherpa-onnx version equals the one
   `src\Cargo.lock` resolves for `sherpa-onnx-sys` — the same check CI fails the
   build on.
4. Unpacks to `C:\Concat-deps\`, and for sherpa-onnx **splits the `.lib` files
   from the `.dll` files into two folders**. This matters: see §4 of
   `RUN-CONCAT-LOCAL.md`. Mixing them causes either `LNK2038` or a panic that
   mentions `$PROFILE`.
5. Sets `FFMPEG_DIR`, `ORT_LIB_LOCATION`, `ORT_PREFER_DYNAMIC_LINK`,
   `SHERPA_ONNX_LIB_DIR`, `LIBCLANG_PATH` at User scope, and merges the four DLL
   folders into the user PATH by read-modify-write (appending with `setx` would
   truncate PATH at 1024 characters and lose what was already there).

**Verify it finished:** the last block prints `== Done` in green and lists the
next commands.

**Useful flags:**

```powershell
-NoUserEnv              # print the values, change nothing on your machine
-Skip ffmpeg,ort        # do only some of them
-Dest D:\Concat-deps    # unpack somewhere else
```

---

## Step 5 — New terminal, then verify

**Close every open terminal and start a fresh one.** Environment changes do not
reach a window that is already open. A plain PowerShell is fine now — no admin.

```powershell
echo $env:FFMPEG_DIR
echo $env:ORT_LIB_LOCATION
echo $env:ORT_PREFER_DYNAMIC_LINK
echo $env:SHERPA_ONNX_LIB_DIR
echo $env:LIBCLANG_PATH
```

Expected, all five non-empty:

```
C:\Concat-deps\ffmpeg
C:\Concat-deps\ort\lib
1
C:\Concat-deps\sherpa-libs
C:\Program Files\LLVM\bin        (or C:\llvm\bin)
```

Then the split that prevents the two worst failures:

```powershell
# .lib files only — no DLLs here
Get-ChildItem $env:SHERPA_ONNX_LIB_DIR -Filter *.dll
# expected: no output at all

Get-ChildItem $env:SHERPA_ONNX_LIB_DIR -Filter *.lib | Measure-Object | % Count
# expected: a number greater than zero
```

Any `.dll` in that folder is the `$PROFILE` panic waiting to happen. Move it out.

Now prove the toolchain and the pinned Rust version:

```powershell
cd C:\dev\Concat\src
rustup show
```

Expected: an active toolchain line naming **1.93.1** — `src\rust-toolchain.toml`
pins it, so rustup downloads it on your first cargo command here (a few hundred
MB, once). Your `rustup-init` default is *not* what runs in this folder.

And the one build that needs no native library at all — this is the fastest
possible proof that your toolchain works:

```powershell
cargo test -p concat-project
```

Expected on unmodified `main`: `test result: ok. 114 passed; 0 failed`.
(`concat-project` depends only on `concat-core`, serde and serde_json — no
FFmpeg, no cmake, no GPU. If this fails, nothing later will work, and the cause
is the toolchain.)

---

## Step 6 — `requirements.txt` (OPTIONAL — skip if you just want Concat)

Only for Video Trim Studio and for checking the port against the Python original.

```powershell
winget install --id Python.Python.3.12 -e
# new terminal, then:
cd C:\dev
git clone https://github.com/itsukaitsutsu/video-trim-studio.git VTS
python -m venv VTS\.venv
VTS\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

If `Activate.ps1` is blocked:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
```

**Two notes.** `faster-whisper` runs on CPU and NVIDIA CUDA only — it cannot use
your RX 6600 XT; the AMD path is whisper.cpp with Vulkan, which needs no Python
package. And the four OTIO packages must all be present, because bare
`opentimelineio` ships no NLE formats:

```powershell
python -c "import opentimelineio as otio; a=set(otio.adapters.available_adapter_names()); print(all(x in a for x in ['fcpx_xml','fcp_xml','cmx_3600','AAF']))"
# expected: True
```

**Do not `pip install -r requirements.txt` and then try to build Concat.** They
are unrelated, and no Python package provides FFmpeg's development libraries.

---

## Step 7 — Apply the Tranche A patch (only if you want the port)

```powershell
cd C:\dev\Concat
git checkout -b vts-tranche-a
git config user.name  "Your Name"
git config user.email "you@example.com"
git am C:\path\to\vts-concat-tranche-a\patch\0001-tranche-a.patch
git commit --amend --author="Your Name <you@example.com>" --no-edit
```

`git config` is needed first — `git am` refuses to run with no committer
identity, which is exactly the error it produced in testing.

**Verify:**

```powershell
cd src
cargo test -p concat-project
```

Expected: `test result: ok. 215 passed; 0 failed` — 114 existing plus 101 new.

```powershell
cargo fmt -p concat-project -- --check      # expected: no output
cargo clippy -p concat-project --all-targets -- -D warnings
```

The clippy line may report **three errors in `concat-core`** —
`frame.rs:130`, `frame.rs:298`, `shader.rs:162` — all
`chunks_exact_to_as_chunks`. That lint does not exist in the pinned 1.93.1; it
arrived in a newer clippy. **They are pre-existing and not from this patch.** If
your rustup fetched 1.93.1 as pinned you should not see them at all; if you do,
add `-A clippy::chunks_exact_to_as_chunks` to see your own result.

Prefer not to use `git am`? Copy the three new `.rs` files into
`src\crates\concat-project\src\` and add three `pub mod` lines to `lib.rs` — see
the README in that zip.

---

## Step 8 — First build and run

**Use the "x64 Native Tools Command Prompt for VS 2022"** for this (Start menu,
installed by step 1). It has `cl.exe` and `link.exe` on PATH. Ordinary PowerShell
usually works because cargo finds the toolchain via the registry — but when a C
build script fails with `link.exe not found`, this is the fix.

Build in this order, because each step needs less than the next and a failure
names itself:

```powershell
cd C:\dev\Concat\src

# (a) pure logic — already done in step 5
cargo test -p concat-project

# (b) proves FFmpeg linked. Needs FFMPEG_DIR right.
cargo build -p concat-media

# (c) the CLI — needs FFmpeg, no GUI, no audio device
cargo run -p concat-cli -- probe some-video.mp4

# (d) the editor window, optimised for iterating
cargo dev

# (e) only if you want the whole thing
cargo build --workspace --all-targets
```

`cargo dev` is an alias in `src\.cargo\config.toml` for
`run --bin concat --profile quick`: an optimised engine with an unoptimised
window crate, so a Rust edit rebuilds in ~20 s and a `.slint` edit in ~25 s
instead of 3 m 20 s.

**Budget honestly for (e).** CI runs the full workspace build as **two parallel
jobs with a 60-minute timeout**, and a comment in `ci.yml` says one took most of
an hour. On 16 GB, if rustc or the linker gets killed, that is memory and not a
code error — drop the parallelism: `cargo build --workspace -j 2`.

---

## When something breaks

**`link.exe not found` / `cc not found`** → build from the x64 Native Tools
Command Prompt (§8). `where cl` should answer.

**`libavutil.pc not found` or a pkg-config error** → `FFMPEG_DIR` is unset or
wrong in *this* session. `echo $env:FFMPEG_DIR`. Then
`cargo build -p concat-media -vv` prints the FFmpeg probe and names the problem.

**`LNK2038` mismatch, `/MT` vs `/MD`** → a static sherpa-onnx library got linked.
It must be the `win-x64-shared-MT-Release-lib` build the script fetches.

**A panic mentioning `$PROFILE` or a "profile directory"** → `.dll` files are
sitting next to the `.lib` files in `SHERPA_ONNX_LIB_DIR`. Check with the
`Get-ChildItem -Filter *.dll` command in step 5; the folder must return nothing.

**`libclang.dll not found`** → `LIBCLANG_PATH` is wrong. Check it points at a
folder that actually contains `libclang.dll`.

**A test binary can't start / missing DLL** → the DLL folders are on PATH for
building but not for running. They must be on the *user* PATH, which the script
does; if you set the variables by hand you skipped that half.

**`cargo nextest run --workspace` goes red** → possibly not your fault.
ARCHITECTURE §15 records that **the Windows CI test job has not passed since
4 October** and the reason is not visible in the public logs. Establish the
baseline first: run the suite on a clean `main` checkout *before* applying step 7
and compare. One test, `every_edit_still_exports`, is singled out in CI as having
hung on Windows alone.

**Full checklist of what could be wrong, in order:** are you in `src\`? Is
`cl.exe` reachable? Are all five variables set *in this session*? Is
`SHERPA_ONNX_LIB_DIR` free of DLLs?
