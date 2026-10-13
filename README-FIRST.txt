CONCAT ON WINDOWS - START HERE
==============================

Do these in order. Do not skip ahead.

  1. DOUBLE-CLICK  install-build-tools.bat

     It starts by printing a DETECTED section listing what is already on
     the PC and exactly where it found it. If all four tools are there it
     says so and exits without downloading anything.

     Otherwise it installs only what is missing. Takes 30-60 minutes if VS
     Build Tools is one of them. Approve every UAC prompt.

     Safe to re-run: it re-detects from scratch each time. It also reuses
     an installer already sitting in the download folder, so if a download
     failed you can fetch that one file in a browser, drop it in the folder
     the error names, and run this again.

  2. CLOSE VSCode completely, then reopen it.
     Not optional. A terminal that was open during the install cannot see
     the new PATH, and that is the cause of most "not recognized" errors.

  3. DOUBLE-CLICK  fetch-native-deps.bat
     From anywhere - it finds your Concat folder by itself. About 10
     minutes. Pure batch: no PowerShell, so nothing to paste and no
     argument parsing to go wrong. Downloads and sha256-verifies FFmpeg,
     ONNX Runtime and sherpa-onnx at the versions deps.lock pins, then
     sets the environment variables. Safe to re-run: verified files are
     kept, so it resumes.

  4. DOUBLE-CLICK  open-a-build-terminal.bat
     A Command Prompt opens with everything on PATH for that window.
     Run cargo commands IN THAT WINDOW. Use this instead of the VSCode
     terminal and PATH stops being a problem entirely.

  5. In that window:  cargo test -p concat-project
     Expect "114 passed". Then STEP-2-AND-3.md from "Step 3b" onwards.

  If a build fails:  DOUBLE-CLICK  diagnose-concat-build.bat
     It changes nothing, only looks, and prints what it finds - the five
     environment variables, whether FFmpeg's headers are actually where
     FFMPEG_DIR points, whether libclang.dll is where LIBCLANG_PATH
     points, and the state of the rest. Copy the whole output and send it
     back rather than guessing.

  The PowerShell route (setup-windows-deps.ps1, run-native-deps.bat) has
  been removed from this folder. Four attempts to invoke that script
  produced an argument-binding error that could not be reproduced or
  explained, so step 3 was rewritten in batch to remove it entirely.

  Everything here is pure batch. There is nothing to paste anywhere.


WHY THE EARLIER ATTEMPTS FAILED
-------------------------------
Three separate things, none of them your fault:

  * The commands used PowerShell backticks (`) at the end of lines to mean
    "continues below". Pasting those into a terminal runs the first half
    alone, which errors, then the second half alone, which errors again.
    There are no backticks anywhere in STEP-2-AND-3.md.

  * winget does not exist on your Windows. It ships inside App Installer,
    which Windows 10 only includes on newer builds. install-build-tools.bat
    does not depend on it, and installs it at the end as a bonus.

  * The setup script assumed it knew where it was on disk. It was wrong
    about your layout and crashed before doing anything. It now searches
    for your Concat folder instead of guessing.

  * The PowerShell command to run that script is long enough to wrap onto
    two lines in an editor. Pasting a wrapped command splits it at the
    break, so PowerShell ran half of it and then treated the other half as
    stray arguments. run-native-deps.bat exists so there is nothing to
    paste at all.


IF SOMETHING ERRORS
-------------------
Copy the exact red text and the exact command you ran, and ask. Do not
guess at a fix and do not re-run a command that already failed - that is
how one small problem becomes four.


THE OTHER FILES (reference only, not needed now)
------------------------------------------------
  RUN-CONCAT-LOCAL.md     every version, variable and failure mode, in detail
  NO-WINGET.md            manual download links, and the winget install
  SETUP-STEPS.md          the same walkthrough as STEP-2-AND-3.md, terser
  requirements.txt        Python packages - NOT needed to run Concat at all
