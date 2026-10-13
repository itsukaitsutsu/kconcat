# STEP-2-AND-3.md — after the installer finishes

Every command in here is **one line**. Copy one, paste it, press Enter, wait for
it to finish, then copy the next. Do not copy two at once, and do not copy
anything that wraps onto a second line in this document — if it looks like two
lines, it is one long line that your browser wrapped.

**Why the last attempts failed:** the commands had PowerShell backticks (`` ` ``)
at the end of lines, which mean "this continues below". When you paste those into
a terminal it runs the first fragment alone and errors, then runs the rest alone
and errors again. There are no backticks anywhere in this file.

---

## Before anything: close and reopen the terminal

The installers changed your PATH. **A terminal that is already open cannot see
that.** In VSCode:

1. Click the terminal panel.
2. Click the **trash can icon** (🗑) at the right of the terminal tabs — that
   kills the terminal. Do this for every tab.
3. Press **Ctrl + Shift + `** (backtick, the key above Tab) to open a fresh one.

Or simplest: **close VSCode completely and reopen it.**

Check the fresh terminal works — type this one line and press Enter:

```
cmake --version
```

The first line of output must start with **`cmake version 3.`** — for example
`cmake version 3.31.12`.

- If it says `4.` → CMake 4 is installed and it will break the build. Run
  `appwiz.cpl`, uninstall CMake, then double-click `install-build-tools.bat` again.
- If it says `'cmake' is not recognized` → the terminal is still the old one, or
  the install failed. Close VSCode entirely, reopen, try again.

Then check the other three, one line at a time:

```
git --version
```
```
where cl
```
```
where libclang
```

`git --version` should print a version. `where cl` and `where libclang` may well
print nothing at all — **that is fine and expected**, because both live in
folders that only the build environment adds to PATH. Do not chase them. The
thing that matters is that the two folders exist, which the installer's SUMMARY
already told you.

---

## Which terminal is VSCode giving you?

Look at the right-hand end of the terminal panel. There is a **`+`** with a small
down-arrow next to it. Click the arrow and pick **Command Prompt**. Use that for
everything below.

Why: the commands in this file are written to work in both, but Command Prompt
has no execution policy, no `$PSScriptRoot`, and no profile script that can
interfere. It is the boring choice and boring is what we want right now.

If you would rather use PowerShell, everything below still works except that you
must type `.\` in front of file names — noted where it matters.

---

## Step 2 — get the Concat source

You already have it, as `C:\Users\E19\Downloads\concat-main`. That folder is the
repository, downloaded as a ZIP. It works for building.

**One suggestion first:** move it out of `Downloads`. Windows Defender scans
`Downloads` hard, and the build produces tens of thousands of small files there,
which makes every rebuild slower. In File Explorer, cut `concat-main` and paste
it into `C:\dev\` (create `C:\dev` if it is not there). Then rename it to
`Concat`. So it becomes:

```
C:\dev\Concat
```

The rest of this file assumes that path. If you kept it in Downloads, substitute
your own path everywhere.

Confirm the folder is the right one — type this and press Enter:

```
dir C:\dev\Concat\scripts\ci\deps.lock
```

It must print a line containing `deps.lock`. If it says *File Not Found*, you are
pointing at the wrong folder — find where `concat-main` actually is and use that
path instead.

---

## Step 3 — the native libraries

**Double-click `run-native-deps.bat`.** That is the whole step.

It must sit in the **same folder as `setup-windows-deps.ps1`** — the root of your
Concat folder, `C:\dev\concat`, next to `ARCHITECTURE.md`. It checks that before
doing anything and tells you if the two are not together.

What it does for you:

1. Confirms both `setup-windows-deps.ps1` and `scripts\ci\deps.lock` are there.
2. **Finds your LLVM automatically** — searches PATH, five common folders, the
   Windows registry, and next to `clang.exe`. This is the part that went wrong
   before: it does not assume a location.
3. Runs the PowerShell script with the correct arguments, so you never type
   `-Skip llvm -LlvmDir ...` yourself.
4. Prints what to do next when it finishes.

It takes about 10 minutes and prints a sha256 verification for each download.
**Re-running is safe** — anything already fetched and verified is kept, so it
resumes rather than starting over.

If it fails, it says which part and keeps the script's own red text. Copy that and
ask; do not guess at a fix.

### Why not type the PowerShell command

Because it is two lines long in most editors, and **pasting a wrapped command
splits it at the line break** — which is what produced
`A positional parameter cannot be found that accepts argument 'deps.lock'`.
PowerShell ran the first fragment as a command and the rest as stray arguments.

If you ever do need to type it, the shape is:

```
powershell -NoProfile -ExecutionPolicy Bypass -File setup-windows-deps.ps1 -Skip llvm -LlvmDir C:\llvm
```

That must go in as **one line with no break anywhere in it**, from a terminal
whose current folder is `C:\dev\concat`. But use the `.bat` — it removes every
one of those ways to get it wrong.

---

## Step 3b — verify, one line at a time

Close the terminal again (trash can icon) and open a new one, so it picks up the
new environment variables. Then:

```
echo %FFMPEG_DIR%
```

Should print `C:\Concat-deps\ffmpeg`. If it prints `%FFMPEG_DIR%` unchanged, the
terminal is still the old one — close VSCode entirely and reopen.

```
echo %ORT_LIB_LOCATION%
```

Should print `C:\Concat-deps\ort\lib`.

```
echo %SHERPA_ONNX_LIB_DIR%
```

Should print `C:\Concat-deps\sherpa-libs`.

```
echo %LIBCLANG_PATH%
```

Should print a path ending in `\bin`.

Now the one check that catches the worst failure. This must print
**`File Not Found`**:

```
dir %SHERPA_ONNX_LIB_DIR%\*.dll
```

If it lists any `.dll` files, something went wrong with the split — tell me, do
not continue. (A `.dll` in that folder causes a panic that mentions `$PROFILE`,
which is a miserable thing to debug without knowing the cause.)

And this one must print a number greater than zero:

```
dir /b %SHERPA_ONNX_LIB_DIR%\*.lib | find /c ".lib"
```

---

## Step 4 — the first build

This is the moment of truth, and it is deliberately the smallest possible build.

```
cd C:\dev\Concat\src
```

```
cargo test -p concat-project
```

**What happens:** the first time, rustup downloads Rust **1.93.1** — that is the
version `src\rust-toolchain.toml` pins, a few hundred MB, once. Then it compiles
three small crates and runs tests. Expect a few minutes.

**What success looks like**, near the end:

```
test result: ok. 114 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

114 is the number on an untouched `main`. (If you already applied the Tranche A
patch it says **215**.)

**This step needs no FFmpeg, no cmake and no GPU.** `concat-project` depends only
on serde and a std-only crate. So if this fails, the problem is the Rust
toolchain and nothing else — which makes it the perfect first test.

Then the one that proves FFmpeg linked:

```
cargo build -p concat-media
```

This one takes longer, and it is where a wrong `FFMPEG_DIR` shows up. If it fails
mentioning `libavutil` or `pkg-config`, your `FFMPEG_DIR` is wrong or the terminal
did not pick it up.

---

## Step 5 — run the editor

```
cd C:\dev\Concat\src
```

```
cargo dev
```

`cargo dev` is an alias defined in `src\.cargo\config.toml`. It builds and launches
the Concat window.

**The first run is long — plan for 30 to 90 minutes** and leave it alone. CI runs
the full workspace build as two parallel jobs with a 60-minute timeout, and a
comment in `ci.yml` says one of them took most of an hour. On your i7-10700F with
16 GB, expect similar or slower.

It is compiling wgpu, Skia, Slint, sherpa-onnx and ONNX Runtime bindings. The
output scrolls steadily. **Scrolling output means it is working.**

Signs it is genuinely stuck versus working:
- Output still appearing → working, wait.
- No output for 10+ minutes and disk/CPU activity in Task Manager → still working,
  it is linking. Linking the window crate takes minutes on its own.
- No output, no CPU, no disk → stuck; Ctrl+C and show me the last 30 lines.

**If rustc or the linker gets killed, or your PC freezes:** that is the 16 GB of
RAM, not a code error. Limit the parallelism:

```
cargo dev -- -j 2
```

When it finishes, the Concat window opens.

---

## If a command errors: how to tell me usefully

Copy the **exact red text** and the **command you ran**. Three things I always
need:

1. The command, verbatim.
2. The full error, not a summary of it.
3. Which terminal — Command Prompt or PowerShell — and whether it was newly
   opened.

The single most common cause of everything so far has been a terminal that was
open before an installer ran. When in doubt: **close VSCode completely, reopen
it, try again.**

---

## Quick reference — the whole thing in order

```
double-click install-build-tools.bat        (30-60 min, approve UAC prompts)
close VSCode, reopen it
cmake --version                             (must say 3.x)
move concat-main to C:\dev\Concat
copy setup-windows-deps.ps1 into C:\dev\Concat
cd C:\dev\Concat
powershell -NoProfile -ExecutionPolicy Bypass -File setup-windows-deps.ps1 -Skip llvm -LlvmDir C:\llvm
close the terminal, open a new one
echo %FFMPEG_DIR%
dir %SHERPA_ONNX_LIB_DIR%\*.dll             (must say File Not Found)
cd C:\dev\Concat\src
cargo test -p concat-project                (114 passed)
cargo build -p concat-media                 (proves FFmpeg)
cargo dev                                   (30-90 min, then the window opens)
```
