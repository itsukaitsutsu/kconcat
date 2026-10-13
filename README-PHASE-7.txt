PHASE 7 - FINISH LINE: DOCS AND SMOKE TEST  (for itsukaitsutsu/kconcat)
=======================================================================

WHAT THIS IS
------------
The last planned phase of the caption work. No new behaviour. It writes
down what phases 1 to 6c-4 added, the limits the phase notes already state,
and the smoke test that only you can run, on a real video.

Phases in the caption line: 1 link field, 2 captions follow edits, 3 silence
detection, 4a link wiring, 4b-1 sections list, 4b-2 sections panel,
5 autotrim, 6a estimated word times, 6b heard word times, 6c karaoke
highlight, 6c-2 colour control, 6c-3 highlights follow animation, 6c-4 switch
off/on.

FILES TO REPLACE (3 docs, no code)
----------------------------------
  ARCHITECTURE.md                 (new section 16: captions and karaoke, with
      the limits and the smoke test; one row added to "Where to look")
  README-PHASE-7.txt              (this file - new)
  CHECKPOINT.md                   (Phase 7 marked delivered)

No code changes in this phase.

WHAT TO RUN
-----------
The smoke test is section 16 of ARCHITECTURE.md, steps 1 to 8. Run them in
order, in a FRESH VSCode terminal (close VSCode fully and reopen first).
Step 1 and 2 are the commands; steps 3 to 8 are by eye in the app.

Quick summary of the commands:
  cd C:\dev\Concat\src
  cargo test -p concat-project        (expect 237 passed)
  cargo test -p concat-text           (expect 30 passed)
  cargo test -p concat-core           (expect 53 passed)
  cargo check -p concat-speech
  cargo check -p concat-host
  cargo check -p concat               (30-90 minutes the first time)

If any step fails, paste the FIRST `error` block verbatim and do not edit
around it.

WHAT WAS CHECKED IN THE SANDBOX, AND WHAT WAS NOT
-------------------------------------------------
  Docs: read against the phase notes and the code the phases touched. Each
        limit in section 16 comes from a phase README, not from a new claim.
  NOT verified: anything marked "not compiled" in the phase notes. The
        smoke test is the first real check of the app side.
