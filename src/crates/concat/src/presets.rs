// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Text presets: a title's look, named, that the library's Text page
//! offers as a card. Presets are data, never code: the built-in ones are
//! folders under concat's text-presets/, embedded in the binary in the
//! same form a user's take, and their names are their authors' own and
//! are not translated. A folder of TOML files beside the app's settings
//! adds more, and a preset shared between people travels as a folder with
//! its font inside.
//!
//! A preset file, `text-presets/<anything>.toml` or
//! `text-presets/<anything>/preset.toml` under the config directory:
//!
//! ```toml
//! id = "studio.big-red"        # unique for ever: project files store it
//! name = "Big Red"
//! font = "BigRed.ttf"          # optional: a file beside this one
//! offsetY = 0.3                # optional: a frame-height fraction from centre
//! language = "Latin"           # optional: the writing it is for; Latin when absent
//! order = 10                   # optional: where it sorts; absent sorts last, by name
//! sample = "Aa"                # optional: what its card draws; see TextPreset::sample
//! kind = "caption"             # optional: the group it belongs to; "title" when absent
//!
//! [style]                      # any of a title's fields; the rest default
//! fontFamily = "Big Red"
//! fontSize = 0.1
//! fontWeight = 700
//! color = "#ff0000"
//! strokeColor = "#000000"
//! strokeWidth = 0.01
//! align = "center"
//! highlightColor = "#39ff88"   # optional: the karaoke accent; absent is off
//! ```
//!
//! The font is installed when the preset is first used, not when it is
//! read: copied once into the app's own `fonts/` folder if it is not there
//! already, and registered on the project so the title painter finds it.
//! A preset used on two machines therefore renders the same on both, and
//! a project carries the font's path the way it carries any font added by
//! hand.

use std::path::{Path, PathBuf};

use concat_host::AppDirs;
use concat_project::model::{CaptionMotion, MotionMode, TextAlign, TextStyle, TextTransform};
use serde::Deserialize;

/// One look a title can be given.
pub struct TextPreset {
    /// Stable for ever; "default" is the plain title.
    pub id: String,
    pub name: String,
    /// The look, with `content` as the words the card places.
    pub style: TextStyle,
    /// Where the title sits, as a frame-height fraction from the centre,
    /// when the preset has an opinion - a lower third does.
    pub offset_y: Option<f64>,
    /// A font file the preset brings with it, resolved to a path.
    pub font: Option<PathBuf>,
    /// The writing its face is made for, as the font libraries name it:
    /// "Latin", "Japanese", "Hebrew". What the library filters and sorts
    /// presets by; a preset file that says nothing is Latin.
    pub language: String,
    /// The group the look belongs to (roadmap C2): "title" for the Text
    /// page's shelves, "caption" for the caption looks. A file that says
    /// nothing is a title, so every preset that shipped before the tag
    /// stays where it was.
    pub kind: String,
    /// What its card draws in the look: the file's `sample`, else "Aa" for
    /// Latin and the first two characters of its words for any other
    /// writing, so a Japanese card shows Japanese.
    pub sample: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PresetFile {
    id: String,
    name: String,
    #[serde(default)]
    font: Option<String>,
    #[serde(default)]
    offset_y: Option<f64>,
    #[serde(default)]
    language: Option<String>,
    /// Where it sorts among its neighbours; absent sorts last, by name.
    #[serde(default)]
    order: Option<i64>,
    /// What the card draws; see [`TextPreset::sample`].
    #[serde(default)]
    sample: Option<String>,
    /// The group tag; see [`TextPreset::kind`]. Absent is "title".
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    style: PresetStyle,
    /// The way-in block (roadmap C5, schema v1, owner-approved 16 Oct
    /// 2026): `[motion]` beside `[style]`. Absent - and in every preset
    /// that shipped before the schema - the style keeps its default,
    /// mode none. Older builds ignore the block (no
    /// `deny_unknown_fields`), which the schema checked before approving.
    #[serde(default)]
    motion: Option<PresetMotion>,
}

/// The `[motion]` block's fields, every one optional. The schema writes
/// `scale_from`; the rest of a preset file is camelCase, so the reader
/// takes both spellings.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PresetMotion {
    mode: Option<String>,
    duration: Option<f64>,
    stagger: Option<f64>,
    #[serde(alias = "scale_from")]
    scale_from: Option<f64>,
}

impl PresetMotion {
    /// The block resolved over the schema's defaults and clamped the way
    /// `PresetStyle::over` clamps - an unrecognised mode is no motion,
    /// never a parse failure.
    fn resolved(self) -> CaptionMotion {
        let base = CaptionMotion::default();
        CaptionMotion {
            mode: match self.mode.as_deref().map(str::trim) {
                Some("fade") => MotionMode::Fade,
                Some("pop") => MotionMode::Pop,
                Some("bounce") => MotionMode::Bounce,
                Some("slide-up") => MotionMode::SlideUp,
                Some("word-reveal") => MotionMode::WordReveal,
                Some("typewriter") => MotionMode::Typewriter,
                Some("karaoke") => MotionMode::Karaoke,
                _ => MotionMode::None,
            },
            duration: self.duration.unwrap_or(base.duration).clamp(0.02, 2.0),
            stagger: self.stagger.unwrap_or(base.stagger).clamp(0.0, 0.22),
            scale_from: self.scale_from.unwrap_or(base.scale_from).clamp(0.25, 3.0),
        }
    }
}

/// A title's fields, every one optional, laid over the default style.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PresetStyle {
    content: Option<String>,
    font_family: Option<String>,
    font_size: Option<f64>,
    font_weight: Option<f64>,
    italic: Option<bool>,
    color: Option<String>,
    align: Option<String>,
    opacity: Option<f64>,
    stroke_width: Option<f64>,
    stroke_color: Option<String>,
    shadow: Option<bool>,
    background: Option<String>,
    background_radius: Option<f64>,
    background_padding_x: Option<f64>,
    background_padding_y: Option<f64>,
    line_height: Option<f64>,
    tracking: Option<f64>,
    max_width: Option<f64>,
    max_height: Option<f64>,
    /// The karaoke accent (roadmap C2/C3, owner A10): a preset that names
    /// one ships with its highlight on; absent or empty is off.
    highlight_color: Option<String>,
    /// The render-time case transform (roadmap C4): "uppercase" draws the
    /// words in caps; absent or an unrecognised name draws them as stored.
    /// The preset never rewrites the transcript - the transform lives on
    /// the style and the painter applies it.
    transform: Option<String>,
}

impl PresetStyle {
    fn over(self, name: &str) -> TextStyle {
        let base = TextStyle::default();
        TextStyle {
            content: self.content.unwrap_or_else(|| name.to_owned()),
            font_family: self
                .font_family
                .unwrap_or_else(|| "Hanken Grotesk".to_owned()),
            font_size: self.font_size.unwrap_or(base.font_size).clamp(0.005, 1.0),
            font_weight: self
                .font_weight
                .unwrap_or(base.font_weight)
                .clamp(100.0, 900.0),
            italic: self.italic.unwrap_or(false),
            color: self.color.unwrap_or(base.color),
            align: match self.align.as_deref() {
                Some("left") => TextAlign::Left,
                Some("right") => TextAlign::Right,
                _ => TextAlign::Center,
            },
            opacity: self.opacity.unwrap_or(1.0).clamp(0.0, 1.0),
            stroke_width: self.stroke_width.unwrap_or(0.0).max(0.0),
            stroke_color: self.stroke_color.unwrap_or(base.stroke_color),
            shadow: self.shadow.unwrap_or(true),
            background: self.background.unwrap_or_default(),
            background_radius: self
                .background_radius
                .unwrap_or(base.background_radius)
                .max(0.0),
            background_padding_x: self
                .background_padding_x
                .unwrap_or(base.background_padding_x)
                .max(0.0),
            background_padding_y: self
                .background_padding_y
                .unwrap_or(base.background_padding_y)
                .max(0.0),
            line_height: self.line_height.unwrap_or(base.line_height).max(0.5),
            tracking: self.tracking.unwrap_or(0.0),
            max_width: self.max_width.unwrap_or(0.0).max(0.0),
            max_height: self.max_height.unwrap_or(0.0).max(0.0),
            motion: base.motion,
            // The karaoke accent travels with the preset (C2/C3): the
            // three caption looks that the manual gives one ship with the
            // highlight on; everything else stays off until the user turns
            // it on with the existing option (owner A10).
            highlight_color: self
                .highlight_color
                .map(|color| color.trim().to_owned())
                .unwrap_or_default(),
            // C4: the transform travels with the preset like every other
            // field of the look; an unrecognised name draws as stored.
            transform: match self.transform.as_deref().map(str::trim) {
                Some("uppercase") => TextTransform::Uppercase,
                _ => TextTransform::None,
            },
        }
    }
}

/// Where the user's presets live.
pub fn dir(dirs: &AppDirs) -> PathBuf {
    dirs.config.join("text-presets")
}

/// The presets that ship with the app: the files under text-presets/,
/// embedded at build time (see build.rs), read exactly as a user's are.
/// In the order their files ask for, then by name.
pub fn builtin() -> Vec<TextPreset> {
    mod embedded {
        include!(concat!(env!("OUT_DIR"), "/text_presets.rs"));
    }
    let mut presets: Vec<(i64, TextPreset)> = embedded::BUILTIN
        .iter()
        .filter_map(|text| parse(text, None))
        .collect();
    presets.sort_by(|(a_order, a), (b_order, b)| {
        a_order.cmp(b_order).then_with(|| a.name.cmp(&b.name))
    });
    presets.into_iter().map(|(_, preset)| preset).collect()
}

/// The presets in the user's folder. A file that does not parse is
/// skipped: one broken preset should not empty the page.
pub fn user(dirs: &AppDirs) -> Vec<TextPreset> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir(dirs)) else {
        return found;
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter_map(|path| {
            if path.is_dir() {
                let inner = path.join("preset.toml");
                inner.is_file().then_some(inner)
            } else {
                (path.extension().and_then(|e| e.to_str()) == Some("toml")).then_some(path)
            }
        })
        .collect();
    paths.sort();
    for path in paths {
        if let Some(preset) = read(&path) {
            found.push(preset);
        }
    }
    found
}

fn read(path: &Path) -> Option<TextPreset> {
    let text = std::fs::read_to_string(path).ok()?;
    parse(&text, path.parent()).map(|(_, preset)| preset)
}

/// One preset file's text, and where it sorts. `folder` is where a font
/// it names is looked for; a built-in preset has none and brings no font.
fn parse(text: &str, folder: Option<&Path>) -> Option<(i64, TextPreset)> {
    let file: PresetFile = toml::from_str(text).ok()?;
    let id = file.id.trim().to_owned();
    if id.is_empty() {
        return None;
    }
    let font = file
        .font
        .filter(|name| !name.trim().is_empty())
        .and_then(|name| folder.map(|folder| folder.join(name.trim())));
    let order = file.order.unwrap_or(i64::MAX);
    let language = file
        .language
        .map(|language| language.trim().to_owned())
        .filter(|language| !language.is_empty())
        .unwrap_or_else(|| "Latin".to_owned());
    let kind = file
        .kind
        .map(|kind| kind.trim().to_owned())
        .filter(|kind| !kind.is_empty())
        .unwrap_or_else(|| "title".to_owned());
    let style = file.style.over(&file.name);
    // C5: a preset's [motion] block rides with the look on apply - the
    // schema's option 1, the one the owner selected. No block, no motion.
    let style = match file.motion {
        Some(motion) => TextStyle {
            motion: motion.resolved(),
            ..style
        },
        None => style,
    };
    let sample = file
        .sample
        .map(|sample| sample.trim().to_owned())
        .filter(|sample| !sample.is_empty())
        .unwrap_or_else(|| {
            if language.eq_ignore_ascii_case("latin") {
                "Aa".to_owned()
            } else {
                style
                    .content
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .take(2)
                    .collect()
            }
        });
    Some((
        order,
        TextPreset {
            name: if file.name.trim().is_empty() {
                id.clone()
            } else {
                file.name.trim().to_owned()
            },
            id,
            style,
            offset_y: file.offset_y,
            font,
            language,
            kind,
            sample,
        },
    ))
}

/// Every preset the page offers: the built-in ones, then the user's. A
/// user preset with a built-in id replaces it, which is how a look can be
/// overridden without a second card for it.
pub fn all(dirs: &AppDirs) -> Vec<TextPreset> {
    let mut presets = builtin();
    for preset in user(dirs) {
        match presets.iter().position(|held| held.id == preset.id) {
            Some(index) => presets[index] = preset,
            None => presets.push(preset),
        }
    }
    presets
}

/// Where the app keeps fonts: the ones presets brought, and the ones the
/// user imported from the Text inspector. A font here outlives the project
/// it was imported into - it is offered to every title, and registered on
/// a project the moment a title in it is set in the face.
pub fn fonts_dir(dirs: &AppDirs) -> PathBuf {
    dirs.config.join("fonts")
}

/// Copies a font file into [`fonts_dir`], unless one of that name is there
/// already, and says where it landed. None when it cannot be copied.
pub fn install_font_file(dirs: &AppDirs, source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?;
    let fonts = fonts_dir(dirs);
    let installed = fonts.join(name);
    if !installed.is_file() {
        std::fs::create_dir_all(&fonts).ok()?;
        std::fs::copy(source, &installed).ok()?;
    }
    Some(installed)
}

/// Every family in [`fonts_dir`], with the file that carries it: what the
/// font picker offers beyond the bundled face and the system's, in the
/// order the files are found. A file that is not a font is passed over.
pub fn installed_fonts(dirs: &AppDirs) -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(fonts_dir(dirs)) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let file = path.to_string_lossy().into_owned();
        for family in concat_text::families_in(&path) {
            out.push((family, file.clone()));
        }
    }
    out
}

/// Puts the preset's font where the app keeps fonts, if it is not there
/// already, and says how to register it: the family the style names and
/// the installed file. None for a preset that brings no font, or whose
/// file cannot be read.
pub fn install_font(dirs: &AppDirs, preset: &TextPreset) -> Option<(String, String)> {
    let source = preset.font.as_ref().filter(|path| path.is_file())?;
    let installed = install_font_file(dirs, source)?;
    let family = preset.style.font_family.trim().trim_matches('"').to_owned();
    if family.is_empty() {
        return None;
    }
    Some((family, installed.to_string_lossy().into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_ids_are_unique_and_the_plain_title_is_first() {
        let presets = builtin();
        assert_eq!(presets[0].id, "default");
        for (index, preset) in presets.iter().enumerate() {
            assert!(
                !presets[..index].iter().any(|held| held.id == preset.id),
                "{} twice",
                preset.id
            );
        }
    }

    /// A file under text-presets/ that does not parse would be skipped
    /// without a word, and its card would simply be missing.
    #[test]
    fn every_shipped_preset_file_parses() {
        mod embedded {
            include!(concat!(env!("OUT_DIR"), "/text_presets.rs"));
        }
        for text in embedded::BUILTIN {
            assert!(parse(text, None).is_some(), "does not parse:\n{text}");
        }
        assert_eq!(builtin().len(), embedded::BUILTIN.len());
    }

    /// C2's prescribed test: a preset with a highlight round-trips through
    /// preset.toml - the file's `highlightColor` arrives on the style, and
    /// the group tag with it.
    #[test]
    fn a_highlight_in_a_preset_file_survives_the_round_trip() {
        let (_, preset) = parse(
            "id = \"t.karaoke\"\nname = \"Karaoke\"\nkind = \"caption\"\n\n[style]\nhighlightColor = \"#39ff88\"\n",
            None,
        )
        .expect("parses");
        assert_eq!(preset.kind, "caption");
        assert_eq!(preset.style.highlight_color, "#39ff88");
        // A file that says nothing is a title look with no highlight, which
        // is how every preset that shipped before the tag behaves.
        let (_, plain) = parse("id = \"t.plain\"\nname = \"Plain\"\n", None).expect("parses");
        assert_eq!(plain.kind, "title");
        assert!(plain.style.highlight_color.is_empty());
    }

    /// The eight caption looks (manual B-caption-styles §4, owner A9) ship
    /// tagged as a group, and the three the manual gives a highlight to
    /// carry it - the rest start with it off (owner A10).
    #[test]
    fn the_eight_caption_presets_ship_tagged_and_three_carry_a_highlight() {
        let presets = builtin();
        let find = |id: &str| {
            presets
                .iter()
                .find(|preset| preset.id == id)
                .unwrap_or_else(|| panic!("{id} ships"))
        };
        for id in [
            "kconcat.pop-bold",
            "kconcat.boxed",
            "kconcat.karaoke",
            "kconcat.minimal",
            "kconcat.hard-shadow",
            "kconcat.upper-tight",
            "kconcat.low-third-cap",
            "kconcat.clean-bilingual",
        ] {
            assert_eq!(find(id).kind, "caption", "{id} is tagged as a group");
        }
        assert_eq!(find("kconcat.pop-bold").style.highlight_color, "#ffd21f");
        assert_eq!(find("kconcat.karaoke").style.highlight_color, "#39ff88");
        assert_eq!(find("kconcat.upper-tight").style.highlight_color, "#ffffff");
        // C4: upper-tight is the caps look the manual names ("CapCut styles
        // are usually ALL CAPS", §1); it ships the render-time transform,
        // and the other seven draw as stored.
        assert_eq!(
            find("kconcat.upper-tight").style.transform,
            TextTransform::Uppercase
        );
        assert_eq!(find("kconcat.boxed").style.transform, TextTransform::None);
        for id in [
            "kconcat.boxed",
            "kconcat.minimal",
            "kconcat.hard-shadow",
            "kconcat.low-third-cap",
            "kconcat.clean-bilingual",
        ] {
            assert!(
                find(id).style.highlight_color.is_empty(),
                "{id} starts with the highlight off"
            );
        }
        // A preset from before the tag stays a title look.
        assert_eq!(find("concat.bold-caption").kind, "title");
    }

    /// C4's prescribed test at the preset layer: a preset with uppercase
    /// renders uppercase, and the stored caption text is unchanged - here,
    /// the file's `transform` arrives on the style and `display_content`
    /// does the rest, while `content` keeps its casing.
    #[test]
    fn a_transform_in_a_preset_file_survives_the_round_trip() {
        let (_, preset) = parse(
            "id = \"t.upper\"\nname = \"Upper\"\n\n[style]\ntransform = \"uppercase\"\n",
            None,
        )
        .expect("parses");
        assert_eq!(preset.style.transform, TextTransform::Uppercase);
        let mut styled = preset.style;
        styled.content = "halo dunia".to_owned();
        assert_eq!(styled.display_content(), "HALO DUNIA");
        assert_eq!(
            styled.content, "halo dunia",
            "the transcript keeps its casing"
        );

        let (_, plain) = parse("id = \"t.plain\"\nname = \"Plain\"\n", None).expect("parses");
        assert_eq!(plain.style.transform, TextTransform::None);

        // An unrecognised name is not a parse failure; it draws as stored.
        let (_, odd) = parse(
            "id = \"t.odd\"\nname = \"Odd\"\n\n[style]\ntransform = \"smallcaps\"\n",
            None,
        )
        .expect("parses");
        assert_eq!(odd.style.transform, TextTransform::None);
    }

    /// C5's prescribed test at the preset layer: a preset yields motion
    /// on apply - the file's `[motion]` block arrives on the style the
    /// apply hands over.
    #[test]
    fn a_motion_block_in_a_preset_file_lands_on_the_style() {
        let (_, preset) = parse(
            "id = \"t.pop\"\nname = \"Pop\"\n\n[motion]\nmode = \"pop\"\nduration = 0.25\nstagger = 0.12\nscale_from = 1.3\n",
            None,
        )
        .expect("parses");
        assert_eq!(preset.style.motion.mode, MotionMode::Pop);
        assert!((preset.style.motion.duration - 0.25).abs() < 1e-9);
        assert!((preset.style.motion.stagger - 0.12).abs() < 1e-9);
        assert!((preset.style.motion.scale_from - 1.3).abs() < 1e-9);

        // The camelCase spelling reads too, the rest of a preset file's
        // habit.
        let (_, camel) = parse(
            "id = \"t.rev\"\nname = \"Rev\"\n\n[motion]\nmode = \"word-reveal\"\nscaleFrom = 1.2\n",
            None,
        )
        .expect("parses");
        assert_eq!(camel.style.motion.mode, MotionMode::WordReveal);
        assert!((camel.style.motion.scale_from - 1.2).abs() < 1e-9);

        // No block is no motion, which is every preset that shipped
        // before the schema.
        let (_, plain) = parse("id = \"t.plain\"\nname = \"Plain\"\n", None).expect("parses");
        assert_eq!(plain.style.motion.mode, MotionMode::None);

        // An unrecognised mode is no motion, not a parse failure.
        let (_, odd) = parse(
            "id = \"t.odd\"\nname = \"Odd\"\n\n[motion]\nmode = \"fireworks\"\n",
            None,
        )
        .expect("parses");
        assert_eq!(odd.style.motion.mode, MotionMode::None);
    }

    #[test]
    fn a_partial_style_lays_over_the_default() {
        let file: PresetFile = toml::from_str(
            r##"
            id = "t.red"
            name = "Red"
            offsetY = 0.3
            [style]
            color = "#ff0000"
            align = "left"
            "##,
        )
        .expect("parses");
        let style = file.style.over(&file.name);
        assert_eq!(style.color, "#ff0000");
        assert_eq!(style.align, TextAlign::Left);
        assert_eq!(style.content, "Red");
        assert_eq!(style.font_weight, TextStyle::default().font_weight);
        assert_eq!(file.offset_y, Some(0.3));
    }
}
