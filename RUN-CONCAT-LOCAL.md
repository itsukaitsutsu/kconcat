# Running Concat locally on Windows

Written for your machine — **Windows 10/11 x64, i7-10700F, 16 GB RAM, RX 6600 XT**
— against `jub0t/Concat` at `main` (`2d3a43e`), v0.2.7.

Every version, URL and environment variable below is taken from the repository's
own CI (`.github/workflows/ci.yml`) and its pinned dependency list
(`scripts/ci/deps.lock`), not guessed. All four download URLs were checked
reachable on 2026-10-13.

You said rustup-init is already installed, so step 1 is about what rustup needs
beside it.

---

## 0. The two-minute version

```powershell
# 1. The C++ toolchain. rustc on Windows means MSVC.
winget install --id Microsoft.VisualStudio.2022.BuildTools --override `
  "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# 2. cmake (builds whisper.cpp) and LLVM (libclang, for bindgen)
winget install --id Kitware.CMake
winget install --id LLVM.LLVM

# 3. Close and reopen the terminal, then let the repo fetch its own pinned
#    FFmpeg, ONNX Runtime and sherpa-onnx, verified by sha256:
cd Concat
powershell -ExecutionPolicy Bypass -File setup-windows-deps.ps1

# 4. Build from src/ — that is where the workspace and the toolchain pin live.
cd src
cargo test -p concat-project        # needs no FFmpeg: pure logic only
cargo dev                           # the editor window
```

Step 3 uses the script in this folder. If you would rather do it by hand, every
value it sets is listed in §3.

**Start a build from the "x64 Native Tools Command Prompt for VS 2022"** (it is
in the Start menu once Build Tools is in). That prompt has `cl.exe`, `link.exe`
and the SDK on PATH. From an ordinary PowerShell, cargo finds the toolchain
through the registry and usually works — but when a C build script fails with
`link.exe not found`, that is why.

---

## 1. Rust

Already done, but two things worth knowing:

- **The repository pins its own toolchain.** `src/rust-toolchain.toml` says
  `channel = "1.93.1"`, with `rustfmt` and `clippy`. Any `cargo` command run
  from `src/` makes rustup download **1.93.1** the first time — a few hundred MB,
  once, and expected. Your `rustup-init` default toolchain is not used there.
- **Everything runs from `src/`.** The Cargo workspace, the toolchain pin and the
  profiles are all in `src/`. `cd Concat && cargo build` will not work;
  `cd Concat/src && cargo build` will.

Check:

```powershell
cd Concat\src
rustup show          # should list 1.93.1 as the active override
```

If you want to pin it explicitly rather than let rustup fetch on demand:

```powershell
rustup toolchain install 1.93.1 --component rustfmt --component clippy
```

---

## 2. The C++ side

Concat spawns nothing at run time — FFmpeg is linked, whisper.cpp and sherpa-onnx
are compiled in — which is why the *build* needs a C++ toolchain at all.

| Tool | Why | Install |
|---|---|---|
| **Visual Studio 2022 Build Tools** | MSVC `cl.exe` / `link.exe` + Windows SDK. The `x86_64-pc-windows-msvc` target has no alternative. | winget command in §0, or the VS Installer with the "Desktop development with C++" workload |
| **cmake** | Builds whisper.cpp from source inside `whisper-rs-sys`. | `winget install Kitware.CMake` |
| **LLVM / libclang** | `bindgen` generates the FFmpeg and Vulkan FFI. | `winget install LLVM.LLVM` |

Build Tools is the large one (~2–7 GB with the recommended components). The
winget command in §0 installs only the C++ workload, which is what is needed.

---

## 3. The prebuilt libraries, at the versions CI pins

`scripts/ci/deps.lock` holds a URL and a sha256 for each. **Use those, not
`latest`** — the comment in that file explains why: BtbN's daily FFmpeg builds
are deleted after a fortnight and `latest` changes under a build every day.

| Library | Version | Environment variable | Also on PATH |
|---|---|---|---|
| **LLVM** | 21.1.0 | `LIBCLANG_PATH=C:\llvm\bin` | `C:\llvm\bin` |
| **FFmpeg** (BtbN GPL *shared*) | n8.1.3-9-g29e619e767, win64 | `FFMPEG_DIR=<unpacked folder>` | `<folder>\bin` |
| **ONNX Runtime** (Microsoft, win-x64) | 1.28.0 | `ORT_LIB_LOCATION=<folder>\lib`<br>`ORT_PREFER_DYNAMIC_LINK=1` | `<folder>\lib` |
| **sherpa-onnx** (win-x64 *shared MT Release*) | 1.13.7 | `SHERPA_ONNX_LIB_DIR=<folder with .lib only>` | `<separate folder with .dll only>` |

### The sherpa-onnx trap, which is worth its own paragraph

`src/crates/concat-speech/Cargo.toml` documents it at length, and getting it
wrong costs an afternoon:

> On Windows it must be [the shared library]: the only static archives k2-fsa
> publishes are /MT (static CRT), and MSVC refuses to link those into a binary
> whose every other object — Rust's own runtime, Skia, whisper — is /MD.
> LNK2038, on every object pair.
>
> Windows builds must also set `SHERPA_ONNX_LIB_DIR` to a directory holding
> **only the import .lib files** (CI does; `~/.cargo/config.toml` locally). Left
> to its own download, the sys crate finds the DLLs next to the import libraries
> and tries to copy them into the "profile directory" — which it locates by
> searching for an ancestor named after `$PROFILE`, a walk that **panics under the
> custom `app` profile**. Keeping the DLLs out of the lib directory skips that
> code path.

So: **`.lib` files in one folder, `.dll` files in another.** `SHERPA_ONNX_LIB_DIR`
points at the `.lib` folder; PATH carries the `.dll` folder. The setup script does
this split for you, exactly as CI does:

```
sherpa\sherpa-onnx-v1.13.7-win-x64-shared-MT-Release-lib\lib\sherpa-onnx-*.lib  -> sherpa-libs\
sherpa\sherpa-onnx-v1.13.7-win-x64-shared-MT-Release-lib\lib\sherpa-onnx-*.dll  -> sherpa-dlls\
```

### Setting them permanently

```powershell
[Environment]::SetEnvironmentVariable("FFMPEG_DIR",         "C:\Concat-deps\ffmpeg",           "User")
[Environment]::SetEnvironmentVariable("ORT_LIB_LOCATION",   "C:\Concat-deps\ort\lib",          "User")
[Environment]::SetEnvironmentVariable("ORT_PREFER_DYNAMIC_LINK", "1",                          "User")
[Environment]::SetEnvironmentVariable("SHERPA_ONNX_LIB_DIR","C:\Concat-deps\sherpa-libs",      "User")
[Environment]::SetEnvironmentVariable("LIBCLANG_PATH",      "C:\llvm\bin",                     "User")
```

Then **open a new terminal** — `SetEnvironmentVariable` does not touch the
session that set it.

Or put them in `~/.cargo/config.toml`, which `concat-speech` explicitly suggests
and which keeps them out of the machine's environment:

```toml
[env]
FFMPEG_DIR = "C:\\Concat-deps\\ffmpeg"
ORT_LIB_LOCATION = "C:\\Concat-deps\\ort\\lib"
ORT_PREFER_DYNAMIC_LINK = "1"
SHERPA_ONNX_LIB_DIR = "C:\\Concat-deps\\sherpa-libs"
LIBCLANG_PATH = "C:\\llvm\\bin"
```

PATH still needs `ffmpeg\bin`, `ort\lib`, `sherpa-dlls` and `C:\llvm\bin`, because
the *test binaries* load those DLLs at run time.

---

## 4. Build and run, in the order to try them

Each step needs less than the one after it, so a failure names itself.

```powershell
cd Concat\src
```

**(a) The pure-logic crate — needs no FFmpeg, no cmake, no GPU.**
`concat-project` depends only on `concat-core`, serde, serde_json and thiserror.
This is what verifies the Tranche A patch:

```powershell
cargo test -p concat-project
# expected: test result: ok. 215 passed; 0 failed
cargo fmt -p concat-project -- --check
cargo clippy -p concat-project --all-targets -- -D warnings
```

`concat-core` is std-only and builds the same way. Both together take seconds
once warm — 2.5 s for `concat-core` on a cold 2-core box.

**(b) The CLI — needs FFmpeg, no GUI, no audio device.**

```powershell
cargo run -p concat-cli -- probe some-video.mp4
cargo run -p concat-cli -- render some-video.mp4 out.mp4 --frames 120
cargo run -p concat-cli -- api "{\"method\":\"catalogue.list\",\"kind\":\"filter\"}"
```

**(c) The editor window.**

```powershell
cargo dev                            # alias for: run --bin concat --profile quick
```

`--profile quick` is the one for iterating: an optimised engine with an
unoptimised window crate and incremental compilation, so a Rust edit rebuilds in
~20 s and a `.slint` edit in ~25 s instead of 3 m 20 s. The window crate is
deliberately `opt-level = 0` in that profile — see the long comment in
`src/Cargo.toml`.

```powershell
cargo build --profile app -p concat  # the shipping binary: fat LTO, panic=abort,
                                     # stripped. Minutes to relink; this is not
                                     # the profile to iterate in.
```

**(d) The whole workspace and its tests.**

```powershell
cargo build --workspace --all-targets
cargo nextest run --workspace --build-jobs 2
```

`cargo nextest` is what CI uses (`cargo install cargo-nextest --version 0.9.148`,
config in `src/.config/nextest.toml`). It runs each test in its own process with
a per-test timeout, so a hanging test is killed and named instead of stalling the
run.

**(e) The API on a socket** — the surface a sidecar or plugin would use:

```powershell
cargo run -p concat-cli -- serve                       # 127.0.0.1:7420
cargo run -p concat-cli --features grpc -- serve --grpc 127.0.0.1:7421
```

Every connection, loopback included, must present a token as its first line. It
is the one given with `--token` or `CONCAT_API_TOKEN`, else one minted at start
and printed under the addresses. Call `version` first: its `capabilities` names
what the build serves (`events`, `json-rpc`, `unix-socket`, `grpc`, `gpu`).

---

## 5. What to expect on 16 GB / 8 cores

Honest numbers, because the first build otherwise looks like a hang:

- **`concat-core` + `concat-project`:** seconds to a minute. Do this first; it
  proves the toolchain and needs none of the native libraries.
- **The full workspace, cold:** long. CI runs it as **two parallel build jobs
  with a 60-minute timeout**, and a comment in `ci.yml` says plainly that *"one
  took most of an hour."* That is on a GitHub runner. Budget an hour or more, and
  expect the `concat` crate (Slint + Skia, ~23 000 lines of Rust and 29 790 of
  Slint) to be the slow part.
- **If the linker or rustc gets killed,** that is memory, not a code error. Drop
  the parallelism: `cargo build --workspace -j 2`, or `-j 1` for the window crate.
- **Disk:** `target/` reaches tens of GB across profiles. `src/.cargo/config.toml`
  deliberately turns incremental compilation off for `dev` and `test` because
  *"a session of edits across the workspace leaves tens of gigabytes of them
  behind."*

---

## 6. Known problems, so they are not mistaken for yours

From `ARCHITECTURE.md` §15, current as of the 11 October audit:

- **The Windows CI test job has not passed since 4 October**, and the audit says
  the reason is not visible in the public logs. A hanging cards test now skips on
  WARP. **So if `cargo nextest run --workspace` goes red on Windows, that may be
  pre-existing and not your patch.** Establish the baseline first: run the suite
  on a clean `main` checkout *before* applying anything, and compare.
- One test is singled out in CI as having hung on Windows alone —
  `every_edit_still_exports` in `concat-host`'s `export` test. CI runs it in a
  step of its own with `--no-capture` and a 15-minute timeout for exactly that
  reason.
- `scripts/locales.py` crashes on a Windows console (cp1252) unless Python runs
  with `-X utf8`.
- FFmpeg's DLLs must be on PATH for the *test binaries*, not just for the build.

---

## 7. Not needed unless you go looking for it

- **Vulkan SDK.** Only if you enable `whisper-rs`'s `vulkan` feature, which the
  workspace does not today — `concat-speech/Cargo.toml` turns on `metal` for
  macOS and nothing for Windows, so whisper.cpp builds CPU-only there. When it is
  enabled, `whisper-rs-sys`'s build script links `vulkan-1` and looks for the SDK
  through `VULKAN_SDK`. Your RX 6600 XT driver already ships the Vulkan *runtime*
  loader; the SDK is only for building against it.
- **Perl / Strawberry Perl.** Only if something pulls in `openssl-sys`. Model
  downloads go through `ureq` with rustls, so nothing in the tree should.
- **Nix.** There is a flake at the repository root that builds the window with
  every native dependency pinned — `nix build`, `nix run`, `nix develop`. Linux
  x86_64 and aarch64 only, so not your path, but it is the easiest route on Linux.
- **macOS,** for completeness: `brew install ffmpeg cmake`, then
  `FFMPEG_DIR=$(brew --prefix ffmpeg)`.

---

## 8. If a build fails, the first four things to check

1. **Are you in `src/`?** The workspace and the toolchain pin are there.
2. **Is `cl.exe` reachable?** Build from the *x64 Native Tools Command Prompt for
   VS 2022*. `where cl` should answer.
3. **Are the four variables set *in this session*?** `echo $env:FFMPEG_DIR`,
   `$env:ORT_LIB_LOCATION`, `$env:SHERPA_ONNX_LIB_DIR`, `$env:LIBCLANG_PATH`.
   A `SetEnvironmentVariable` from an earlier terminal does not reach a new one
   unless it was `"User"` or `"Machine"` scoped, and even then only new sessions.
4. **Is `SHERPA_ONNX_LIB_DIR` free of DLLs?** `LNK2038` means the /MT-vs-/MD
   mismatch of §3. A panic mentioning `$PROFILE` or a "profile directory" means
   the DLLs are sitting next to the import libraries.

`cargo build -p concat-media -vv` prints the FFmpeg probe, which names the
`libavutil.pc` / `FFMPEG_DIR` problem directly if that is the one.
