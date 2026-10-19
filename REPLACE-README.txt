KConcat cumulative replacement bundle

Extract this ZIP at the repository root, replacing the matching files. It
contains the scrollable-panel fix requested, plus the earlier caption undo and
beat-grid changes from this work pass.

Scrollable panel changes:
- PaneScroll now accepts an optional min-content-height and uses it to size its
  content and calculate overflow. Existing users default to 0px.
- Keyframes now scroll vertically (wheel and finger drag); its header/tool
  strip remain fixed. Dopesheet height accounts for every key row; curve view
  keeps a minimum usable canvas.
- Meters now scroll vertically (wheel and finger drag); its header stays put.
- The UI-1 investigation note records what changed and what remains to check.

Also included:
- Caption Enter-to-split now records one undo for an edit+split event; regression
  test included.
- Dependency-free beat-grid estimation and snapping helper, with synthetic
  pulse, tolerance, and silence tests.
- Roadmap and next-steps synchronization.

I could not compile/run the Slint UI or Rust tests here because Cargo/Rust is
unavailable. This is the implementation to test as a whole on your Windows
checkout; it is not a claim that UI runtime behavior was verified. From `src/`:
  cargo fmt --check -p concat-timing -p concat
  cargo test -p concat-timing
  cargo test -p concat --lib panes::captions
  cargo check -p concat

Back up files before replacing. If the filtered concat test finds no tests,
run `cargo test -p concat`.
