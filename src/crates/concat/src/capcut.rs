// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Conservative importer for a small, documented slice of CapCut's local
//! animation cache. CapCut's Lua, shaders and binary prefabs are not executed;
//! they are proprietary runtime formats. Instead, supported parameter curves
//! are translated to native Kconcat effects and everything else is reported
//! as unsupported rather than silently approximated.

use std::fs;
use std::path::{Path, PathBuf};

use concat_project::model::{AppliedFilter, KeyEase, ParamKey};
use serde_json::Value;

/// A native Kconcat effect translated from a CapCut draft/cache entry.
#[derive(Debug)]
pub struct ImportedAnimation {
    /// Name from the CapCut draft.
    pub name: String,
    /// The effect Kconcat can render; this is a native approximation, not
    /// CapCut's original shader/runtime output.
    pub effect: AppliedFilter,
}

/// Read a CapCut `draft_content.json` and translate the first cache entry
/// whose data format this importer understands. Cache files are read in
/// place; none are copied into the project or executed.
pub fn import_first_supported(draft_path: &Path) -> Result<ImportedAnimation, String> {
    let bytes = fs::read(draft_path).map_err(|error| format!("Cannot read draft: {error}"))?;
    let draft: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Invalid CapCut draft JSON: {error}"))?;
    let animations = draft
        .get("materials")
        .and_then(|value| value.get("material_animations"))
        .and_then(Value::as_array)
        .ok_or_else(|| "This JSON has no CapCut animation materials.".to_owned())?;

    let mut saw_animation = false;
    for material in animations {
        let Some(entries) = material.get("animations").and_then(Value::as_array) else {
            continue;
        };
        for entry in entries {
            saw_animation = true;
            let Some(folder) = cache_folder(draft_path, entry) else {
                continue;
            };
            let anim_file = folder.join("studioAnim.lsanim");
            let Ok(bytes) = fs::read(&anim_file) else {
                continue;
            };
            let Ok(data) = serde_json::from_slice::<Value>(&bytes) else {
                continue;
            };
            if let Some(effect) = radiance_glow_to_bloom(&data) {
                let name = entry
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("CapCut animation")
                    .to_owned();
                return Ok(ImportedAnimation { name, effect });
            }
        }
    }

    let detail = if saw_animation {
        "This draft has cached animations, but none contains a supported RadianceGlow exposure curve."
    } else {
        "This draft has no CapCut text/sticker animations to import."
    };
    Err(format!(
        "No supported CapCut animation found. {detail} Kconcat currently translates RadianceGlow exposure curves to its native Bloom Pulse effect; Lua, shader, image-sequence, and prefab packages are not loaded."
    ))
}

/// Resolve an entry's cache folder using its recorded path first, then the
/// standard `Cache/effect/<resource-id>/<hash>` layout relative to the draft.
fn cache_folder(draft_path: &Path, entry: &Value) -> Option<PathBuf> {
    if let Some(path) = entry.get("path").and_then(Value::as_str) {
        let recorded = PathBuf::from(path);
        if recorded.is_dir() {
            return Some(recorded);
        }
    }

    let resource_id = entry
        .get("resource_id")
        .or_else(|| entry.get("id"))
        .and_then(Value::as_str)?;
    let hash = entry
        .get("path")
        .and_then(Value::as_str)
        .and_then(|path| {
            path.trim_end_matches(|ch| ch == '/' || ch == '\\')
                .rsplit(|ch| ch == '/' || ch == '\\')
                .next()
        })?;

    for ancestor in draft_path.ancestors() {
        for root in [ancestor.join("Cache"), ancestor.join("cache")] {
            let candidate = root.join("effect").join(resource_id).join(hash);
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
        // Some exports place the selected effect folder beside Cache/effect.
        let candidate = ancestor.join("effect").join(resource_id).join(hash);
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Translate CapCut's RadianceGlow exposure animation into Kconcat's
/// Bloom Pulse amount curve. The glow colour, mask, threshold, Lua and shader
/// behaviour are intentionally not claimed as equivalent.
fn radiance_glow_to_bloom(data: &Value) -> Option<AppliedFilter> {
    let animators = data
        .get("studio_anim_params")?
        .get("effectAnimators")?
        .as_array()?;
    for animator in animators {
        let effects = animator.get("effects")?.as_array()?;
        for source in effects {
            if source.get("name")?.as_str()? != "RadianceGlow" {
                continue;
            }
            let exposure = source.get("params")?.get("exposure")?;
            let base = exposure
                .get("value")
                .and_then(Value::as_f64)
                .unwrap_or(0.45);
            let source_keys = exposure
                .get("motionKeyFrameInfo")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let max_time = source_keys
                .iter()
                .filter_map(|key| key.get("t").and_then(Value::as_f64))
                .fold(0.0_f64, f64::max);

            let mut effect = AppliedFilter::new("concat.bloom-pulse");
            let amount = |value: f64| (value * 100.0).clamp(0.0, 100.0);
            effect.params.insert("amount".to_owned(), amount(base));
            effect.params.insert("speed".to_owned(), 2.0);

            if max_time > 0.0 {
                let mut keys: Vec<ParamKey> = source_keys
                    .iter()
                    .filter_map(|key| {
                        let time = key.get("t")?.as_f64()?;
                        let value = key.get("v")?.as_f64()?;
                        Some(ParamKey {
                            at: (time / max_time).clamp(0.0, 1.0),
                            value: amount(value),
                            ease: KeyEase::IN_OUT,
                        })
                    })
                    .collect();
                keys.sort_by(|left, right| left.at.total_cmp(&right.at));
                keys.dedup_by(|left, right| (left.at - right.at).abs() < 1e-9);
                if keys.len() >= 2 {
                    effect.keys.insert("amount".to_owned(), keys);
                }
            }
            return Some(effect);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("kconcat-capcut-import-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn imports_radiance_glow_curves_from_a_local_cache() {
        let root = temp_root();
        let resource = root
            .join("Cache/effect/12345/abc123");
        fs::create_dir_all(&resource).unwrap();
        fs::write(
            resource.join("studioAnim.lsanim"),
            serde_json::to_vec(&json!({
                "studio_anim_params": {"effectAnimators": [{"effects": [{
                    "name": "RadianceGlow",
                    "params": {"exposure": {"value": 0.3, "motionKeyFrameInfo": [
                        {"t": 0.0, "v": 0.3, "it": "cubic"},
                        {"t": 1.5, "v": 0.6, "it": "cubic"},
                        {"t": 3.0, "v": 0.3, "it": "cubic"}
                    ]}}
                }]}]}
            }))
            .unwrap(),
        )
        .unwrap();
        let draft = json!({
            "materials": {"material_animations": [{"animations": [{
                "id": "12345", "resource_id": "12345", "name": "Subtle Water",
                "path": "C:/old-machine/Cache/effect/12345/abc123"
            }]}]}
        });
        let draft_path = root.join("draft_content.json");
        fs::write(&draft_path, serde_json::to_vec(&draft).unwrap()).unwrap();

        let imported = import_first_supported(&draft_path).unwrap();
        assert_eq!(imported.name, "Subtle Water");
        assert_eq!(imported.effect.id, "concat.bloom-pulse");
        assert_eq!(imported.effect.keys["amount"].len(), 3);
        assert_eq!(imported.effect.keys["amount"][1].at, 0.5);
        assert_eq!(imported.effect.keys["amount"][1].value, 60.0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reports_cache_packages_without_supported_lsanims() {
        let root = temp_root();
        fs::create_dir_all(&root).unwrap();
        let draft_path = root.join("draft_content.json");
        fs::write(
            &draft_path,
            br#"{"materials":{"material_animations":[{"animations":[{"name":"Star Cluster"}]}]}}"#,
        )
        .unwrap();
        let error = import_first_supported(&draft_path).unwrap_err();
        assert!(error.contains("No supported CapCut animation found"));
        fs::remove_dir_all(root).unwrap();
    }
}
