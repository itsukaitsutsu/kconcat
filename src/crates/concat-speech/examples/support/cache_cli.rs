// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Argument handling for the corpus tool. Standard-library only so its
//! regression tests do not need FFmpeg or an installed model.

use std::path::PathBuf;

pub(super) const USAGE: &str = "usage: make_cache <media-file> <cache.json> <model-id> [language] [--models-dir <absolute-folder>]\n       make_cache --list-models [--models-dir <absolute-folder>]";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Options {
    pub(super) positional: Vec<String>,
    pub(super) models_dir: Option<PathBuf>,
    pub(super) list_models: bool,
}

pub(super) fn parse(args: &[String]) -> Result<Options, String> {
    let mut out = Options {
        positional: Vec::new(),
        models_dir: None,
        list_models: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--models-dir" => {
                i += 1;
                let value = args
                    .get(i)
                    .filter(|v| !v.is_empty() && !v.starts_with("--"))
                    .ok_or_else(|| "--models-dir needs an absolute folder path".to_owned())?;
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err("--models-dir must be an absolute path".to_owned());
                }
                if out.models_dir.replace(path).is_some() {
                    return Err("--models-dir was supplied more than once".to_owned());
                }
            }
            "--list-models" => out.list_models = true,
            value if value.starts_with("--") => {
                return Err(format!("unknown option {value}\n{USAGE}"));
            }
            value => out.positional.push(value.to_owned()),
        }
        i += 1;
    }
    if out.list_models {
        if !out.positional.is_empty() {
            return Err(
                "--list-models does not accept media/cache/model positional arguments".to_owned(),
            );
        }
    } else if !(3..=4).contains(&out.positional.len()) {
        return Err(USAGE.to_owned());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }
    fn dir() -> String {
        std::env::temp_dir()
            .join("cache cli model files")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn old_three_argument_invocation_is_preserved() {
        let got = parse(&args(&["clip.mp4", "cache.json", "small"])).unwrap();
        assert_eq!(got.positional, args(&["clip.mp4", "cache.json", "small"]));
        assert!(got.models_dir.is_none());
        assert!(!got.list_models);
    }
    #[test]
    fn old_language_argument_is_preserved() {
        let got = parse(&args(&["clip.mp4", "cache.json", "small", "id"])).unwrap();
        assert_eq!(got.positional[3], "id");
    }
    #[test]
    fn source_path_spaces_are_data_not_extra_arguments() {
        let media = r"D:\autotrim\mentahan inheal - people pleaser.MP4";
        let got = parse(&args(&[media, "cache.json", "small", "id"])).unwrap();
        assert_eq!(got.positional[0], media);
    }
    #[test]
    fn an_explicit_model_folder_is_available_to_the_generator() {
        let folder = dir();
        let got = parse(&args(&[
            "clip.mp4",
            "cache.json",
            "small",
            "--models-dir",
            &folder,
        ]))
        .unwrap();
        assert_eq!(got.models_dir, Some(PathBuf::from(folder)));
        assert_eq!(got.positional.len(), 3);
    }
    #[test]
    fn listing_models_requires_no_media_and_no_inference() {
        let folder = dir();
        let got = parse(&args(&["--models-dir", &folder, "--list-models"])).unwrap();
        assert!(got.list_models);
        assert!(got.positional.is_empty());
    }
    #[test]
    fn missing_relative_or_duplicate_model_folders_are_refused() {
        assert!(parse(&args(&["--models-dir"])).is_err());
        assert!(parse(&args(&["--models-dir", "--list-models"])).is_err());
        assert!(parse(&args(&["--list-models", "--models-dir", "relative-models"])).is_err());
        let folder = dir();
        assert!(
            parse(&args(&[
                "--list-models",
                "--models-dir",
                &folder,
                "--models-dir",
                &folder
            ]))
            .is_err()
        );
    }
    #[test]
    fn unknown_flags_and_incomplete_requests_are_refused() {
        assert!(parse(&args(&["--no-snap"])).is_err());
        assert!(parse(&args(&["clip.mp4", "cache.json"])).is_err());
        assert!(parse(&args(&["--list-models", "clip.mp4"])).is_err());
    }
}
