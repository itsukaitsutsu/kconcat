# Contributing to kconcat

This is a development fork of [Concat](https://github.com/jub0t/Concat). Open fork-specific issues and pull requests in [itsukaitsutsu/kconcat](https://github.com/itsukaitsutsu/kconcat), not in upstream support channels. Discuss changes beyond small fixes before starting substantial work.

## Reports and support boundaries

Include the exact commit/build, operating system, reproduction steps, expected result, actual result, and relevant sanitized logs. Use minimal synthetic media where possible; do not upload private projects, credentials, personal data, or order/payment details publicly. Report vulnerabilities using [SECURITY.md](SECURITY.md).

Community participation is voluntary. Reporting a bug does not guarantee a fix, update, review deadline, or ongoing maintenance. Any paid installation assistance must be defined by a completed order agreement; the [customer agreement pack](makethisforagreementuser/agreementREADME.md) is still a draft. Mandatory legal rights and express service promises remain unaffected.

## Setting up

You will need Rust (see `rust-version` in `src/Cargo.toml`), the FFmpeg
7+ development libraries (`brew install ffmpeg`; see `src/README.md` for
Windows and Linux), cmake and a C++ compiler. Then:

```sh
cd src && cargo run -p concat
```

That is the editor window. Everything - the engine, the host layer and the
Slint UI - is one Cargo workspace under `src/`.

## Layout

| Path | What lives there |
|---|---|
| `src/crates/` | The engine (core, media, render, export, project), the host layer (`concat-host`, `concat-speech`), the CLI, and `concat`, the Slint editor window |

[`src/README.md`](src/README.md) explains how the crates fit together and
where the sharp edges are. Read it before touching the engine.

## Checks

Run these before opening a PR:

```sh
cd src && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

If you added or changed text the interface shows, also run
`python3 scripts/locales.py` so the string inventory follows; CI checks it.
Translations live in one JSON file per language — see
[`TRANSLATING.md`](TRANSLATING.md).

If you added or changed a downloadable model, put it in
[`models/manifest.toml`](models/manifest.toml) as well as the engine table it
belongs to, and run `python3 scripts/models.py --check`; CI checks that too.
A maintainer then runs the *Mirror models* workflow, which fetches the model,
records its digest in both places and publishes it to the mirror the app
downloads from.

Write what happens through the `log` facade — `log::info!`, `log::warn!`,
`log::error!` — rather than to standard output or standard error. Every run
writes `<app data>/logs/concat-<when>.log`, and a packaged build has no
terminal for anything that goes anywhere else; `CONCAT_LOG=debug` turns the
level up. See `src/crates/concat-host/src/logs.rs`.

New source files need a licence header — see below. Match the style of the code
around you; the engine avoids cleverness on purpose.

## Licensing your contribution

The workspace declares **AGPL-3.0-or-later**, with the existing [plugin exception](LICENSE-EXCEPTIONS.md). See the full [LICENSE](LICENSE). Submit only work you have the right to contribute under applicable project terms. Preserve copyright and license notices and document the origin, version, and license of imported code, models, fonts, codecs, and assets. Resolve missing or incompatible permissions before merging or distributing them.

The existing [CLA.md](CLA.md) identifies the upstream Concat project and its maintainers as its beneficiary. It is retained as upstream documentation, not silently rewritten as an agreement with this fork. For upstream submissions, follow upstream's current CLA/sign-off requirements. Fork maintainers must clarify any separate contributor agreement before requesting consent; the customer package agreements are not a contributor CLA.

Preserve the source file's applicable SPDX identifier and copyright notices. For new files licensed under the workspace's AGPL-3.0-or-later terms, use:

```rust
// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: YEAR ACTUAL COPYRIGHT HOLDER
```

Replace the example year and holder accurately. Do not remove existing attribution or claim another author's work as your own.

## Release and documentation contributions

- Identify the exact build/commit and complete Corresponding Source for every distributed AGPL binary. Provide compliant source access before distributing it, with the source link beside download/purchase access.
- Keep the full license, change notices, copyright notices, and [third-party notices](THIRD_PARTY_NOTICES.md) with releases. Check commercial redistribution rights for every bundled component.
- Do not add store, payment, support, or plugin terms that restrict AGPL rights or relabel covered modifications as proprietary software.
- Record tested systems and known issues honestly. Do not describe historical test results as current validation.
- For privacy-related changes, document actual network requests, media uploads, model downloads, diagnostics, vendors, retention, and deletion behavior. Do not claim “fully offline” or “no data collected” without verification.
- Keep legal templates visibly marked as drafts until placeholders are completed and reviewed by Indonesian counsel; prepare aligned Bahasa Indonesia documents where required.

## Trademarks and conduct

Use your own product identity and icon for modified distributions unless permission has been obtained; see the upstream [trademark policy](TRADEMARK.md). Do not imply upstream endorses or supports this fork.

The [Code of Conduct](CODE_OF_CONDUCT.md) applies to this fork's community spaces. Community moderation does not remove recipients' software-license rights.
