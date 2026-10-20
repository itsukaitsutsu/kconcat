kconcat - fix for the 23 motion errors in concat-host/src/titles.rs
====================================================================

WHAT WENT WRONG
---------------
Your titles.rs is the newer version (it knows the Fade, Bounce, SlideUp and
Karaoke entrances, and a `stagger` dial), but two crates underneath it are
still the older version:

  * crates/concat-project/src/model.rs   - the document's MotionMode enum only
    has None / Pop / WordReveal / Typewriter, and CaptionMotion has no
    `stagger` field.
  * crates/concat-text/src/lib.rs        - the painter's mirror has the same
    gap: MotionMode without Fade / Bounce / SlideUp / Karaoke, and Motion
    without `stagger`.

So every `MotionMode::Karaoke`, `MotionMode::Fade`, `MotionMode::Bounce`,
`MotionMode::SlideUp` and every `style.motion.stagger` in titles.rs has
nothing to resolve to. Nothing is wrong with titles.rs - do not change it.

FILES IN THIS ZIP
-----------------
These paths are relative to C:\dev\kconcat\src (the folder the compiler was
run from), so extract this zip INTO C:\dev\kconcat\src and let it overwrite:

  crates\concat-project\src\model.rs                      (replace)
  crates\concat-text\src\lib.rs                           (replace)
  crates\concat-project\tests\caption_motion.rs            (replace)
  crates\concat\src\presets.rs                             (replace, optional)

Back up those four files first if you want to be able to undo.

PowerShell, from C:\dev\kconcat\src, after extracting the zip next to it:

  Copy-Item -Force -Recurse .\kconcat-motion-fix\crates .\

Then:

  cargo build

WHAT EACH FILE NOW HAS
----------------------
model.rs (the document)
  * MotionMode gains Fade, Bounce and SlideUp (all animate a whole text clip
    from its start, no speech timing needed) and Karaoke (highlight each
    spoken word in the accent colour). The wire names are `fade`, `bounce`,
    `slide-up`, `karaoke` in kebab-case, next to the existing `none`, `pop`,
    `word-reveal`, `typewriter`.
  * CaptionMotion gains `stagger`: the delay between successive word
    entrances.
  * Defaults move to duration 0.45 s, stagger 0.09 s, scale_from 0.35 -
    the same defaults concat-text already assumes.
  * `TextStyle::tidy` clamps the three dials: duration 0.02..2.0,
    stagger 0.0..0.22, scale_from 0.25..3.0. scale_from is now allowed
    below 1 (the modern pop grows into place); values above 1 keep the old
    shrink-to-1 behaviour.
  * Old projects still load: a motion block without `stagger` reads as the
    default, and unknown mode names fall back to `none` as before.

concat-text/src/lib.rs (the painter)
  * The same four modes and the `stagger` field on `Motion`, mirroring the
    document field for field, so titles.rs' copy loop covers every variant
    exhaustively.

tests/caption_motion.rs
  * Updated for the new defaults and the extra mode names so
    `cargo test` compiles and passes.
  * One upstream assertion is corrected: for a non-finite `scale_from` the
    test expected 3.0 (as if infinity clamped to the top of the range), but
    `tidy` falls back to the default for every non-finite dial - so the
    expected value is 0.35. With 3.0 that test fails even on the upstream
    main branch.

presets.rs (optional)
  * Lets a preset .toml write `mode = "fade" | "bounce" | "slide-up" |
    "karaoke"` and a `stagger = ...` line. Skip this file if your concat
    crate is older than the rest - the build does not need it.

WHAT THIS ZIP DELIBERATELY DOES NOT TOUCH
-----------------------------------------
Upstream's model.rs also carries a newer, unrelated change - typed tracks
(`TrackKind`, `Track::kind`). That work is not part of this error, and
dropping the upstream file on top of your tree would break the versions of
`commands/tracks.rs` and friends you have. This model.rs is your file plus
the motion change only, which is why it is safe to drop in.

VERIFIED
--------
  cargo check -p concat-project          clean
  cargo check -p concat-text             clean
  cargo test  -p concat-project --test caption_motion      5 passed
  cargo test  -p concat-text --lib                         31 passed
  plus the five motion tests copied out of titles.rs, run against these
  files: 5 passed (word-reveal, typewriter, pop, fade/bounce/slide-up,
  karaoke fallback clock).

concat (the Slint app) and concat-host were not built here - this sandbox
has no ffmpeg development libraries - but every symbol titles.rs asks for
now exists, and the type-check of titles.rs' motion code passes.

If your Slint UI is the older one, note that the inspector's motion picker
will only offer the modes it knows until crates/concat/ui/inspector/
text-panel.slint is updated too; the build is unaffected.
